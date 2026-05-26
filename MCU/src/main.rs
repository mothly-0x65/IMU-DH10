#![no_std]
#![no_main]

use defmt::{error, info, warn};
use embassy_executor::Spawner;
use embassy_stm32::eth::{Ethernet, PacketQueue};
use embassy_stm32::{bind_interrupts,
                    eth, peripherals,
                    exti::ExtiInput,
                    gpio::{Input, Level, Output, Pull, Speed},
                    mode::Async,
                    spi::{self, Spi, Config},
                    time::Hertz,
                    interrupt,
                    dma
};
use embassy_net::{Ipv4Address, Ipv4Cidr, StaticConfigV4, Stack, StackResources};
use lan8742::Lan8742;
use embassy_net::udp::{UdpSocket, PacketMetadata};
use embassy_stm32::interrupt::SPI3;
use embassy_stm32::pac::exti::Exti;
use static_cell::StaticCell;
use embassy_time::{Duration, Timer};

use ad7768_pac::{calc_odr_hz, code_to_mv, AdcFrame, DecRate, MclkDiv};
use ad7768_pac::registers::*;

use ad7768::{Ad7768, Ad7768Config, Error};
use ad7768_pac::registers::registers::Cal24;

mod mpu;

#[repr(C, packed)]
struct ImuPacket {
    id: u16, // 0xAD77 - identifier for the imu packet
    channel_count: u8, // always 8
    sequence: u32, // increments every packet, lets PLC detect drops
    channels: [f32; 8], // the 8 ADC channel values
}

bind_interrupts!(struct ETHIrqs {
    ETH => eth::InterruptHandler;
});

bind_interrupts!(struct SPI4Irqs {
    DMA1_STREAM0 => dma::InterruptHandler<peripherals::DMA1_CH0>;
    DMA1_STREAM1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
});

bind_interrupts!(struct SPI3Irqs {
    DMA1_STREAM2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
    DMA1_STREAM3 => dma::InterruptHandler<peripherals::DMA1_CH3>;
});

bind_interrupts!(struct DrdyIrq {
    EXTI15_10 => embassy_stm32::exti::InterruptHandler<interrupt::typelevel::EXTI15_10>;}
);


#[link_section = ".eth_buffers"]
static mut PACKETS: PacketQueue<4, 4> = PacketQueue::new();

static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();


