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
//! |b 0x0A | Revision ID          | R   |
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
// ---------------------------------------------------------------------------
// Register structs
// ---------------------------------------------------------------------------

/// All typed register structs.
pub mod registers {
    use crate::addr::addr::*;
    use crate::*;

    // -----------------------------------------------------------------------
    // 0x00 – Channel Standby
    // -----------------------------------------------------------------------

    /// **Channel Standby** register (0x00).
    ///
    /// Each bit disables the corresponding channel. The disabled channel's
    /// position in the data stream is held with the header and data output
    /// as all zeros.
    ///
    /// Reset value: `0x00` (all channels enabled).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct ChannelStandby(pub u8);

    impl ChannelStandby {
        /// Register address.
        pub const ADDR: u8 = CHANNEL_STANDBY;
        /// Reset value.
        pub const RESET: u8 = 0x00;

        /// Return `true` if the given channel is in standby.
        ///
        /// `channel` must be 0–7 (AD7768) or 0–3 (AD7768-4).
        #[inline]
        pub fn is_standby(self, channel: u8) -> bool {
            debug_assert!(channel < 8);
            self.0 & (1 << channel) != 0
        }

        /// Put channel into standby.
        #[inline]
        pub fn set_standby(mut self, channel: u8) -> Self {
            debug_assert!(channel < 8);
            self.0 |= 1 << channel;
            self
        }

        /// Bring channel out of standby (enable it).
        #[inline]
        pub fn clear_standby(mut self, channel: u8) -> Self {
            debug_assert!(channel < 8);
            self.0 &= !(1 << channel);
            self
        }

        /// Put all channels into standby.
        #[inline]
        pub fn all_standby(mut self) -> Self {
            self.0 = 0xFF;
            self
        }

        /// Enable all channels (clear all standby bits).
        #[inline]
        pub fn all_enabled(mut self) -> Self {
            self.0 = 0x00;
            self
        }
    }

    impl From<u8> for ChannelStandby {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<ChannelStandby> for u8 {
        fn from(r: ChannelStandby) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x01 / 0x02 – Channel Mode A / B
    // -----------------------------------------------------------------------

    /// **Channel Mode A** (0x01) or **Channel Mode B** (0x02) register.
    ///
    /// Sets the filter type and decimation rate for one of the two channel
    /// mode groups.
    ///
    /// Reset value: `0x0D` (sinc5, ×1024).
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct ChannelModeReg(pub u8);

    impl ChannelModeReg {
        /// Address of Channel Mode A.
        pub const ADDR_A: u8 = CHANNEL_MODE_A;
        /// Address of Channel Mode B.
        pub const ADDR_B: u8 = CHANNEL_MODE_B;
        /// Reset value for both registers.
        pub const RESET: u8 = 0x0D; // sinc5 + dec×1024

        /// Construct with explicit filter and decimation rate.
        pub fn new(filter: FilterType, dec: DecRate) -> Self {
            Self(((filter as u8) << 3) | (dec as u8 & 0x7))
        }

        /// Read the filter type field.
        pub fn filter_type(self) -> Result<FilterType, u8> {
            FilterType::try_from((self.0 >> 3) & 0x1)
        }

        /// Read the decimation rate field.
        pub fn dec_rate(self) -> Result<DecRate, u8> {
            DecRate::try_from(self.0 & 0x7)
        }

        /// Set filter type, preserving decimation rate bits.
        pub fn set_filter_type(mut self, ft: FilterType) -> Self {
            self.0 = (self.0 & !(1 << 3)) | ((ft as u8) << 3);
            self
        }

        /// Set decimation rate, preserving filter type bit.
        pub fn set_dec_rate(mut self, dr: DecRate) -> Self {
            self.0 = (self.0 & !0x7) | (dr as u8 & 0x7);
            self
        }
    }

    impl Default for ChannelModeReg {
        fn default() -> Self {
            Self(Self::RESET)
        }
    }
    impl From<u8> for ChannelModeReg {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<ChannelModeReg> for u8 {
        fn from(r: ChannelModeReg) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x03 – Channel Mode Select
    // -----------------------------------------------------------------------

    /// **Channel Mode Select** register (0x03).
    ///
    /// Assigns each of the 8 ADC channels to either Channel Mode A or
    /// Channel Mode B. Bit N = 0 → Mode A, 1 → Mode B.
    ///
    /// Reset value: `0x00` (all channels in Mode A).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct ChannelModeSelect(pub u8);

