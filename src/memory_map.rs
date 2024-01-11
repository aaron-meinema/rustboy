use crate::cardridge::Cardridge;
use crate::renderer::Renderer;

pub struct MemoryMap {
    pub cardridge: Cardridge,
    pub renderer: Renderer,
    memory: [u8; 0x10000],
}

impl MemoryMap {
    pub fn new(the_cardridge: Cardridge) -> MemoryMap {
        let mem: [u8; 0x10000] = [0; 0x10000];

        let memory_map = MemoryMap {
            cardridge: the_cardridge,
            renderer: Renderer::new(),
            memory: mem,
        };

        memory_map
    }

    pub fn get_8bit(&self, memory_location: u8) -> u8 {
        let memory_address = self.get_8bit_address(memory_location);
        self.get_8bit_full_address(memory_address)
    }

    pub fn get_8bit_full_address(&self, memory_location: usize) -> u8 {
        for x in 0x0500..0x0510 {
            println!("location: {:x} has the value: {}",x, self.memory.get(x).unwrap());
        }
        self.memory.get(memory_location).unwrap().clone()
    }

    pub fn store_8bit(&mut self, memory_location: u8, value: u8) {
        let memory_address = self.get_8bit_address(memory_location);
        self.store_8bit_full_address(memory_address, value);
    }

    pub fn store_8bit_full_address(&mut self, memory_location: usize, value: u8) {
        match memory_location {
            0x8000..= 0x9fff => self.renderer.store(memory_location, value),
            0xff40           => self.renderer.set_lcdc(value),
            
            _ =>                self.memory[memory_location] = value,
        }
    }

    fn get_8bit_address(&self, memory_location: u8) -> usize {
        let location_16bit: u16 = memory_location.into();
        (0xff00 + location_16bit).into()
    }
}