// this task runs the newtork stack forever in the background
#[embassy_executor::task]
async fn net_task(stack: &'static Stack<Ethernet<'static, ETH, Lan8742>>) {
    stack.run().await 
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    mpu::init();

    let p = embassy_stm32::init(Default::default()); // import peripherals
    info!("booting");

    let mut spi_cfg = Config::default();
    spi_cfg.frequency = Hertz(10_000_000);

    let ctrl_spi: Spi<Async, spi::mode::Master> = Spi::new(
        p.SPI4,
        p.PE2,      // SCK  → AD7768 SCLK
        p.PE6,      // MOSI → AD7768 SDI
        p.PE5,      // MISO ← AD7768 SDO
        p.DMA1_CH0,
        p.DMA1_CH1,
        SPI4Irqs,
        spi_cfg,
    );

    // SPI2: data port, slave — AD7768 drives DCLK so STM32 must be slave here.
    // Wire: AD7768 DCLK (pin 28) → PC10 (SPI2 SCK)
    //       AD7768 DOUT0 (pin 27) → PC12 (SPI2 MISO)
    // No frequency set on slave — the clock comes from the AD7768.
    let data_spi = Spi::new_slave(
        p.SPI3,
        p.PC10,
        p.PC12,
        p.PC13,
        p.PB0,
        p.DMA1_CH2,
        p.DMA1_CH3,
        SPI3Irqs,
        spi::Config::default(),
    );

    let cs = Output::new(p.PE4, Level::High, Speed::High);
    let rst = Output::new(p.PD0, Level::High, Speed::Low);
    let drdy = ExtiInput::new(p.PA15, p.EXTI15, Pull::Up, DrdyIrq);

    let mut adc = Ad7768::new(ctrl_spi, data_spi, cs, rst, drdy);

    adc.reset().await.unwrap();

    match adc.check_revision().await {
        Ok(rev) => info!("AD7768 rev 0x{:02X}", rev),
        Err(Error::BadRevision(rev)) => info!("wrong revision 0x{:02X}", rev),
        Err(e) => panic!("AD7768 rev failed: {:?}", e),
    }

    let cfg = Ad7768Config::default();
    adc.configure(&cfg).await.unwrap();

    let odr = calc_odr_hz(25_000_000, MclkDiv::Div4, DecRate::X32);

    info!("running at {} kSPS", odr / 1000);

    for ch in [0u8, 1, 2, 3, 5, 6] {
        adc.set_offset(ch, Cal24::from_i32(0)).await.unwrap();
    }
    let mut frames = [AdcFrame::from_u32(0); 8];
    let mut n = 0u32;

    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    let eth_driver = unsafe {
        Ethernet::new(
        &mut PACKETS, //packet buffers
        p.ETH, //pins
        ETHIrqs,
        p.PA1,   // ref_clk
        p.PA7,   // crs_dv
        p.PC4,   // rxd0
        p.PC5,   // rxd1
        p.PB12,  // txd0
        p.PB13,  // txd1
        p.PB11,  // tx_en
        mac_addr,
        Lan8742::new(0),
        p.PA2,   // mdio
        p.PC1,   // mdc
        )
    };

    let config = Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 1, 1), 24),
        gateway: None, 
        dns_servers: heapless::Vec::new(),
    });

    let resources = RESOURCES.init(StackResources::new());

    let stack = &*make_static!(Stack::new(
        eth_driver,
        config,
        resources,
        1234567890 // random seed
    ));

    //spawn tasks 
    spawner.spawn(net_task(stack)).unwrap(); 

    // buffers for the UDP socket
    let mut rx_meta = [PacketMetadata::EMPTY; 4];
    let mut rx_buffer = [0u8; 1024];
    let mut tx_meta = [PacketMetadata::EMPTY; 4];
    let mut tx_buffer = [0u8; 1024];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    socket.bind(1234).unwrap();

    let remote = (Ipv4Address::new(192, 168, 1, 2), 1234);

    let mut seq: u32 = 0; // initiate sqeuence number before sending first packet

    loop {
        match adc.read_all_channels(10_000, &mut frames).await {
            Ok(()) => {
                n += 1;

                let packet = ImuPacket {
                    id: 0xAD77,
                    channel_count: 8,
                    sequence: seq,
                    channels: core::array::from_fn(|i| {
                        code_to_mv(frames[i].data, 2500) as f32
                    })
                };

                let bytes = unsafe {
                    core::slice::from_raw_parts(
                        &packet as *const ImuPacket as *const u8,
                        core::mem::size_of::<ImuPacket>(),
                    )
                };
co
                socket.send_to(bytes, remote).await.unwrap();
                seq += 1;

                if n % 10 == 0 {
                    for f in &frames {
                        if cfg.standby_mask & (1 << f.header.channel_id) != 0 {
                            continue;
                        }
                        if f.header.chip_error {
                            error!("CH{}: chip error", f.header.channel_id);
                            continue;
                        }

                        let mv = code_to_mv(f.data, 2500);
                        info!("CH{}: {} counts  {} mV", f.header.channel_id, f.data, mv);
                    }
                    info!("--- {} samples ---", n);
                }
            }
            Err(Error::Timeout) => warn!("DRDY timeout"),
            Err(Error::ChipError) => {
                error!("chip error — resetting");
                adc.reset().await.ok();
                adc.configure(&cfg).await.ok();
            }
            Err(e) => error!("error: {:?}", e),
        }
    }
}