    impl ChannelModeSelect {
        /// Register address.
        pub const ADDR: u8 = CHANNEL_MODE_SELECT;
        /// Reset value.
        pub const RESET: u8 = 0x00;

        /// Get the mode assignment of a channel (0–7).
        #[inline]
        pub fn channel_mode(self, ch: u8) -> ChannelMode {
            debug_assert!(ch < 8);
            ChannelMode::from(self.0 & (1 << ch) != 0)
        }

        /// Assign a channel to Mode A or Mode B.
        #[inline]
        pub fn set_channel_mode(mut self, ch: u8, mode: ChannelMode) -> Self {
            debug_assert!(ch < 8);
            match mode {
                ChannelMode::ModeA => self.0 &= !(1 << ch),
                ChannelMode::ModeB => self.0 |= 1 << ch,
            }
            self
        }

        /// Assign all channels to Mode A.
        #[inline]
        pub fn all_mode_a(mut self) -> Self {
            self.0 = 0x00;
            self
        }

        /// Assign all channels to Mode B.
        #[inline]
        pub fn all_mode_b(mut self) -> Self {
            self.0 = 0xFF;
            self
        }
    }

    impl From<u8> for ChannelModeSelect {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<ChannelModeSelect> for u8 {
        fn from(r: ChannelModeSelect) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x04 – Power Mode
    // -----------------------------------------------------------------------

    /// **Power Mode** register (0x04).
    ///
    /// Controls sleep mode, power/speed mode, LVDS clock enable, and the
    /// MCLK divider ratio.
    ///
    /// Reset value: `0x00` (normal operation, Eco mode, MCLK/32).
    ///
    /// # Important
    ///
    /// The `POWER_MODE` bits and `MCLK_DIV` bits are **independent** in SPI
    /// control mode.  You must set both to achieve the correct modulator
    /// frequency.  Use [`PowerMode::default_mclk_div`] for the datasheet-
    /// recommended pairing.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct PowerModeReg(pub u8);

    impl PowerModeReg {
        /// Register address.
        pub const ADDR: u8 = POWER_MODE;
        /// Reset value.
        pub const RESET: u8 = 0x00;

        /// Construct with the recommended pairing for a given power mode.
        ///
        /// Sets `POWER_MODE` and `MCLK_DIV` to the datasheet-recommended
        /// values for `mode`, clears sleep and LVDS.
        pub fn from_power_mode(mode: PowerMode) -> Self {
            Self::default()
                .set_power_mode(mode)
                .set_mclk_div(mode.default_mclk_div())
        }

        /// Read the sleep mode bit.
        #[inline]
        pub fn sleep_mode(self) -> bool {
            self.0 & 0x80 != 0
        }

        /// Enable or disable sleep mode.
        #[inline]
        pub fn set_sleep(mut self, sleep: bool) -> Self {
            if sleep {
                self.0 |= 0x80
            } else {
                self.0 &= !0x80
            }
            self
        }

        /// Read the power mode field (bits [5:4]).
        pub fn power_mode(self) -> Result<PowerMode, u8> {
            PowerMode::try_from((self.0 >> 4) & 0x3)
        }

        /// Set the power mode field.
        pub fn set_power_mode(mut self, pm: PowerMode) -> Self {
            self.0 = (self.0 & !(0x3 << 4)) | ((pm as u8) << 4);
            self
        }

        /// Read the LVDS enable bit.
        #[inline]
        pub fn lvds_enable(self) -> bool {
            self.0 & 0x08 != 0
        }

        /// Enable or disable the LVDS clock input.
        ///
        /// Only effective when `CLK_SEL` pin is high.
        #[inline]
        pub fn set_lvds_enable(mut self, en: bool) -> Self {
            if en {
                self.0 |= 0x08
            } else {
                self.0 &= !0x08
            }
            self
        }

        /// Read the MCLK divider field (bits [1:0]).
        pub fn mclk_div(self) -> Result<MclkDiv, u8> {
            MclkDiv::try_from(self.0 & 0x3)
        }

        /// Set the MCLK divider field.
        pub fn set_mclk_div(mut self, div: MclkDiv) -> Self {
            self.0 = (self.0 & !0x3) | (div as u8 & 0x3);
            self
        }
    }

    impl From<u8> for PowerModeReg {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<PowerModeReg> for u8 {
        fn from(r: PowerModeReg) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x05 – General Configuration
    // -----------------------------------------------------------------------

