// ---------------------------------------------------------------------------
// Register addresses
// ---------------------------------------------------------------------------

/// Register address constants.
pub mod addr {
    /// Channel standby (0x00)
    pub const CHANNEL_STANDBY: u8 = 0x00;
    /// Channel Mode A (0x01)
    pub const CHANNEL_MODE_A: u8 = 0x01;
    /// Channel Mode B (0x02)
    pub const CHANNEL_MODE_B: u8 = 0x02;
    /// Channel Mode Select (0x03)
    pub const CHANNEL_MODE_SELECT: u8 = 0x03;
    /// Power Mode (0x04)
    pub const POWER_MODE: u8 = 0x04;
    /// General Configuration (0x05)
    pub const GENERAL_CONFIG: u8 = 0x05;
    /// Data Control (0x06)
    pub const DATA_CONTROL: u8 = 0x06;
    /// Interface Configuration (0x07)
    pub const INTERFACE_CONFIG: u8 = 0x07;
    /// BIST Control (0x08)
    pub const BIST_CONTROL: u8 = 0x08;
    /// Device Status (0x09) — read only
    pub const DEVICE_STATUS: u8 = 0x09;
    /// Revision ID (0x0A) — read only
    pub const REVISION_ID: u8 = 0x0A;
    /// GPIO Control (0x0E)
    pub const GPIO_CONTROL: u8 = 0x0E;
    /// GPIO Write Data (0x0F)
    pub const GPIO_WRITE: u8 = 0x0F;
    /// GPIO Read Data (0x10) — read only
    pub const GPIO_READ: u8 = 0x10;
    /// Analog Input Precharge Buffer, CH0–CH3 (0x11)
    pub const PRECHARGE_BUF1: u8 = 0x11;
    /// Analog Input Precharge Buffer, CH4–CH7 (0x12)
    pub const PRECHARGE_BUF2: u8 = 0x12;
    /// Positive Reference Precharge Buffer (0x13)
    pub const REF_POS_PRECHARGE: u8 = 0x13;
    /// Negative Reference Precharge Buffer (0x14)
    pub const REF_NEG_PRECHARGE: u8 = 0x14;

    // Offset registers: CH0 = 0x1E..0x20, CH1 = 0x21..0x23, ...
    // AD7768 uses all 8; AD7768-4 uses CH0..CH3 at the same addresses.
    /// Channel 0 offset MSB
    pub const CH0_OFFSET_MSB: u8 = 0x1E;
    /// Channel 0 offset MID
    pub const CH0_OFFSET_MID: u8 = 0x1F;
    /// Channel 0 offset LSB
    pub const CH0_OFFSET_LSB: u8 = 0x20;
    /// Channel 1 offset MSB
    pub const CH1_OFFSET_MSB: u8 = 0x21;
    /// Channel 1 offset MID
    pub const CH1_OFFSET_MID: u8 = 0x22;
    /// Channel 1 offset LSB
    pub const CH1_OFFSET_LSB: u8 = 0x23;
    /// Channel 2 offset MSB (AD7768-4: 0x2A)
    pub const CH2_OFFSET_MSB: u8 = 0x24; // AD7768; AD7768-4 uses 0x2A
    /// Channel 2 offset MID
    pub const CH2_OFFSET_MID: u8 = 0x25;
    /// Channel 2 offset LSB
    pub const CH2_OFFSET_LSB: u8 = 0x26;
    /// Channel 3 offset MSB (AD7768-4: 0x2D)
    pub const CH3_OFFSET_MSB: u8 = 0x27;
    /// Channel 3 offset MID
    pub const CH3_OFFSET_MID: u8 = 0x28;
    /// Channel 3 offset LSB
    pub const CH3_OFFSET_LSB: u8 = 0x29;
    /// Channel 4 offset MSB (AD7768 only)
    pub const CH4_OFFSET_MSB: u8 = 0x2A;
    /// Channel 4 offset MID
    pub const CH4_OFFSET_MID: u8 = 0x2B;
    /// Channel 4 offset LSB
    pub const CH4_OFFSET_LSB: u8 = 0x2C;
    /// Channel 5 offset MSB (AD7768 only)
    pub const CH5_OFFSET_MSB: u8 = 0x2D;
    /// Channel 5 offset MID
    pub const CH5_OFFSET_MID: u8 = 0x2E;
    /// Channel 5 offset LSB
    pub const CH5_OFFSET_LSB: u8 = 0x2F;
    /// Channel 6 offset MSB (AD7768 only)
    pub const CH6_OFFSET_MSB: u8 = 0x30;
    /// Channel 6 offset MID
    pub const CH6_OFFSET_MID: u8 = 0x31;
    /// Channel 6 offset LSB
    pub const CH6_OFFSET_LSB: u8 = 0x32;
    /// Channel 7 offset MSB (AD7768 only)
    pub const CH7_OFFSET_MSB: u8 = 0x33;
    /// Channel 7 offset MID
    pub const CH7_OFFSET_MID: u8 = 0x34;
    /// Channel 7 offset LSB
    pub const CH7_OFFSET_LSB: u8 = 0x35;

