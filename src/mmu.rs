pub trait MMU {
    fn read_byte_immutable(&self, addr: u32) -> u8;
    fn read_word_immutable(&self, addr: u32) -> u16;
    fn read_long_immutable(&self, addr: u32) -> u32;

    fn read_byte(&mut self, addr: u32) -> u8;
    fn read_word(&mut self, addr: u32) -> u16;
    fn read_long(&mut self, addr: u32) -> u32;

    fn write_byte(&mut self, addr: u32, val: u8);
    fn write_word(&mut self, addr: u32, val: u16);
    fn write_long(&mut self, addr: u32, val: u32);
}