    /// **General Configuration** register (0x05).
    ///
    /// Controls SYNC_OUT retime, VCM buffer, and VCM voltage selection.
    ///
    /// Reset value: `0x08` (reserved bit 3 must stay 1 on AD7768-4).
    #[derive(Debug, Clone, Copy)]
    pub struct GeneralConfig(pub u8);

    impl GeneralConfig {
        /// Register address.
        pub const ADDR: u8 = GENERAL_CONFIG;
        /// Reset value.
        pub const RESET: u8 = 0x08;

        /// Read the SYNC_OUT retime enable bit (bit 5).
        #[inline]
        pub fn retime_enable(self) -> bool {
            self.0 & 0x20 != 0
        }

        /// Set or clear the SYNC_OUT retime enable bit.
        #[inline]
        pub fn set_retime_enable(mut self, en: bool) -> Self {
            if en {
                self.0 |= 0x20
            } else {
                self.0 &= !0x20
            }
            self
        }

        /// Read VCM power-down bit (bit 4). `true` = powered down.
        #[inline]
        pub fn vcm_powered_down(self) -> bool {
            self.0 & 0x10 != 0
        }

        /// Power down (`true`) or enable (`false`) the VCM buffer.
        #[inline]
        pub fn set_vcm_power_down(mut self, pd: bool) -> Self {
            if pd {
                self.0 |= 0x10
            } else {
                self.0 &= !0x10
            }
            self
        }

        /// Read the VCM voltage selection (bits [1:0]).
        pub fn vcm_vsel(self) -> Result<VcmVsel, u8> {
            VcmVsel::try_from(self.0 & 0x3)
        }

        /// Set the VCM voltage selection.
        pub fn set_vcm_vsel(mut self, sel: VcmVsel) -> Self {
            self.0 = (self.0 & !0x3) | (sel as u8 & 0x3);
            self
        }
    }

    impl Default for GeneralConfig {
        fn default() -> Self {
            Self(Self::RESET)
        }
    }
    impl From<u8> for GeneralConfig {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<GeneralConfig> for u8 {
        fn from(r: GeneralConfig) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x06 – Data Control
    // -----------------------------------------------------------------------

    /// **Data Control** register (0x06).
    ///
    /// Controls SPI sync, one-shot mode, and the soft reset sequence.
    ///
    /// Reset value: `0x80` (SPI_SYNC bit starts high).
    ///
    /// # SPI Sync
    ///
    /// To trigger a sync, write this register **twice**:
    /// 1. Write `SPI_SYNC = 0` (use [`DataControl::with_sync_low`]).
    /// 2. Write `SPI_SYNC = 1` (use [`DataControl::with_sync_high`]).
    ///
    /// # Soft Reset
    ///
    /// Write `SPI_RESET = 0x03` (first byte), then `SPI_RESET = 0x02`
    /// (second byte).  Use [`DataControl::reset_byte1`] and
    /// [`DataControl::reset_byte2`].
    #[derive(Debug, Clone, Copy)]
    pub struct DataControl(pub u8);

    impl DataControl {
        /// Register address.
        pub const ADDR: u8 = DATA_CONTROL;
        /// Reset value.
        pub const RESET: u8 = 0x80;

        /// Byte 1 of the soft reset sequence (write 0x03 to SPI_RESET).
        pub const fn reset_byte1() -> Self {
            Self(0x03)
        }
        /// Byte 2 of the soft reset sequence (write 0x02 to SPI_RESET).
        pub const fn reset_byte2() -> Self {
            Self(0x02)
        }

        /// Return a copy with the SPI_SYNC bit set to 0 (step 1 of sync).
        #[inline]
        pub fn with_sync_low(mut self) -> Self {
            self.0 &= !0x80;
            self
        }

        /// Return a copy with the SPI_SYNC bit set to 1 (step 2 of sync).
        #[inline]
        pub fn with_sync_high(mut self) -> Self {
            self.0 |= 0x80;
            self
        }

        /// Whether the SPI_SYNC bit is high.
        #[inline]
        pub fn sync_high(self) -> bool {
            self.0 & 0x80 != 0
        }

        /// Whether one-shot mode is enabled (bit 4).
        #[inline]
        pub fn one_shot_enabled(self) -> bool {
            self.0 & 0x10 != 0
        }

        /// Enable or disable one-shot conversion mode.
        #[inline]
        pub fn set_one_shot(mut self, en: bool) -> Self {
            if en {
                self.0 |= 0x10
            } else {
                self.0 &= !0x10
            }
            self
        }
    }

