use crate::cardridge::Cardridge;

pub struct Memory_Map {
    pub cardridge: Cardridge,
    memory: Vec<u8>,
}

impl Memory_Map {
    pub fn new(the_cardridge: Cardridge) -> Memory_Map {
        let mut mem = Vec::new();
        for _ in 0..0xffff {
            mem.push(0);
        }

        let memory_map = Memory_Map {
            cardridge: the_cardridge,
            memory: mem,
        };

        memory_map
    }

    pub fn get_8bit(&self, memory_location: u8) -> u8 {
        let memory_address = self.get_8bit_address(memory_location);
        self.memory.get(memory_address).unwrap().clone()
    }

    pub fn store_8bit(&mut self, memory_location: u8, value: u8) {
        let memory_address = self.get_8bit_address(memory_location);
        self.memory.insert(memory_address, value);
    }

    fn get_8bit_address(&self, memory_location: u8) -> usize {
        let location_16bit: u16 = memory_location.into();
        (0xff00 + location_16bit).into()
    }
}