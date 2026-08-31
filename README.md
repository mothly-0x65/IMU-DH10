# IMU-DH10

Firmware for the **DH10 IMU board**, a custom inertial measurement unit built
around an STM32H753ZI (Cortex-M7) that samples its analog sensors through an
**Analog Devices AD7768** 8-channel 24-bit sigma-delta ADC and streams the
readings out over 100 Mbit Ethernet as UDP packets.

Everything is written in embedded async Rust on [Embassy](https://embassy.dev/):
no RTOS and no heap, just cooperative async tasks, DMA-driven SPI, and `defmt`
logging over RTT.

## How it works

1. The MCU configures the AD7768 over its SPI **control port** (SPI4 in master
   mode: register writes, reset, revision check, channel standby, offset
   calibration).
2. Conversion data comes in on the AD7768's **data port**. The ADC drives the
   bit clock, so SPI3 runs in *slave* mode with the AD7768's DCLK as clock and
   DOUT0 as data. The DRDY line (EXTI on PA15) paces each frame read.
3. Every cycle reads all active channels: accelerometer X/Y/Z on channels 0–2
   and gyroscope X/Y/Z on channels 4–6 (channels 3 and 7 are held in standby).
4. ADC codes are converted to millivolts and packed into a fixed 30-byte
   `ImuPacket`, which `embassy-net` sends as UDP from the board's static IP
   `192.168.1.1` to the host at `192.168.1.2:1234`, through a **LAN8742**
   Ethernet PHY on RMII.
5. On the STM32H7 the Ethernet DMA and the Cortex-M7 data cache can go out of
   sync, so the MPU marks D2 SRAM (`0x3000_0000`) non-cacheable and the
   Ethernet buffers are linked into that region (see `MCU/memory.x` and
   `MCU/src/mpu.rs`).

### UDP packet layout (`repr(C, packed)`, little-endian, 30 bytes)

| Field      | Type  | Notes                       |
| ---------- | ----- | --------------------------- |
| `sync`     | `u16` | Constant `0xAD77`           |
| `sequence` | `u32` | Incrementing packet counter |
| `accel_x/y/z` | `f32 × 3` | Channel 0–2 readings, mV |
| `gyro_x/y/z`  | `f32 × 3` | Channel 4–6 readings, mV |

## Workspace layout

| Crate | Path | What it is |
| ----- | ---- | ---------- |
| `imu-embassy-project` | `MCU/` | The firmware binary: clock/pin setup, Ethernet + `embassy-net` stack, acquisition loop |
| `ad7768` | `ADC/` | High-level async driver for the AD7768 (configuration, DRDY-paced `read_all_channels`, error handling) |
| `ad7768-pac` | `ADC/ad7768-pac/` | Register-level access layer for the AD7768: register maps, bitfields, and helpers such as `calc_odr_hz` and `code_to_mv` |
| `lan8742` | `PHY/lan8742/` | Minimal LAN8742 PHY driver implementing `embassy-stm32`'s `Phy` trait (reset, autonegotiation, link polling) |

## Branches

| Branch | State |
| ------ | ----- |
| `main` | Skeleton: only the `ad7768-pac` register layer |
| `phy` | Ethernet bring-up: LAN8742 driver and first UDP transmissions |
| `adc` | **Most complete.** Full AD7768 driver merged with the PHY work; reads the ADC and streams IMU packets over UDP |
| `adc_task` | Experiment on top of `adc`: moving acquisition into a separate Embassy task (work in progress) |

## Building and flashing

The firmware lives on the `adc` branch. You need a recent Rust toolchain, the
Cortex-M7 target, and [probe-rs](https://probe.rs/) with a debug probe attached
(ST-Link on a Nucleo works):

```sh
rustup target add thumbv7em-none-eabihf
cargo install probe-rs-tools

git switch adc
cargo run -p imu-embassy-project        # builds, flashes, and tails defmt logs
```

`.cargo/config.toml` sets the target and the runner
(`probe-rs run --chip STM32H753ZITx`), so a plain `cargo run` flashes the board
and streams the `defmt` log output over RTT.

The `nucleo` feature of the MCU crate retargets the RMII TX pins so the same
firmware can be brought up on a Nucleo-H753ZI instead of the DH10 board.

## Pin map (custom board)

| Function | Pins |
| -------- | ---- |
| AD7768 control (SPI4, master) | PE2 SCK, PE6 MOSI → SDI, PE5 MISO ← SDO, PE4 CS, PD0 RST |
| AD7768 data (SPI3, slave) | PC10 ← DCLK, PC12 ← DOUT0, PA15 ← DRDY (EXTI15) |
| Ethernet RMII | PA1 REF_CLK, PA7 CRS_DV, PC4 RXD0, PC5 RXD1, PB12 TXD0, PB13 TXD1, PB11 TX_EN |
| PHY management (SMI) | PA2 MDIO, PC1 MDC |
| Status LED | PB0 (toggles per packet sent) |