    impl Default for DataControl {
        fn default() -> Self {
            Self(Self::RESET)
        }
    }
    impl From<u8> for DataControl {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<DataControl> for u8 {
        fn from(r: DataControl) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x07 – Interface Configuration
    // -----------------------------------------------------------------------

    /// **Interface Configuration** register (0x07).
    ///
    /// Controls the CRC mode and the DCLK divider.
    ///
    /// Reset value: `0x00` (no CRC, DCLK = MCLK/8).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct InterfaceConfig(pub u8);

    impl InterfaceConfig {
        /// Register address.
        pub const ADDR: u8 = INTERFACE_CONFIG;
        /// Reset value.
        pub const RESET: u8 = 0x00;

        /// Read the CRC mode (bits [3:2]).
        pub fn crc_select(self) -> Result<CrcSelect, u8> {
            CrcSelect::try_from((self.0 >> 2) & 0x3)
        }

        /// Set the CRC mode.
        pub fn set_crc_select(mut self, crc: CrcSelect) -> Self {
            self.0 = (self.0 & !(0x3 << 2)) | ((crc as u8) << 2);
            self
        }

        /// Enable 4-sample CRC (shorthand used by the Linux driver).
        pub fn with_crc_4sample(mut self) -> Self {
            self.0 = (self.0 & !(0x3 << 2)) | (0x01 << 2);
            self
        }

        /// Read the DCLK divider field (bits [1:0]).
        pub fn dclk_div(self) -> Result<DclkDiv, u8> {
            DclkDiv::try_from(self.0 & 0x3)
        }

        /// Set the DCLK divider.
        pub fn set_dclk_div(mut self, div: DclkDiv) -> Self {
            self.0 = (self.0 & !0x3) | (div as u8 & 0x3);
            self
        }
    }

    impl From<u8> for InterfaceConfig {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<InterfaceConfig> for u8 {
        fn from(r: InterfaceConfig) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x08 – BIST Control
    // -----------------------------------------------------------------------

    /// **BIST Control** register (0x08).
    ///
    /// Starts the RAM built-in self test (BIST).  Normal ADC conversions are
    /// disrupted while the test runs; a sync pulse is required afterwards.
    ///
    /// Reset value: `0x00`.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct BistControl(pub u8);

    impl BistControl {
        /// Register address.
        pub const ADDR: u8 = BIST_CONTROL;
        /// Value to write to start the BIST.
        pub const START: Self = Self(0x01);
        /// Value to write to stop the BIST.
        pub const STOP: Self = Self(0x00);

        /// Whether the BIST start bit is set.
        #[inline]
        pub fn started(self) -> bool {
            self.0 & 0x01 != 0
        }
    }

    impl From<u8> for BistControl {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<BistControl> for u8 {
        fn from(r: BistControl) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x09 – Device Status (read-only)
    // -----------------------------------------------------------------------

    /// **Device Status** register (0x09) — read-only.
    ///
    /// Reset value: `0x00`.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct DeviceStatus(pub u8);

    impl DeviceStatus {
        /// Register address.
        pub const ADDR: u8 = DEVICE_STATUS;

        /// A serious chip error has occurred (bit 3).
        ///
        /// Set when: power-up CRC fails, background memory XOR check fails,
        /// or no external clock detected.  A reset is required to clear.
        #[inline]
        pub fn chip_error(self) -> bool {
            self.0 & 0x08 != 0
        }

        /// External MCLK was **not** detected (bit 2).
        ///
        /// When set, `chip_error` is also set and all conversion data is
        /// output as zeros.
        #[inline]
        pub fn no_clock_error(self) -> bool {
            self.0 & 0x04 != 0
        }

        /// The most recent RAM BIST passed (bit 1).
        ///
        /// `false` means the BIST has not been run or has failed.
        #[inline]
        pub fn bist_passed(self) -> bool {
            self.0 & 0x02 != 0
        }

        /// The RAM BIST is currently running (bit 0).
        #[inline]
        pub fn bist_running(self) -> bool {
            self.0 & 0x01 != 0
        }
    }

    impl From<u8> for DeviceStatus {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<DeviceStatus> for u8 {
        fn from(r: DeviceStatus) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x0A – Revision ID (read-only)
    // -----------------------------------------------------------------------

    /// **Revision ID** register (0x0A) — read-only.
    ///
    /// Reset value (Rev A silicon): `0x06`.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct RevisionId(pub u8);