    // Gain registers: same layout as offset
    /// Channel 0 gain MSB
    pub const CH0_GAIN_MSB: u8 = 0x36;
    /// Channel 0 gain MID
    pub const CH0_GAIN_MID: u8 = 0x37;
    /// Channel 0 gain LSB
    pub const CH0_GAIN_LSB: u8 = 0x38;
    /// Channel 1 gain MSB
    pub const CH1_GAIN_MSB: u8 = 0x39;
    /// Channel 1 gain MID
    pub const CH1_GAIN_MID: u8 = 0x3A;
    /// Channel 1 gain LSB
    pub const CH1_GAIN_LSB: u8 = 0x3B;
    /// Channel 2 gain MSB
    pub const CH2_GAIN_MSB: u8 = 0x3C;
    /// Channel 2 gain MID
    pub const CH2_GAIN_MID: u8 = 0x3D;
    /// Channel 2 gain LSB
    pub const CH2_GAIN_LSB: u8 = 0x3E;
    /// Channel 3 gain MSB
    pub const CH3_GAIN_MSB: u8 = 0x3F;
    /// Channel 3 gain MID
    pub const CH3_GAIN_MID: u8 = 0x40;
    /// Channel 3 gain LSB
    pub const CH3_GAIN_LSB: u8 = 0x41;
    /// Channel 4 gain MSB (AD7768 only)
    pub const CH4_GAIN_MSB: u8 = 0x42;
    /// Channel 4 gain MID
    pub const CH4_GAIN_MID: u8 = 0x43;
    /// Channel 4 gain LSB
    pub const CH4_GAIN_LSB: u8 = 0x44;
    /// Channel 5 gain MSB (AD7768 only)
    pub const CH5_GAIN_MSB: u8 = 0x45;
    /// Channel 5 gain MID
    pub const CH5_GAIN_MID: u8 = 0x46;
    /// Channel 5 gain LSB
    pub const CH5_GAIN_LSB: u8 = 0x47;
    /// Channel 6 gain MSB (AD7768 only)
    pub const CH6_GAIN_MSB: u8 = 0x48;
    /// Channel 6 gain MID
    pub const CH6_GAIN_MID: u8 = 0x49;
    /// Channel 6 gain LSB
    pub const CH6_GAIN_LSB: u8 = 0x4A;
    /// Channel 7 gain MSB (AD7768 only)
    pub const CH7_GAIN_MSB: u8 = 0x4B;
    /// Channel 7 gain MID
    pub const CH7_GAIN_MID: u8 = 0x4C;
    /// Channel 7 gain LSB
    pub const CH7_GAIN_LSB: u8 = 0x4D;

    // Sync phase offset registers
    /// Channel 0 sync phase offset
    pub const CH0_SYNC_OFFSET: u8 = 0x4E;
    /// Channel 1 sync phase offset
    pub const CH1_SYNC_OFFSET: u8 = 0x4F;
    /// Channel 2 sync phase offset (AD7768: 0x50, AD7768-4: 0x52)
    pub const CH2_SYNC_OFFSET: u8 = 0x50;
    /// Channel 3 sync phase offset (AD7768: 0x51, AD7768-4: 0x53)
    pub const CH3_SYNC_OFFSET: u8 = 0x51;
    /// Channel 4 sync phase offset (AD7768 only)
    pub const CH4_SYNC_OFFSET: u8 = 0x52;
    /// Channel 5 sync phase offset (AD7768 only)
    pub const CH5_SYNC_OFFSET: u8 = 0x53;
    /// Channel 6 sync phase offset (AD7768 only)
    pub const CH6_SYNC_OFFSET: u8 = 0x54;
    /// Channel 7 sync phase offset (AD7768 only)
    pub const CH7_SYNC_OFFSET: u8 = 0x55;

    /// Diagnostic Rx Select (0x56)
    pub const DIAGNOSTIC_RX: u8 = 0x56;
    /// Diagnostic Mux Control (0x57)
    pub const DIAGNOSTIC_MUX: u8 = 0x57;
    /// Modulator Delay Control (0x58)
    pub const MOD_DELAY_CTRL: u8 = 0x58;
    /// Chop Control (0x59)
    pub const CHOP_CONTROL: u8 = 0x59;
}
