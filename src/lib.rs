use embassy_stm32::{
    exti::ExtiInput,
    gpio::{ Level, Output, Pin, },
    spi::Spi,
    time::Hertz,
};

use embassy_stm32::i2c::Master;
use embassy_stm32::mode::Async;
use embassy_stm32::spi::mode::Slave;
use embassy_time::{Duration, Timer};
use embedded_hal_async::spi::{ErrorType, SpiBus};
use ad7768_pac::*;
use ad7768_pac::registers::registers::{ChannelModeReg, ChannelStandby, DataControl, DeviceStatus, InterfaceConfig, PowerModeReg, RevisionId};
use crate::Error::BadRevision;

#[derive(Debug, defmt::Format)]

pub enum Error {
    /// Underlying SPI Peripheral Error
    Spi(embassy_stm32::spi::Error),
    /// Device returned an unexpected revision ID.
    BadRevision(u8),
    /// A register read-back check failed (expected, got)
    Verify(u8, u8),
    /// DRDY timeout - device did not assert data-ready within the deadline
    Timeout,
    /// Chip Error bit set in the status header
    ChipError,
}


impl<E> From<E> for Error {
    fn from(e: E) -> Self { Error::Spi(e) }
}


// ---------------------------------------------------------------------------
// Configuration struct
// ---------------------------------------------------------------------------

/// High level configuration for the AD7768
pub struct Ad7768Config {
    /// Power mode / speed mode (Eco / Median / Fast).
    pub power_mode: PowerMode,
    /// Digital filter type (Wideband / Sinc5)
    pub filter: FilterType,
    /// Decimation rate (affects ODR and noise floor)
    pub dec_rate: DecRate,
    /// DCLK divider relative to MCLK.
    pub dclk_div: DclkDiv,
    /// Enable 4-sample CRC on the data interface
    pub enable_crc: bool,
    /// Channels to put into standby (bitmask, bit N = channel N).
    ///
    /// `0x00` means all channels active (default).
    /// For us channel 3 and 7 are not active so we set `0x88`
    pub standby_mask: u8,
}

impl Default for Ad7768Config {
    fn default() -> Self {
        Self {
            power_mode: PowerMode::Fast,
            filter: FilterType::Wideband,
            dec_rate: DecRate::X32,
            dclk_div: DclkDiv::Div4,
            enable_crc: true,
            standby_mask: 0x88,
        }
    }
}



// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

// The concrete Spi type from embassy-stm32 is:
//
//   Spi<'d, M: PeriMode, CM: CommunicationMode>
//
// where M is embassy_stm32::mode::Async or Blocking,
// and CM is embassy_stm32::spi::Master or Slave.
//

/// Async driver for the AD7768 / AD7768-4.
///
/// Two SPI peripherals are required:
///
/// - `ctrl_spi` — master mode, drives SCLK/SDI and reads SDO for register
///   access. CS is software-controlled via the `cs` pin.
/// - `data_spi` — slave mode, receives DOUT0 clocked by DCLK from the AD7768.
///   Wire AD7768 DCLK → STM32 SPI SCK input, AD7768 DOUT0 → STM32 SPI MISO.
///   The AD7768 owns the clock here so the STM32 must be slave.
pub struct Ad7768<'d>
{
    ctrl_spi:   Spi<'d, Async, Master>,
    data_spi:   Spi<'d, Async, Slave>,
    cs:         Output<'d>,
    rst:        Output<'d>,
    drdy:       ExtiInput<'d, Async>,
}


// ---------------------------------------------------------------------------
// The SPI protocol is 16-bit frames, Mode 0 (CPOL=0, CPHA=0).
// CS must be manually toggled around every 16-bit frame because the AD7768
// uses an "off-frame" protocol (response arrives on the next CS assertion).
// ---------------------------------------------------------------------------

impl<'d> Ad7768<'d>
{
    /// Construct the driver.  Does **not** reset the device; call
    /// [`reset`](Self::reset) first.
    pub fn new(
        ctrl_spi: Spi<'d, Async, Master>,
        data_spi: Spi<'d, Async, Slave>,
        cs: Output<'d>,
        rst: Output<'d>,
        drdy: ExtiInput<'d, Async>,
    ) -> Self {
        Self { ctrl_spi, data_spi, cs, rst, drdy }
    }


    // -----------------------------------------------------------------------
    // Low-level SPI helpers
    // -----------------------------------------------------------------------