    impl RevisionId {
        /// Register address.
        pub const ADDR: u8 = REVISION_ID;
        /// Expected revision for Rev A silicon.
        pub const REV_A: u8 = 0x06;

        /// Raw revision byte.
        #[inline]
        pub fn revision(self) -> u8 {
            self.0
        }
    }

    impl From<u8> for RevisionId {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<RevisionId> for u8 {
        fn from(r: RevisionId) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x0E – GPIO Control
    // -----------------------------------------------------------------------

    /// **GPIO Control** register (0x0E).
    ///
    /// Bit 7 is a universal GPIO enable. Bits [4:0] set each GPIO as input (0)
    /// or output (1).
    ///
    /// Reset value: `0x00` (GPIO disabled, all inputs).
    ///
    /// GPIO pin mapping (SPI mode):
    /// - GPIO0 → MODE0/GPIO0 (pin 12)
    /// - GPIO1 → MODE1/GPIO1 (pin 13)
    /// - GPIO2 → MODE2/GPIO2 (pin 14)
    /// - GPIO3 → MODE3/GPIO3 (pin 15)
    /// - GPIO4 → FILTER/GPIO4 (pin 11)
    #[derive(Debug, Clone, Copy, Default)]
    pub struct GpioControl(pub u8);

    impl GpioControl {
        /// Register address.
        pub const ADDR: u8 = GPIO_CONTROL;
        /// Reset value.
        pub const RESET: u8 = 0x00;

        /// Whether the universal GPIO enable bit is set (bit 7).
        #[inline]
        pub fn ugpio_enabled(self) -> bool {
            self.0 & 0x80 != 0
        }

        /// Set or clear the universal GPIO enable.
        #[inline]
        pub fn set_ugpio_enable(mut self, en: bool) -> Self {
            if en {
                self.0 |= 0x80
            } else {
                self.0 &= !0x80
            }
            self
        }

        /// Whether GPIO `n` (0–4) is configured as an output.
        ///
        /// Returns `false` (input) if `n > 4`.
        #[inline]
        pub fn is_output(self, n: u8) -> bool {
            if n > 4 {
                return false;
            }
            self.0 & (1 << n) != 0
        }

        /// Configure GPIO `n` (0–4) as output (`true`) or input (`false`).
        #[inline]
        pub fn set_direction(mut self, n: u8, output: bool) -> Self {
            debug_assert!(n <= 4);
            if output {
                self.0 |= 1 << n
            } else {
                self.0 &= !(1 << n)
            }
            self
        }
    }

    impl From<u8> for GpioControl {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<GpioControl> for u8 {
        fn from(r: GpioControl) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x0F – GPIO Write Data
    // -----------------------------------------------------------------------

    /// **GPIO Write Data** register (0x0F).
    ///
    /// Sets output levels for GPIOs configured as outputs in [`GpioControl`].
    /// Bits [4:0] correspond to GPIO4..GPIO0.
    ///
    /// Reset value: `0x00`.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct GpioWrite(pub u8);

    impl GpioWrite {
        /// Register address.
        pub const ADDR: u8 = GPIO_WRITE;

        /// Read the logic level that will be driven on GPIO `n` (0–4).
        #[inline]
        pub fn get(self, n: u8) -> bool {
            debug_assert!(n <= 4);
            self.0 & (1 << n) != 0
        }

        /// Set the output level for GPIO `n`.
        #[inline]
        pub fn set(mut self, n: u8, high: bool) -> Self {
            debug_assert!(n <= 4);
            if high {
                self.0 |= 1 << n
            } else {
                self.0 &= !(1 << n)
            }
            self
        }
    }

    impl From<u8> for GpioWrite {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<GpioWrite> for u8 {
        fn from(r: GpioWrite) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x10 – GPIO Read Data (read-only)
    // -----------------------------------------------------------------------

    /// **GPIO Read Data** register (0x10) — read-only.
    ///
    /// Reflects the logic level at each GPIO pin when configured as an input.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct GpioRead(pub u8);

    impl GpioRead {
        /// Register address.
        pub const ADDR: u8 = GPIO_READ;

        /// Read the logic level of GPIO `n` (0–4).
        #[inline]
        pub fn get(self, n: u8) -> bool {
            debug_assert!(n <= 4);
            self.0 & (1 << n) != 0
        }
    }

    impl From<u8> for GpioRead {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<GpioRead> for u8 {
        fn from(r: GpioRead) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x11 / 0x12 – Analog Input Precharge Buffers
    // -----------------------------------------------------------------------

