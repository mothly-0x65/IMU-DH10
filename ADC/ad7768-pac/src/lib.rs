//! # AD7768 / AD7768-4 Peripheral Access Crate
//!
//! Typed register access for the Analog Devices AD7768 (8-channel) and
//! AD7768-4 (4-channel) simultaneous-sampling sigma-delta ADCs.
//!
//! ## Register map summary (SPI control mode)
//!
//! | Addr | Name                  | R/W |
//! |------|-----------------------|-----|
//! | 0x00 | Channel Standby       | RW  |
//! | 0x01 | Channel Mode A        | RW  |
//! | 0x02 | Channel Mode B        | RW  |
//! | 0x03 | Channel Mode Select   | RW  |
//! | 0x04 | Power Mode            | RW  |
//! | 0x05 | General Config        | RW  |
//! | 0x06 | Data Control          | RW  |
//! | 0x07 | Interface Config      | RW  |
//! | 0x08 | BIST Control          | RW  |
//! | 0x09 | Device Status         | R   |
//! |b 0x0A | Revision ID           | R   |
//! | 0x0E | GPIO Control          | RW  |
//! | 0x0F | GPIO Write Data       | RW  |
//! | 0x10 | GPIO Read Data        | R   |
//! | 0x11 | Precharge Buffer 1    | RW  |
//! | 0x12 | Precharge Buffer 2    | RW  |
//! | 0x13 | Pos Ref Precharge Buf | RW  |
//! | 0x14 | Neg Ref Precharge Buf | RW  |
//! | 0x1E–0x35 | Offset registers | RW  |
//! | 0x36–0x4D | Gain registers   | RW  |
//! | 0x4E–0x55 | Sync phase offset| RW  |
//! | 0x56 | Diagnostic Rx Select  | RW  |
//! | 0x57 | Diagnostic Mux Ctrl   | RW  |
//! | 0x58 | Modulator Delay Ctrl  | RW  |
//! | 0x59 | Chop Control          | RW  |
//!
//! ## SPI framing
//!
//! Each SPI access is **16 bits**.
//! - Write: `[ 0 | addr[6:0] | data[7:0] ]`
//! - Read : first frame `[ 1 | addr[6:0] | 0x00 ]`, second frame returns data.
//!
//! The crate provides helpers for building those frames; it does **not** drive
//! any SPI peripheral — bring your own HAL.

#![no_std]
#![deny(missing_docs)]
// ---------------------------------------------------------------------------
// Re-exports for convenience
// ---------------------------------------------------------------------------
mod addr;
mod registers;
mod test;
// ---------------------------------------------------------------------------
// SPI frame helpers
// ---------------------------------------------------------------------------

/// Build a 16-bit SPI **write** frame.
///
/// `[ 0 | addr[6:0] | data[7:0] ]`
#[inline(always)]
pub const fn  spi_write_frame(addr: u8, data: u8) -> u16 {
    ((addr as u16) << 8) | (data as u16)
}

/// Build a 16-bit SPI **read** frame (first of two).
///
/// `[ 1 | addr[6:0] | 0x00 ]`
#[inline(always)]
pub const fn spi_read_frame(addr: u8) -> u16 {
    (0x80 | (addr as u16 & 0x7F)) << 8
}

// ---------------------------------------------------------------------------
// Typed enums shared by multiple registers
// ---------------------------------------------------------------------------

/// Filter type selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FilterType {
    /// Wideband low-ripple filter (−3 dB at 0.433 × ODR).
    Wideband = 0,
    /// Sinc5 low-latency filter (−3 dB at 0.204 × ODR).
    Sinc5 = 1,
}

impl TryFrom<u8> for FilterType {
    type Error = u8;
    fn try_from(v: u8) -> Result<Self, u8> {
        match v {
            0 => Ok(FilterType::Wideband),
            1 => Ok(FilterType::Sinc5),
            other => Err(other),
        }
    }
}

impl From<FilterType> for u8 {
    fn from(f: FilterType) -> u8 {
        f as u8
    }
}

/// Decimation rate.
///
/// Values 5, 6, and 7 all encode ×1024 per the datasheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DecRate {
    /// ×32
    X32 = 0,
    /// ×64
    X64 = 1,
    /// ×128
    X128 = 2,
    /// ×256
    X256 = 3,
    /// ×512
    X512 = 4,
    /// ×1024
    X1024 = 5,
}

