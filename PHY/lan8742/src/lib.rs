#![no_std]

use core::task::Context;
use embassy_stm32::eth::{Phy, StationManagement};

const BCR: u8 = 0x00;     // basic control
const BSR: u8 = 0x01;     // basic status
const PSCSR: u8 = 0x1F;   // phy special control
const BCR_RESET: u16 = 0x8000;  
const BCR_AUTONEG: u16 = 0x1200; // value to enable + restart autoneg

pub struct Lan8742<S: StationManagement> {
    addr: u8,
    sm: S,
}

impl<S: StationManagement> Lan8742<S> {
    pub fn new(addr: u8, sm: S) -> Self {
        Self { addr, sm }
    }
}

impl<S: StationManagement> Phy for Lan8742<S> {
    fn phy_reset(&mut self) {
        self.sm.smi_write(self.addr, BCR, BCR_RESET); //write 1 to bit 15 (reset bit) of the bcr
        while self.sm.smi_read(self.addr, BCR) & BCR_RESET != 0 {} //wait for the reset bit to clear
    }

    fn phy_init(&mut self) {
        self.sm.smi_write(self.addr, BCR, BCR_AUTONEG);
    }

    fn poll_link(&mut self, _cx: &mut Context) -> bool {
        let bsr_value = self.sm.smi_read(self.addr, BSR);
        if bsr_value & 0x0004 == 0 { //if bit 2 is 0 then link is down so return false
            return false; 
        }
        let pscsr_value = self.sm.smi_read(self.addr, PSCSR);
        let speed_duplex = (pscsr_value >> 2) & 0x07; //only keep last 3 bits
        match speed_duplex {
            0b001 | 0b010 | 0b101 | 0b110 => true, //only these settings are allowed
            _ => false,
        }
    }
}