    /// **Analog Input Precharge Buffer** register.
    ///
    /// Used for both register 0x11 (CH0–CH3) and register 0x12 (CH4–CH7 on
    /// the AD7768; CH2–CH3 on the AD7768-4).
    ///
    /// **Important**: to clear (disable) a buffer bit you must write the
    /// *inverse* of the desired bit pattern.  This struct stores the register
    /// value directly — encode it with [`PrechargeBuf::raw`] when writing.
    ///
    /// Reset value: `0xFF` (all buffers enabled).
    ///
    /// Bit layout (per register):
    /// - bit 0: CH_low_POS (e.g. CH0_PREBUF_POS_EN for reg 0x11)
    /// - bit 1: CH_low_NEG
    /// - bit 2: CH_low+1_POS
    /// - ...
    #[derive(Debug, Clone, Copy)]
    pub struct PrechargeBuf(pub u8);

    impl PrechargeBuf {
        /// Address of buffer register 1 (CH0–CH3).
        pub const ADDR1: u8 = PRECHARGE_BUF1;
        /// Address of buffer register 2 (CH4–CH7 / CH2–CH3).
        pub const ADDR2: u8 = PRECHARGE_BUF2;
        /// Reset value.
        pub const RESET: u8 = 0xFF;

        /// All precharge buffers enabled.
        pub const ALL_ON: Self = Self(0xFF);
        /// All precharge buffers disabled.
        pub const ALL_OFF: Self = Self(0x00);

        /// Is the positive precharge buffer for the n-th channel in this
        /// register's group enabled? (`n` = 0 or 1 for the low/high channel
        /// within the pair covered by this register).
        ///
        /// - reg 0x11: n=0 → CH0_POS (bit 0), n=1 → CH1_POS (bit 2), etc.
        #[inline]
        pub fn pos_enabled(self, pair_index: u8) -> bool {
            debug_assert!(pair_index < 4);
            self.0 & (1 << (pair_index * 2)) != 0
        }

        /// Is the negative precharge buffer for the n-th pair enabled?
        #[inline]
        pub fn neg_enabled(self, pair_index: u8) -> bool {
            debug_assert!(pair_index < 4);
            self.0 & (1 << (pair_index * 2 + 1)) != 0
        }

        /// Enable or disable the positive buffer for a pair.
        #[inline]
        pub fn set_pos(mut self, pair_index: u8, en: bool) -> Self {
            debug_assert!(pair_index < 4);
            let bit = 1 << (pair_index * 2);
            if en {
                self.0 |= bit
            } else {
                self.0 &= !bit
            }
            self
        }

        /// Enable or disable the negative buffer for a pair.
        #[inline]
        pub fn set_neg(mut self, pair_index: u8, en: bool) -> Self {
            debug_assert!(pair_index < 4);
            let bit = 1 << (pair_index * 2 + 1);
            if en {
                self.0 |= bit
            } else {
                self.0 &= !bit
            }
            self
        }
    }

    impl Default for PrechargeBuf {
        fn default() -> Self {
            Self(Self::RESET)
        }
    }
    impl From<u8> for PrechargeBuf {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<PrechargeBuf> for u8 {
        fn from(r: PrechargeBuf) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x13 / 0x14 – Reference Precharge Buffers
    // -----------------------------------------------------------------------

    /// **Reference Precharge Buffer** register.
    ///
    /// Used for register 0x13 (positive REF) and 0x14 (negative REF).
    /// One bit per channel; `1` = buffer on.
    ///
    /// Reset value: `0x00` (all reference buffers disabled).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct RefPrechargeBuf(pub u8);

    impl RefPrechargeBuf {
        /// Address of the positive reference precharge buffer register.
        pub const ADDR_POS: u8 = REF_POS_PRECHARGE;
        /// Address of the negative reference precharge buffer register.
        pub const ADDR_NEG: u8 = REF_NEG_PRECHARGE;
        /// Reset value.
        pub const RESET: u8 = 0x00;

        /// Is the reference precharge buffer for `channel` (0–7) enabled?
        #[inline]
        pub fn enabled(self, channel: u8) -> bool {
            debug_assert!(channel < 8);
            self.0 & (1 << channel) != 0
        }

        /// Enable or disable the reference precharge buffer for `channel`.
        #[inline]
        pub fn set_enabled(mut self, channel: u8, en: bool) -> Self {
            debug_assert!(channel < 8);
            if en {
                self.0 |= 1 << channel
            } else {
                self.0 &= !(1 << channel)
            }
            self
        }

