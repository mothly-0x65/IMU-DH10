#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::eth::{Ethernet, PacketQueue};
use embassy_stm32::{bind_interrupts, eth};
use embassy_stm32::peripherals::ETH;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_net::{Config, Ipv4Address, Ipv4Cidr, StaticConfigV4, Stack, StackResources};
use embassy_net::udp::PacketMetadata;
use embassy_net::udp::UdpSocket;
use lan8742::Lan8742;
use static_cell::StaticCell;
use defmt_rtt as _;
use panic_probe as _;

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

#[link_section = ".eth_buffers"]
static mut PACKETS: PacketQueue<4, 4> = PacketQueue::new();

static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

static STACK: StaticCell<Stack<Ethernet<'static, ETH, Lan8742>>> = StaticCell::new();

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<Ethernet<'static, ETH, Lan8742>>) {
    stack.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::info!("main task start");
    mpu::init();
    defmt::info!("MPU initialized");
    let p = embassy_stm32::init(Default::default());

    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    let eth_driver = unsafe {
        Ethernet::new(
            &mut PACKETS,
            p.ETH,
            Irqs,
            p.PA1,  // ref_clk
            p.PA2,  // mdio
            p.PC1,  // mdc
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
            #[cfg(feature = "nucleo")]
            p.PG11, // tx_en (nucleo)
            #[cfg(not(feature = "nucleo"))]
            p.PB11, // tx_en (custom board)
            Lan8742::new(0),
            mac_addr,
        )
    };

    let config = Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 1, 1), 24),
        gateway: None,
        dns_servers: heapless::Vec::new(),
    });

    let resources = RESOURCES.init(StackResources::new());

    let stack = STACK.init(Stack::new(
        eth_driver,
        config,
        resources,
        1234567890,
    ));

    spawner.spawn(net_task(stack)).unwrap();

    defmt::info!("network stack started, sending IMU packets...");

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
        let packet = ImuPacket {
            sync: 0xAD77,
            sequence: seq,
            accel_x: 0.0,
            accel_y: 0.0,
            accel_z: 0.0,
            gyro_x: 0.0,
            gyro_y: 0.0,
            gyro_z: 0.0,
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

        embassy_time::Timer::after_millis(10).await;
    }
}