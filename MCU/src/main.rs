#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;
use defmt::{error, info, warn};
use embassy_executor::Spawner;
use embassy_stm32::eth::{Ethernet, GenericPhy, PacketQueue, Sma};
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
use embassy_net::{Ipv4Address, Ipv4Cidr, StaticConfigV4, Stack, StackResources, Runner};
use embassy_stm32::peripherals::{ETH, ETH_SMA};
use lan8742::Lan8742;
use embassy_net::udp::{UdpSocket, PacketMetadata};
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
    sync: u16,
    sequence: u32,
    accel_x: f32,
    accel_y: f32,
    accel_z: f32,
    gyro_x: f32,
    gyro_y: f32,
    gyro_z: f32,
}

bind_interrupts!(struct Irqs {
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


#[unsafe(link_section = ".eth_buffers")]
static PACKETS: StaticCell<PacketQueue<4, 4>> = StaticCell::new();
static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

static STACK: StaticCell<Stack> = StaticCell::new();


// this task runs the network stack forever in the background
#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Ethernet<'static, ETH, Lan8742<Sma<'static, ETH_SMA>>>>) {
    runner.run().await
}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::info!("main task start");
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
        p.PC11,     // dummy: we don't use a miso
        p.PA4,      // dummy: drdy is set as EXTInput below
        p.DMA1_CH2,
        p.DMA1_CH3,
        SPI3Irqs,
        Config::default(),
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
    info!("MPU initialized");
    let p = embassy_stm32::init(Default::default());

    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    let sma_driver = Sma::new(p.ETH_SMA, p.PA2, p.PC1);
    let eth_phy = Lan8742::new(0x00, sma_driver);

    let packet_queue = unsafe{PACKETS.init(PacketQueue::new())};

    let eth_driver = unsafe {
        // We use an external PHY Lan8742
        Ethernet::new_with_phy(
            packet_queue,
            p.ETH,
            Irqs,
            p.PA1,  // ref_clk
            p.PA7,  // crs_dv
            p.PC4,  // rxd0
            p.PC5,  // rxd1
            #[cfg(feature = "nucleo")]
            p.PG13, // txd0 (nucleo)
            #[cfg(not(feature = "nucleo"))]
            p.PB12, // txd0 (custom board)
            #[cfg(feature = "nucleo")]
            p.PB13, // txd1 (nucleo)
            #[cfg(not(feature = "nucleo"))]
            p.PB13, // txd1 (custom board)
            //#[cfg(feature = "nucleo")]
            //p.PG11, // tx_en (nucleo)
            #[cfg(not(feature = "nucleo"))]
            p.PB11, // tx_en (custom board)
            mac_addr,
            eth_phy,
        )
    };

    let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 1, 1), 24),
        gateway: None,
        dns_servers: heapless::Vec::new(),
    });

    let resources = RESOURCES.init(StackResources::new());

    let (stack, runner) = embassy_net::new(
        eth_driver,
        config,
        resources,
        1234567890,
    );

    spawner.spawn(net_task(runner).expect("REASON"));

    info!("network stack started, sending IMU packets...");

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

    let mut led = Output::new(p.PB0, Level::Low, Speed::Low);

    let remote = (Ipv4Address::new(192, 168, 1, 2), 1234);

    let mut seq: u32 = 0;

    loop {
        match adc.read_all_channels(10_000, &mut frames).await {
            Ok(()) => {
                n += 1;

                let packet = ImuPacket {
                    sync: 0xAD77,
                    sequence: seq,
                    accel_x: code_to_mv(frames[0].data, 2500) as f32,
                    accel_y: code_to_mv(frames[1].data, 2500) as f32,
                    accel_z: code_to_mv(frames[2].data, 2500) as f32,
                    gyro_x: code_to_mv(frames[4].data, 2500) as f32,
                    gyro_y: code_to_mv(frames[5].data, 2500) as f32,
                    gyro_z: code_to_mv(frames[6].data, 2500) as f32,
                };

        let bytes = unsafe {
            core::slice::from_raw_parts(
                &packet as *const ImuPacket as *const u8,
                core::mem::size_of::<ImuPacket>(),
            )
        };

        match socket.send_to(bytes, remote).await {
            Ok(_) => led.toggle(),
            Err(_) => {} // silent fail
        }
        defmt::info!("sent packet seq={}", seq);
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