        /// Enable all reference precharge buffers.
        #[inline]
        pub fn all_on(mut self) -> Self {
            self.0 = 0xFF;
            self
        }

        /// Disable all reference precharge buffers.
        #[inline]
        pub fn all_off(mut self) -> Self {
            self.0 = 0x00;
            self
        }
    }

    impl From<u8> for RefPrechargeBuf {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<RefPrechargeBuf> for u8 {
        fn from(r: RefPrechargeBuf) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // Offset / Gain registers (24-bit, three bytes each)
    // -----------------------------------------------------------------------

    /// A 24-bit signed two's-complement calibration value, stored as three
    /// consecutive 8-bit registers (MSB, MID, LSB).
    ///
    /// Used for both offset and gain calibration registers.
    ///
    /// # Offset interpretation
    ///
    /// With nominal gain (0x555555), each LSB of offset adjustment shifts
    /// the digital output by −4/3 LSBs.
    ///
    /// # Gain interpretation
    ///
    /// Factory-programmed around 0x555555.  Overwriting survives until reset.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Cal24(pub u32); // only lower 24 bits are used

    impl Cal24 {
        /// Construct from a signed 24-bit value (sign-extended from bit 23).
        pub fn from_i32(v: i32) -> Self {
            Self((v as u32) & 0xFF_FFFF)
        }

        /// Interpret stored value as a signed 24-bit integer.
        pub fn as_i32(self) -> i32 {
            let v = self.0 & 0xFF_FFFF;
            if v & 0x80_0000 != 0 {
                // sign extend
                (v | 0xFF00_0000) as i32
            } else {
                v as i32
            }
        }

        /// MSB byte (written to the _MSB register address).
        #[inline]
        pub fn msb(self) -> u8 {
            ((self.0 >> 16) & 0xFF) as u8
        }
        /// MID byte.
        #[inline]
        pub fn mid(self) -> u8 {
            ((self.0 >> 8) & 0xFF) as u8
        }
        /// LSB byte (written to the _LSB register address).
        #[inline]
        pub fn lsb(self) -> u8 {
            (self.0 & 0xFF) as u8
        }

        /// Reconstruct from the three bytes read back from the device.
        pub fn from_bytes(msb: u8, mid: u8, lsb: u8) -> Self {
            Self(((msb as u32) << 16) | ((mid as u32) << 8) | (lsb as u32))
        }
    }

    // -----------------------------------------------------------------------
    // 0x56 – Diagnostic Rx Select
    // -----------------------------------------------------------------------

    /// **Diagnostic Rx Select** register (0x56).
    ///
    /// Each bit enables the internal diagnostic voltage injection for the
    /// corresponding channel.  Analog input precharge buffers must be on
    /// for the selected channels.
    ///
    /// Reset value: `0x00`.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct DiagnosticRx(pub u8);

    impl DiagnosticRx {
        /// Register address.
        pub const ADDR: u8 = DIAGNOSTIC_RX;

        /// Is the diagnostic receive enabled for `channel` (0–7)?
        #[inline]
        pub fn enabled(self, channel: u8) -> bool {
            debug_assert!(channel < 8);
            self.0 & (1 << channel) != 0
        }

        /// Enable or disable diagnostic receive for `channel`.
        #[inline]
        pub fn set_enabled(mut self, channel: u8, en: bool) -> Self {
            debug_assert!(channel < 8);
            if en {
                self.0 |= 1 << channel
            } else {
                self.0 &= !(1 << channel)
            }
            self
        }

        /// Enable all channels for diagnostic receive.
        #[inline]
        pub fn all_on(mut self) -> Self {
            self.0 = 0xFF;
            self
        }
    }

    impl From<u8> for DiagnosticRx {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<DiagnosticRx> for u8 {
        fn from(r: DiagnosticRx) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x57 – Diagnostic Mux Control
    // -----------------------------------------------------------------------

    /// **Diagnostic Mux Control** register (0x57).
    ///
    /// Selects the voltage to inject into channels assigned to Mode A (bits
    /// [2:0]) and Mode B (bits [6:4]).
    ///
    /// Reset value: `0x00` (both muxes off).
    #[derive(Debug, Clone, Copy, Default)]
    pub struct DiagnosticMux(pub u8);

    impl DiagnosticMux {
        /// Register address.
        pub const ADDR: u8 = DIAGNOSTIC_MUX;

