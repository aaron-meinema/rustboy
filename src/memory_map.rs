use crate::cardridge::Cardridge;

pub struct MemoryMap {
    pub cardridge: Cardridge,
    memory: Vec<u8>,
}

impl MemoryMap {
    pub fn new(the_cardridge: Cardridge) -> MemoryMap {
        let mut mem = Vec::new();
        for _ in 0..0xffff {
            mem.push(0);
        }

        let memory_map = MemoryMap {
            cardridge: the_cardridge,
            memory: mem,
        };

        memory_map
    }

    pub fn get_8bit(&self, memory_location: u8) -> u8 {
        let memory_address = self.get_8bit_address(memory_location);
        self.get_8bit_full_address(memory_address)
    }

    pub fn get_8bit_full_address(&self, memory_location: usize) -> u8 {
        self.memory.get(memory_location).unwrap().clone()
    }

    pub fn store_8bit(&mut self, memory_location: u8, value: u8) {
        let memory_address = self.get_8bit_address(memory_location);
        self.store_8bit_full_address(memory_address, value);
    }

    pub fn store_8bit_full_address(&mut self, memory_location: usize, value: u8) {
        self.memory.insert(memory_location.into(), value);
    }

    fn get_8bit_address(&self, memory_location: u8) -> usize {
        let location_16bit: u16 = memory_location.into();
        (0xff00 + location_16bit).into()
    }
}