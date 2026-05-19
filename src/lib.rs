//! # AD7768 Embassy-STM32 async driver
//!
//! Wraps the `ad7768-pac` register types in an async driver that uses
//! Embassy's SPI and GPIO traits.
//!
//! ## Wiring (SPI + one DRDY interrupt pin)
//!
//! ```text
//! STM32              AD7768
//! ─────────────────────────────────────────
//! SPI_SCK   ──────►  SCLK  (pin 17)
//! SPI_MOSI  ──────►  SDI   (pin 18)
//! SPI_MISO  ◄──────  SDO   (pin 19)
//! GPIO_CS   ──────►  CS    (pin 16)   (active-low, software-controlled)
//! GPIO_DRDY ◄──────  DRDY  (pin 29)  (falling-edge interrupt)
//! GPIO_RESET──────►  RESET (pin 30)  (active-low)
//!
//! All data lines:
//! GPIO      ◄──────  DOUT0..7 / DCLK     (captured via SPI or parallel bus)
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut adc = Ad7768::new(spi, cs, reset_pin, drdy_pin)?;
//! adc.reset().await?;
//! adc.configure(Ad7768Config {
//!     power_mode: PowerMode::Fast,
//!     filter:     FilterType::Wideband,
//!     dec_rate:   DecRate::X32,
//!     dclk_div:   DclkDiv::Div4,
//!     enable_crc: true,
//! }).await?;
//!
//! // Wait for DRDY and read one sample from channel 0
//! let frame = adc.read_frame().await?;
//! defmt::info!("CH{}: {} counts", frame.header.channel_id, frame.data);
//! ```