        /// Read the Group A mux selection (bits [2:0]).
        pub fn group_a(self) -> Result<DiagMuxSel, u8> {
            DiagMuxSel::try_from(self.0 & 0x7)
        }

        /// Read the Group B mux selection (bits [6:4]).
        pub fn group_b(self) -> Result<DiagMuxSel, u8> {
            DiagMuxSel::try_from((self.0 >> 4) & 0x7)
        }

        /// Set Group A mux selection.
        pub fn set_group_a(mut self, sel: DiagMuxSel) -> Self {
            self.0 = (self.0 & !0x7) | (sel as u8 & 0x7);
            self
        }

        /// Set Group B mux selection.
        pub fn set_group_b(mut self, sel: DiagMuxSel) -> Self {
            self.0 = (self.0 & !(0x7 << 4)) | ((sel as u8 & 0x7) << 4);
            self
        }
    }

    impl From<u8> for DiagnosticMux {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<DiagnosticMux> for u8 {
        fn from(r: DiagnosticMux) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x58 – Modulator Delay Control
    // -----------------------------------------------------------------------

    /// **Modulator Delay Control** register (0x58).
    ///
    /// **Caution**: bits [1:0] are reserved and must always be `0b10`.  The
    /// default constructor sets this correctly.
    ///
    /// Reset value: `0x02`.
    #[derive(Debug, Clone, Copy)]
    pub struct ModDelayCtrl(pub u8);

    impl ModDelayCtrl {
        /// Register address.
        pub const ADDR: u8 = MOD_DELAY_CTRL;
        /// Reset value (reserved bits [1:0] = 0b10).
        pub const RESET: u8 = 0x02;

        /// Read the delayed-clock enable field (bits [3:2]).
        pub fn delay_enable(self) -> Result<ModDelayEn, u8> {
            ModDelayEn::try_from((self.0 >> 2) & 0x3)
        }

        /// Set the delayed-clock enable, preserving the reserved bits.
        pub fn set_delay_enable(mut self, en: ModDelayEn) -> Self {
            self.0 = (self.0 & !(0x3 << 2)) | ((en as u8) << 2);
            // always keep reserved bits [1:0] = 0b10
            self.0 = (self.0 & !0x3) | 0x02;
            self
        }
    }

    impl Default for ModDelayCtrl {
        fn default() -> Self {
            Self(Self::RESET)
        }
    }
    impl From<u8> for ModDelayCtrl {
        fn from(v: u8) -> Self {
            // Enforce the reserved bits
            Self((v & !0x3) | 0x02)
        }
    }
    impl From<ModDelayCtrl> for u8 {
        fn from(r: ModDelayCtrl) -> u8 {
            r.0
        }
    }

    // -----------------------------------------------------------------------
    // 0x59 – Chop Control
    // -----------------------------------------------------------------------

    /// **Chop Control** register (0x59).
    ///
    /// Sets the chopping frequency for Group A (bits [3:2]) and Group B
    /// (bits [1:0]).
    ///
    /// Reset value: `0x0A` (both groups at fMOD/32).
    #[derive(Debug, Clone, Copy)]
    pub struct ChopControl(pub u8);

    impl ChopControl {
        /// Register address.
        pub const ADDR: u8 = CHOP_CONTROL;
        /// Reset value.
        pub const RESET: u8 = 0x0A;

        /// Read Group A chop frequency (bits [3:2]).
        pub fn group_a(self) -> Result<ChopFreq, u8> {
            ChopFreq::try_from((self.0 >> 2) & 0x3)
        }

        /// Read Group B chop frequency (bits [1:0]).
        pub fn group_b(self) -> Result<ChopFreq, u8> {
            ChopFreq::try_from(self.0 & 0x3)
        }

        /// Set Group A chop frequency.
        pub fn set_group_a(mut self, f: ChopFreq) -> Self {
            self.0 = (self.0 & !(0x3 << 2)) | ((f as u8) << 2);
            self
        }

        /// Set Group B chop frequency.
        pub fn set_group_b(mut self, f: ChopFreq) -> Self {
            self.0 = (self.0 & !0x3) | (f as u8 & 0x3);
            self
        }
    }

    impl Default for ChopControl {
        fn default() -> Self {
            Self(Self::RESET)
        }
    }
    impl From<u8> for ChopControl {
        fn from(v: u8) -> Self {
            Self(v)
        }
    }
    impl From<ChopControl> for u8 {
        fn from(r: ChopControl) -> u8 {
            r.0
        }
    }
}