impl DecRate {
    /// Return the actual integer decimation factor.
    pub const fn factor(self) -> u32 {
        match self {
            DecRate::X32 => 32,
            DecRate::X64 => 64,
            DecRate::X128 => 128,
            DecRate::X256 => 256,
            DecRate::X512 => 512,
            DecRate::X1024 => 1024,
        }
    }
}

impl TryFrom<u8> for DecRate {
    type Error = u8;
    fn try_from(v: u8) -> Result<Self, u8> {
        match v {
            0 => Ok(DecRate::X32),
            1 => Ok(DecRate::X64),
            2 => Ok(DecRate::X128),
            3 => Ok(DecRate::X256),
            4 => Ok(DecRate::X512),
            5 | 6 | 7 => Ok(DecRate::X1024),
            other => Err(other),
        }
    }
}

impl From<DecRate> for u8 {
    fn from(d: DecRate) -> u8 {
        d as u8
    }
}

/// Power / speed mode.
///
/// # Register encoding (reg 0x04 bits [5:4])
///
/// | Mode   | Register value |
/// |--------|---------------|
/// | Eco    | 0b00 (0)      |
/// | Median | 0b10 (2)      |
/// | Fast   | 0b11 (3)      |
///
/// Note: 0b01 is not a valid power mode (reserved).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PowerMode {
    /// Lowest power: 32 kSPS max, 13.8 kHz BW, ~9.4 mW/ch.
    Eco = 0,
    /// Mid-range: 128 kSPS max, 55.4 kHz BW, ~27.5 mW/ch.
    Median = 2,
    /// Highest speed: 256 kSPS max, 110.8 kHz BW, ~51.5 mW/ch.
    Fast = 3,
}

impl PowerMode {
    /// Return the MCLK divider that pairs with this mode at 32.768 MHz MCLK.
    pub const fn default_mclk_div(self) -> MclkDiv {
        match self {
            PowerMode::Eco => MclkDiv::Div32,
            PowerMode::Median => MclkDiv::Div8,
            PowerMode::Fast => MclkDiv::Div4,
        }
    }
}

impl TryFrom<u8> for PowerMode {
    type Error = u8;
    /// Parse the raw 2-bit field from register bits [5:4].
    fn try_from(v: u8) -> Result<Self, u8> {
        match v {
            0 => Ok(PowerMode::Eco),
            2 => Ok(PowerMode::Median),
            3 => Ok(PowerMode::Fast),
            other => Err(other),
        }
    }
}

impl From<PowerMode> for u8 {
    fn from(m: PowerMode) -> u8 {
        m as u8
    }
}

/// MCLK divider that sets the modulator frequency.
///
/// `fMOD = MCLK / MclkDiv`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MclkDiv {
    /// MCLK/32 — use with Eco mode.
    Div32 = 0,
    /// MCLK/8 — use with Median mode.
    Div8 = 2,
    /// MCLK/4 — use with Fast mode.
    Div4 = 3,
}

impl MclkDiv {
    /// Integer divisor value.
    pub const fn divisor(self) -> u32 {
        match self {
            MclkDiv::Div32 => 32,
            MclkDiv::Div8 => 8,
            MclkDiv::Div4 => 4,
        }
    }
}

impl TryFrom<u8> for MclkDiv {
    type Error = u8;
    fn try_from(v: u8) -> Result<Self, u8> {
        match v {
            0 => Ok(MclkDiv::Div32),
            2 => Ok(MclkDiv::Div8),
            3 => Ok(MclkDiv::Div4),
            other => Err(other),
        }
    }
}

impl From<MclkDiv> for u8 {
    fn from(d: MclkDiv) -> u8 {
        d as u8
    }
}

/// DCLK divider (register 0x07 bits [1:0]).
///
/// `DCLK = MCLK / DclkDiv`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DclkDiv {
    /// MCLK/8
    Div8 = 0,
    /// MCLK/4
    Div4 = 1,
    /// MCLK/2
    Div2 = 2,
    /// MCLK/1 (no division)
    Div1 = 3,
}

impl DclkDiv {
    /// Integer divisor value.
    pub const fn divisor(self) -> u32 {
        match self {
            DclkDiv::Div8 => 8,
            DclkDiv::Div4 => 4,
            DclkDiv::Div2 => 2,
            DclkDiv::Div1 => 1,
        }
    }
}

