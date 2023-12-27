use crate::cardridge::Cardridge;
use crate::renderer::Renderer;

pub struct MemoryMap {
    pub cardridge: Cardridge,
    pub renderer: Renderer,
    d_pad: u8,
    buttons: u8,
    memory: [u8; 0x10000],
}

impl MemoryMap {
    pub fn new(the_cardridge: Cardridge) -> MemoryMap {
        let mem: [u8; 0x10000] = [0; 0x10000];

        let memory_map = MemoryMap {
            cardridge: the_cardridge,
            renderer: Renderer::new(),
            d_pad: 0xff,
            buttons: 0xff,
            memory: mem,
        };

        memory_map
    }

    pub fn get_8bit(&self, memory_location: u8) -> u8 {
        let memory_address = self.get_8bit_address(memory_location);
        self.get_8bit_full_address(memory_address)
    }

    pub fn get_8bit_full_address(&self, memory_location: usize) -> u8 {
        match memory_location {
            0xff00      => return self.get_joypad(),
            _           => return self.memory.get(memory_location).unwrap().clone()
        } 
    }

    pub fn store_8bit(&mut self, memory_location: u8, value: u8) {
        let memory_address = self.get_8bit_address(memory_location);
        self.store_8bit_full_address(memory_address, value);
    }

    pub fn store_d_pad(&mut self, d_pad: u8) {
        self.d_pad = d_pad;
    } 

    pub fn store_buttons(&mut self, buttons: u8) {
        self.buttons = buttons;
    }

    fn get_joypad(&self) -> u8 {
        let compare = self.memory[0xff00] & 0x30;
        if compare == 0x30 {
            return 0x3f;
        }
        else if compare == 0x20 {
            return compare + (self.d_pad & 0x0f);
        }
        else if compare == 0x10 {
            return compare + (self.buttons & 0x0f);
        }
        else {
            return 0x00;
        }
    }

    pub fn store_8bit_full_address(&mut self, memory_location: usize, value: u8) {
        self.memory[memory_location] = value;
        match memory_location {
            0x8000..= 0x9fff => self.renderer.store(memory_location, value),
            0xff40..         => self.renderer.set_lcdc(value),
            _ => (),
        }

        self.memory[memory_location] = value;

    }

    fn get_8bit_address(&self, memory_location: u8) -> usize {
        let location_16bit: u16 = memory_location.into();
        (0xff00 + location_16bit).into()
    }
}