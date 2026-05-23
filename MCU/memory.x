MEMORY
{
  FLASH   : ORIGIN = 0x08000000, LENGTH = 2048K
  RAM     : ORIGIN = 0x20000000, LENGTH = 128K
  AXISRAM : ORIGIN = 0x24000000, LENGTH = 512K
  SRAM1   : ORIGIN = 0x30000000, LENGTH = 128K  /* D2 SRAM - Ethernet DMA */
}

SECTIONS
{
    .eth_buffers (NOLOAD) : ALIGN(4)
    {
        *(.eth_buffers);
    } > SRAM1
}