    /// Send a single 16-bit frame to the control spi, return the 16-bit response.
    ///
    /// The AD7768 uses an *off-frame* protocol: the response to command N
    /// arrives during command N+1.  Callers that need read data must issue a
    /// second (dummy) transfer.
    async fn transfer16(&mut self, tx: u16) -> Result<u16, Error>
    {
        use embedded_hal_async::spi::SpiBus;
        let tx_bytes = tx.to_be_bytes();
        let mut rx_bytes = [0u8; 2];

        self.cs.set_low();
        self.ctrl_spi.transfer(&mut rx_bytes, &tx_bytes).await.map_err(Error::Spi)?;
        self.cs.set_high();

        Ok(u16::from_be_bytes(rx_bytes))
    }

    /// Write a register (one 16-bit frame)
    pub async fn write_reg<E>(&mut self, addr: u8, data: u8) -> Result<(), Error>
    {
        self.transfer16(spi_write_frame(addr, data)).await?;
        Ok(())
    }

    /// Read a register.
    ///
    /// Requires **two** SPI frames (off-frame protocol):
    /// 1. Send the read command — device ignores data, queues response.
    /// 2. Send a dummy frame — device clocks out the response.
    pub async fn read_reg(&mut self, addr: u8) -> Result<u8, Error>
    {
        // Frame 1: read request
        self.transfer16(spi_read_frame(addr)).await?;
        // Frame 2: dummy write, collect response in low byte
        let resp = self.transfer16(0x0000).await?;

        Ok((resp & 0xFF) as u8)
    }

    /// Read-modify-wrie one register
    pub async fn reg_modify<F>(
        &mut self,
        addr: u8,
        f: F
    ) -> Result<(), Error>
    where
        F: FnOnce(u8) -> u8,
    {
        let old = self.read_reg(addr).await?;
        let new = f(old);
        if new != old {
            self.write_reg(addr, new).await?;
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Reset and sync
    // -----------------------------------------------------------------------

    /// Hard reset via ADC_RESET pin, then wait for device to start-up
    pub async fn reset(&mut self) -> Result<(), Error> {
        self.rst.set_low();
        Timer::after(Duration::from_micros(2)).await;
        self.rst.set_high();
        // account for worst case starput 25Mhz / fast / dec*32
        Timer::after(Duration::from_millis(5)).await;
        Ok(())
    }

    /// Software reset via the SPI data control register.
    pub async fn soft_reset(&mut self) -> Result<(), Error> {
        self.write_reg(addr::addr::DATA_CONTROL, u8::from(DataControl::reset_byte1())).await?;
        self.write_reg(addr::addr::DATA_CONTROL, u8::from(DataControl::reset_byte2())).await?;
        Timer::after(Duration::from_micros(5)).await;
        Ok(())
    }


    /// Issue the SPI_SYNC pulse so the digital filters restart with current config
    ///
    /// Must be called after each configuration change.
    pub async fn sync(&mut self) -> Result<(), Error> {
        let low = u8::from(DataControl::default().with_sync_low());
        let high = u8::from(DataControl::default().with_sync_high());
        self.write_reg(DataControl::ADDR, low).await?;
        self.write_reg(DataControl::ADDR, high).await?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Configuration
    // -----------------------------------------------------------------------

    /// Apply a full [`Ad7768Config`] to the device, then issue sync.
    pub async fn configure(&mut self, cfg: &Ad7768Config) -> Result<(), Error> {
        let pm = PowerModeReg::from_power_mode(cfg.power_mode);
        self.write_reg(PowerModeReg::ADDR, u8::from(pm)).await?;

        let mode_a = ChannelModeReg::new(cfg.filter, cfg.dec_rate);
        self.write_reg(ChannelModeReg::ADDR_A, u8::from(mode_a)).await?;

        let crc = if cfg.enable_crc { CrcSelect::Every4 } else { CrcSelect::Disabled };
        let iface = InterfaceConfig::default()
            .set_dclk_div(cfg.dclk_div)
            .set_crc_select(crc);
        self.write_reg(InterfaceConfig::ADDR, u8::from(iface)).await?;

        self.write_reg(ChannelStandby::ADDR, u8::from(cfg.standby_mask)).await?;

        self.sync().await
    }

    /// Verify the revision ID register reads 0x06 (Rev A silicon).
    pub async fn check_revision(&mut self) -> Result<u8, Error> {
        let rev = self.read_reg(RevisionId::ADDR).await?;
        if rev != RevisionId::REV_A {
            Err(BadRevision(rev))
        } else {
            Ok(rev)
        }
    }

    /// Read the device status register.
    pub async fn status(&mut self) -> Result<DeviceStatus, Error> {
        Ok(DeviceStatus(self.read_reg(DeviceStatus::ADDR).await?))
    }
}
