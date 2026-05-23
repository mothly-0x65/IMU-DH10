/// Configures the MPU to mark D2 SRAM (0x30000000) as non-cacheable.
/// This is required on STM32H7 so that the Ethernet DMA and CPU 
/// don't get out of sync due to the Cortex-M7 cache.
pub fn init() {
    unsafe {
        let mpu = &*cortex_m::peripherals::MPU::PTR; // memory processing unit register 
        let scb = &*cortex_m::peripherals::SCB::PTR; 

        mpu.ctrl.write(0); // disable the mpu before configuring it

        cortex_m::asm::dmb(); // memory barrier 

        mpu.rnr.write(0); // select region 0 to configure

        mpu.rbar.write(0x30000000); // set the base address to D2 SRAM

        // configure the region properties
        mpu.rasr.write(
            (0b011 << 24) |  // AP: full access for both privileged and unprivileged code
            (0b001 << 19) |  // TEX: type extension field, set to 1 for normal memory
            (0 << 17)     |  // C: cacheable = 0 (disabled)
            (0 << 16)     |  // B: bufferable = 0 (disabled)
            (0b10000 << 1)|  // SIZE: 2^(16+1) = 128KB, matches our SRAM1 region
            (1 << 0)         // ENABLE: turn this region on 
        );
        
        // re-enable mpu
        // bit 0 = enable mpu
        // bit 2 = use default memory map for regions not covered by mpu
        mpu.ctrl.write((1<<2) | (1 << 0));

        // instruction and data barrier - flush the pipeline
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
    }
}