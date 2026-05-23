#![no_std]

use embassy_stm32::eth::{PHY, StationManagement};

const BCR: u8 = 0x00;     // basic control
const BSR: u8 = 0x01;     // basic status
const PSCSR: u8 = 0x1F;   // phy special control
const BCR_RESET: u16 = 0x8000;  
const BCR_AUTONEG: u16 = 0x1200; // value to enable + restart autoneg

pub struct Lan8742 {
    addr: u8,
}

impl Lan8742 {
    pub fn new(addr: u8) -> Self {
        Self { addr }
    }
}

impl PHY for Lan8742 {
    fn init(&mut self, sm: &mut impl StationManagement) {
        sm.smi_write(self.addr, BCR, BCR_RESET); //write 1 to bit 15 (reset bit) of the bcr
        while sm.smi_read(self.addr, BCR) & BCR_RESET != 0 {} //wait for the reset bit to clear
        sm.smi_write(self.addr, BCR, BCR_AUTONEG); 
    }

    fn poll_link(&mut self, sm: &mut impl StationManagement) -> bool {
        let bsr_value = sm.smi_read(self.addr, BSR);
        if bsr_value & 0x0004 == 0 { //if bit 2 is 0 then link is down so return false
            return false; 
        }
        let pscsr_value = sm.smi_read(self.addr, PSCSR);
        let speed_duplex = (pscsr_value >> 2) & 0x07; //only keep last 3 bits
        match speed_duplex {
            0b001 | 0b010 | 0b101 | 0b110 => true, //only these settings are allowed
            _ => false,
        }
    }
}