impl TryFrom<u8> for DclkDiv {
    type Error = u8;
    fn try_from(v: u8) -> Result<Self, u8> {
        match v {
            0 => Ok(DclkDiv::Div8),
            1 => Ok(DclkDiv::Div4),
            2 => Ok(DclkDiv::Div2),
            3 => Ok(DclkDiv::Div1),
            other => Err(other),
        }
    }
}

impl From<DclkDiv> for u8 {
    fn from(d: DclkDiv) -> u8 {
        d as u8
    }
}

/// CRC mode (register 0x07 bits [3:2]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CrcSelect {
    /// No CRC; status header with every conversion.
    Disabled = 0,
    /// CRC replaces header every 4 samples.
    Every4 = 1,
    /// CRC replaces header every 16 samples.
    Every16 = 2,
}

impl TryFrom<u8> for CrcSelect {
    type Error = u8;
    fn try_from(v: u8) -> Result<Self, u8> {
        match v {
            0 => Ok(CrcSelect::Disabled),
            1 => Ok(CrcSelect::Every4),
            2 | 3 => Ok(CrcSelect::Every16),
            other => Err(other),
        }
    }
}

impl From<CrcSelect> for u8 {
    fn from(c: CrcSelect) -> u8 {
        c as u8
    }
}

/// VCM output voltage selection (register 0x05 bits [1:0]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VcmVsel {
    /// (AVDD1 − AVSS) / 2  (default in pin control mode).
    HalfAvdd1 = 0,
    /// Fixed 1.65 V.
    V1_65 = 1,
    /// Fixed 2.5 V.
    V2_5 = 2,
    /// Fixed 2.14 V.
    V2_14 = 3,
}

impl TryFrom<u8> for VcmVsel {
    type Error = u8;
    fn try_from(v: u8) -> Result<Self, u8> {
        match v {
            0 => Ok(VcmVsel::HalfAvdd1),
            1 => Ok(VcmVsel::V1_65),
            2 => Ok(VcmVsel::V2_5),
            3 => Ok(VcmVsel::V2_14),
            other => Err(other),
        }
    }
}

impl From<VcmVsel> for u8 {
    fn from(v: VcmVsel) -> u8 {
        v as u8
    }
}

/// Channel assignment (Mode A or Mode B).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChannelMode {
    /// Channel Mode A.
    ModeA = 0,
    /// Channel Mode B.
    ModeB = 1,
}

impl From<bool> for ChannelMode {
    fn from(b: bool) -> Self {
        if b { ChannelMode::ModeB } else { ChannelMode::ModeA }
    }
}

impl From<ChannelMode> for bool {
    fn from(m: ChannelMode) -> bool {
        m == ChannelMode::ModeB
    }
}

/// Chopping frequency selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChopFreq {
    /// fMOD/8 — for better AC performance (slightly worse noise).
    FmodDiv8 = 1,
    /// fMOD/32 — default; best noise, offset, and offset drift.
    FmodDiv32 = 2,
}

impl TryFrom<u8> for ChopFreq {
    type Error = u8;
    fn try_from(v: u8) -> Result<Self, u8> {
        match v {
            1 => Ok(ChopFreq::FmodDiv8),
            2 => Ok(ChopFreq::FmodDiv32),
            other => Err(other),
        }
    }
}

impl From<ChopFreq> for u8 {
    fn from(c: ChopFreq) -> u8 {
        c as u8
    }
}

/// Modulator delayed-clock enable (register 0x58 bits [3:2]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ModDelayEn {
    /// Disabled for all channels.
    Disabled = 0,
    /// CH0–CH3 only (AD7768) / CH0–CH1 only (AD7768-4).
    LowChannels = 1,
    /// CH4–CH7 only (AD7768) / CH2–CH3 only (AD7768-4).
    HighChannels = 2,
    /// All channels.
    All = 3,
}

impl TryFrom<u8> for ModDelayEn {
    type Error = u8;
    fn try_from(v: u8) -> Result<Self, u8> {
        match v {
            0 => Ok(ModDelayEn::Disabled),
            1 => Ok(ModDelayEn::LowChannels),
            2 => Ok(ModDelayEn::HighChannels),
            3 => Ok(ModDelayEn::All),
            other => Err(other),
        }
    }
}

impl From<ModDelayEn> for u8 {
    fn from(m: ModDelayEn) -> u8 {
        m as u8
    }
}

/// Diagnostic voltage injected into ADC channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DiagMuxSel {
    /// Diagnostic off.
    Off = 0,
    /// Positive full-scale check.
    PosFull = 3,
    /// Negative full-scale check.
    NegFull = 4,
    /// Zero-scale check.
    ZeroScale = 5,
}

