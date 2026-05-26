// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::registers::registers::*;

    #[test]
    fn spi_frame_write() {
        // addr=0x04, data=0x30 → 0x0430
        assert_eq!(spi_write_frame(0x04, 0x30), 0x0430);
    }

    #[test]
    fn spi_frame_read() {
        // addr=0x09 → 0x8900
        assert_eq!(spi_read_frame(0x09), 0x8900);
    }

    #[test]
    fn channel_standby_roundtrip() {
        let r = ChannelStandby::default()
            .set_standby(3)
            .set_standby(7);
        assert!(r.is_standby(3));
        assert!(r.is_standby(7));
        assert!(!r.is_standby(0));
        let r2 = r.clear_standby(3);
        assert!(!r2.is_standby(3));
        assert!(r2.is_standby(7));
    }

    #[test]
    fn channel_mode_reg_encode_decode() {
        let r = ChannelModeReg::new(FilterType::Wideband, DecRate::X64);
        assert_eq!(r.filter_type(), Ok(FilterType::Wideband));
        assert_eq!(r.dec_rate(), Ok(DecRate::X64));
        // raw value: filter=0 << 3 | dec=1 = 0x01
        assert_eq!(u8::from(r), 0x01);
    }

    #[test]
    fn channel_mode_reg_reset_is_sinc5_x1024() {
        let r = ChannelModeReg::default();
        assert_eq!(r.filter_type(), Ok(FilterType::Sinc5));
        assert_eq!(r.dec_rate(), Ok(DecRate::X1024));
        // raw value: filter = << 3 | dec=5 = 0x0D
        assert_eq!(u8::from(r), 0x0D);
        assert_eq!(ChannelModeReg::from(0x0D), r);
    }

    #[test]
    fn power_mode_reg_eco_pairing() {
        let r = PowerModeReg::from_power_mode(PowerMode::Eco);
        assert_eq!(r.power_mode(), Ok(PowerMode::Eco));
        assert_eq!(r.mclk_div(), Ok(MclkDiv::Div32));
        assert!(!r.sleep_mode());
        assert!(!r.lvds_enable());
    }

    #[test]
    fn power_mode_reg_fast_pairing() {
        let r = PowerModeReg::from_power_mode(PowerMode::Fast);
        assert_eq!(r.power_mode(), Ok(PowerMode::Fast));
        assert_eq!(r.mclk_div(), Ok(MclkDiv::Div4));
        // raw: power=0b11 << 4 | mclk=0b11 = 0x33
        assert_eq!(u8::from(r), 0x33);
    }

    #[test]
    fn power_mode_reg_median_pairing() {
        let mut r = PowerModeReg::from_power_mode(PowerMode::Median);
        assert_eq!(r.power_mode(), Ok(PowerMode::Median));
        assert_eq!(r.mclk_div(), Ok(MclkDiv::Div8));
        assert_eq!(r.sleep_mode(), false);
        // power=0b10 << 4 | mclk=0b10 = 0x22
        assert_eq!(u8::from(r), 0x22);
        r = r.set_power_mode(PowerMode::Eco);
        assert_eq!(u8::from(r), 0x02);
        r = r.set_mclk_div(MclkDiv::Div32);
        assert_eq!(u8::from(r), 0x00);

    }

    #[test]
    fn general_config() {
        let mut r = GeneralConfig::default()
            .set_retime_enable(true)
            .set_vcm_power_down(true); //0 0x38
        assert_eq!(r.retime_enable(), true);
        assert_eq!(r.vcm_powered_down(), true);
        assert_eq!(u8::from(r), 0x38);
    }
    #[test]
    fn interface_config_crc_and_dclk() {
        let r = InterfaceConfig::default()
            .with_crc_4sample()
            .set_dclk_div(DclkDiv::Div4);
        assert_eq!(r.crc_select(), Ok(CrcSelect::Every4));
        assert_eq!(r.dclk_div(), Ok(DclkDiv::Div4));
        // crc=0b01 << 2 | dclk=0b01 = 0x05
        assert_eq!(u8::from(r), 0x05);
    }

    #[test]
    fn data_control_sync_sequence() {
        let base = DataControl::default(); // 0x80
        assert!(base.sync_high());
        let low = base.with_sync_low();
        assert!(!low.sync_high());
        assert_eq!(u8::from(low), 0x00);
        let high = low.with_sync_high();
        assert!(high.sync_high());
        assert_eq!(u8::from(high), 0x80);
    }

    #[test]
    fn data_control_reset_bytes() {
        assert_eq!(u8::from(DataControl::reset_byte1()), 0x03);
        assert_eq!(u8::from(DataControl::reset_byte2()), 0x02);
    }

    #[test]
    fn device_status_bits() {
        let s = DeviceStatus(0x0F);
        assert!(s.chip_error());
        assert!(s.no_clock_error());
        assert!(s.bist_passed());
        assert!(s.bist_running());
        let clean = DeviceStatus(0x00);
        assert!(!clean.chip_error());
    }

    #[test]
    fn gpio_control_roundtrip() {
        let g = GpioControl::default()
            .set_ugpio_enable(true)
            .set_direction(0, true)   // GPIO0 output
            .set_direction(4, true);  // GPIO4 output
        assert!(g.ugpio_enabled());
        assert!(g.is_output(0));
        assert!(!g.is_output(1));
        assert!(g.is_output(4));
    }

    #[test]
    fn gpio_write_roundtrip() {
        let w = GpioWrite::default().set(2, true).set(4, true);
        assert!(!w.get(0));
        assert!(w.get(2));
        assert!(w.get(4));
    }

    #[test]
    fn channel_mode_select_per_channel() {
        let s = ChannelModeSelect::default()
            .set_channel_mode(0, ChannelMode::ModeB)
            .set_channel_mode(5, ChannelMode::ModeB);
        assert_eq!(s.channel_mode(0), ChannelMode::ModeB);
        assert_eq!(s.channel_mode(1), ChannelMode::ModeA);
        assert_eq!(s.channel_mode(5), ChannelMode::ModeB);
    }

    #[test]
    fn precharge_buf_pairs() {
        let b = PrechargeBuf::ALL_ON;
        assert!(b.pos_enabled(0));
        assert!(b.neg_enabled(0));
        let b2 = b.set_pos(1, false);
        assert!(!b2.pos_enabled(1));
        assert!(b2.neg_enabled(1));
    }

    #[test]
    fn ref_precharge_buf_roundtrip() {
        let r = RefPrechargeBuf::default()
            .set_enabled(0, true)
            .set_enabled(3, true);
        assert!(r.enabled(0));
        assert!(!r.enabled(1));
        assert!(r.enabled(3));
    }

    #[test]
    fn cal24_signed() {
        // Positive
        let c = Cal24::from_i32(1000);
        assert_eq!(c.as_i32(), 1000);
        // Negative
        let c2 = Cal24::from_i32(-500);
        assert_eq!(c2.as_i32(), -500);
        // Byte split / reassembly
        let c3 = Cal24::from_i32(0x123456);
        assert_eq!(c3.msb(), 0x12);
        assert_eq!(c3.mid(), 0x34);
        assert_eq!(c3.lsb(), 0x56);
        assert_eq!(Cal24::from_bytes(0x12, 0x34, 0x56).as_i32(), 0x123456);
    }

    #[test]
    fn adc_frame_positive() {
        // header=0x00 (ch0, wideband, settled), data = 0x7FFFFF (+FS-1LSB)
        let raw: u32 = 0x007F_FFFF;
        let f = AdcFrame::from_u32(raw);
        assert_eq!(f.header.channel_id, 0);
        assert!(!f.header.chip_error);
        assert_eq!(f.data, 0x7F_FFFF);
    }

    #[test]
    fn adc_frame_negative() {
        // data = 0x800000 (−FS)
        let raw: u32 = 0x0080_0000;
        let f = AdcFrame::from_u32(raw);
        assert_eq!(f.data, -8_388_608); // −2^23
    }

    #[test]
    fn adc_frame_header_decode() {
        // chip_error=1, filter_not_settled=0, repeated=0, sinc5=1, sat=0, ch=3
        let hdr: u8 = 0x80 | 0x10 | 0x03;
        let h = StatusHeader::from_byte(hdr);
        assert!(h.chip_error);
        assert!(!h.filter_not_settled);
        assert!(h.sinc5_filter);
        assert!(!h.filter_saturated);
        assert_eq!(h.channel_id, 3);
        assert_eq!(h.to_byte(), hdr);
    }

    #[test]
    fn odr_calculation() {
        // Fast mode, 32.768 MHz, dec×32 → 256 kSPS
        let odr = calc_odr_hz(32_768_000, MclkDiv::Div4, DecRate::X32);
        assert_eq!(odr, 256_000);
        // Eco-mode, dec×1024 → 1 kSPS
        let odr2 = calc_odr_hz(32_768_000, MclkDiv::Div32, DecRate::X1024);
        assert_eq!(odr2, 1_000);
    }

    #[test]
    fn code_to_mv_midscale() {
        // code=0 → 0 mV regardless of Vref
        assert_eq!(code_to_mv(0, 4096), 0);
    }

    #[test]
    fn mod_delay_reserved_bits() {
        // Constructing from raw byte must preserve reserved bits = 0b10
        let r = ModDelayCtrl::from(0xFF);
        assert_eq!(r.0 & 0x3, 0x02, "reserved bits must be 0b10");
    }

    #[test]
    fn chop_control_reset() {
        let r = ChopControl::default();
        assert_eq!(r.group_a(), Ok(ChopFreq::FmodDiv32));
        assert_eq!(r.group_b(), Ok(ChopFreq::FmodDiv32));
    }

    #[test]
    fn diagnostic_mux_roundtrip() {
        let d = DiagnosticMux::default()
            .set_group_a(DiagMuxSel::PosFull)
            .set_group_b(DiagMuxSel::NegFull);
        assert_eq!(d.group_a(), Ok(DiagMuxSel::PosFull));
        assert_eq!(d.group_b(), Ok(DiagMuxSel::NegFull));
    }

    #[test]
    fn dec_rate_factors() {
        assert_eq!(DecRate::X32.factor(), 32);
        assert_eq!(DecRate::X1024.factor(), 1024);
    }

    #[test]
    fn dclk_div_divisors() {
        assert_eq!(DclkDiv::Div8.divisor(), 8);
        assert_eq!(DclkDiv::Div1.divisor(), 1);
    }
}


