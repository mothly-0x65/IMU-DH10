#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::eth::{Ethernet, PacketQueue};
use embassy_stm32::{bind_interrupts, eth, peripherals};
use embassy_net::{Config, Ipv4Address, Ipv4Cidr, StaticConfigV4, Stack, StackResources};
use lan8742::Lan8742;
use embassy_net::udp::{UdpSocket, PacketMetadata};

mod mpu;

#[repr(C, packed)]
struct ImuPacket {
    id: u16, // 0xAD77 - identifier for the imu packet
    channel_count: u8, // always 8
    sequence: u32, // increments every packet, lets PLC detect drops
    channels: [f32; 8], // the 8 ADC channel values
}

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
});

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

    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    let eth_driver = unsafe {
        Ethernet::new(
        &mut PACKETS, //packet buffers
        p.ETH, //pins
        Irqs,
        p.PA1,   // ref_clk
        p.PA2,   // mdio
        p.PC1,   // mdc
        p.PA7,   // crs_dv
        p.PC4,   // rxd0
        p.PC5,   // rxd1
        p.PB11,  // tx_en
        p.PB12,  // txd0
        p.PB13,  // txd1
        Lan8742::new(0),
        mac_addr
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
        &mut rx_buffer.
        &mut tx_meta,
        &mut tx_buffer,
    );

    socket.bind(1234).unwrap();

    let remote = (Ipv4Address::new(192, 168, 1, 2), 1234);

    let mut seq: u32 = 0; // initiate sqeuence number before sending first packet

    loop {
        let packet = ImuPacket {
            id: 0xAD77,
            channel_count: 8,
            sequence: seq,
            channels: [ch0, ch1, ch2, ch3, ch4, ch5, ch6, ch7] // adc readings
        };

        let bytes = unsafe {
            core::slice::from_raw_parts(
                &packet as *const ImuPacket as *const u8,
                core::mem::size_of::<ImuPacket>(),
            )
        };

        socket.send_to(bytes, remote).await.unwrap();
        seq += 1;

        embassy_time::Timer::after_millis(10).await;
    }
}