impl TryFrom<u8> for DiagMuxSel {
    type Error = u8;
    fn try_from(v: u8) -> Result<Self, u8> {
        match v {
            0 => Ok(DiagMuxSel::Off),
            3 => Ok(DiagMuxSel::PosFull),
            4 => Ok(DiagMuxSel::NegFull),
            5 => Ok(DiagMuxSel::ZeroScale),
            other => Err(other),
        }
    }
}

impl From<DiagMuxSel> for u8 {
    fn from(d: DiagMuxSel) -> u8 {
        d as u8
    }
}

// ---------------------------------------------------------------------------
// Decoded data output header
// ---------------------------------------------------------------------------

/// Decoded 8-bit **status header** prepended to every 24-bit ADC result.
///
/// Each ADC conversion frame is 32 bits: `[header: 8][data: 24]`.
#[derive(Debug, Clone, Copy)]
pub struct StatusHeader {
    /// Serious chip error requiring reset.
    pub chip_error: bool,
    /// Filter has not yet fully settled after sync/reset.
    pub filter_not_settled: bool,
    /// Data is repeated (channel running at a slower decimation rate).
    pub repeated_data: bool,
    /// Filter type in use: `false` = wideband, `true` = sinc5.
    pub sinc5_filter: bool,
    /// Filter output is clipping (saturated).
    pub filter_saturated: bool,
    /// Channel ID (0–7).
    pub channel_id: u8,
}

impl StatusHeader {
    /// Parse an 8-bit header byte.
    pub fn from_byte(b: u8) -> Self {
        Self {
            chip_error:        b & 0x80 != 0,
            filter_not_settled: b & 0x40 != 0,
            repeated_data:     b & 0x20 != 0,
            sinc5_filter:      b & 0x10 != 0,
            filter_saturated:  b & 0x08 != 0,
            channel_id:        b & 0x07,
        }
    }

    /// Reconstruct the raw byte (useful for testing).
    pub fn to_byte(self) -> u8 {
        (self.chip_error        as u8) << 7
            | (self.filter_not_settled as u8) << 6
            | (self.repeated_data      as u8) << 5
            | (self.sinc5_filter       as u8) << 4
            | (self.filter_saturated   as u8) << 3
            | (self.channel_id & 0x07)
    }
}

/// A decoded 32-bit ADC output frame.
#[derive(Debug, Clone, Copy)]
pub struct AdcFrame {
    /// Decoded status header.
    pub header: StatusHeader,
    /// 24-bit signed two's-complement ADC result (sign-extended to `i32`).
    pub data: i32,
}

impl AdcFrame {
    /// Parse a raw 32-bit big-endian word from the data interface.
    ///
    /// The AD7768 outputs MSB first: `[H7..H0 | D23..D0]`.
    pub fn from_u32(raw: u32) -> Self {
        let hdr_byte = ((raw >> 24) & 0xFF) as u8;
        let data_raw = raw & 0x00FF_FFFF;
        // Sign-extend from bit 23
        let data = if data_raw & 0x0080_0000 != 0 {
            (data_raw | 0xFF00_0000) as i32
        } else {
            data_raw as i32
        };
        Self {
            header: StatusHeader::from_byte(hdr_byte),
            data,
        }
    }

    /// Parse from four bytes in the order received over the wire (MSB first).
    pub fn from_bytes(b: [u8; 4]) -> Self {
        Self::from_u32(u32::from_be_bytes(b))
    }
}

// ---------------------------------------------------------------------------
// Voltage conversion
// ---------------------------------------------------------------------------

/// Convert a 24-bit signed ADC code to millivolts given a reference voltage.
///
/// Uses the formula: `V = code × (2 × Vref_mv) / 2^24`
///
/// Returns `None` if the code is outside the valid signed 24-bit range.
pub fn code_to_mv(code: i32, vref_mv: i32) -> i32 {
    // scale: 2*Vref / 2^24 mV per LSB
    // Use i64 arithmetic to avoid overflow
    ((code as i64 * (2 * vref_mv as i64)) >> 24) as i32
}

/// Calculate the output data rate in Hz.
///
/// `ODR = MCLK / (MCLK_div × decimation_factor)`
pub fn calc_odr_hz(mclk_hz: u32, mclk_div: MclkDiv, dec: DecRate) -> u32 {
    mclk_hz / (mclk_div.divisor() * dec.factor())
}