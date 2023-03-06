use crate::cardridge::{Cardridge};
use crate::memory_map::MemoryMap;
pub struct Cpu {
    b: u8,  // 000
    c: u8,  // 001
    d: u8,  // 010
    e: u8,  // 011
    h: u8,  // 100
    l: u8,  // 101
    a: u8,  // 111

    f: u8,
    cycle_counter: u16,
    memory_counter: usize,
    memory_map: MemoryMap
}
const CPU_FIRST: u8  = 0b0000_0111;
const CPU_SECOND: u8 = 0b0011_1000;
impl Cpu {
    pub fn new(the_cardridge: Cardridge) -> Cpu {
        let cpu = Cpu {
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            a: 0,

            f: 0,
            memory_counter: 0,
            cycle_counter: 0,
            memory_map: MemoryMap::new(the_cardridge),
        };
        cpu
    }

    pub fn start_cycle(&mut self) {
        loop {
            if self.memory_counter >= self.memory_map.cardridge.memory.len().try_into().unwrap() {
                return
            }
            let number = self.memory_map.cardridge.memory.get(self.memory_counter).unwrap();
            self.run_opcode(*number);
        }
    }

    fn run_opcode(&mut self, opcode: u8) {
        match opcode {
            0x02         => self.ldbca(),
            0x12         => self.lddea(),
            0x22         => self.ldhlp(),
            0x32         => self.ldhlm(),
            0xf0         => self.ld_from_memory(),
            0xf2         => self.ld_from_memory_c(),
            0xe0         => self.ld_to_memory(),
            0xe2         => self.ld_to_memory_c(),
            t if t & 0xc7 == 0x06 => self.ld_from_cardridge(opcode),
            0x40..= 0x7f => self.ldrr(opcode),
            0x80..= 0x87 => self.add(opcode),
            0x88..= 0x8f => self.adc(opcode),
            _ => self.default(opcode),

        }
    }

    pub fn a(&self) -> u8 {
        self.a
    }

    fn ldhlm(&mut self) {
        if self.l == 0x00 {
            self.l = 0xff;
            self.h -= 1;
        }
        else {
            self.l -= 1;
        }
        let location = self.get_hl();
        self.memory_map.store_8bit_full_address(location.into(), self.a);
    }

    fn ldhlp(&mut self) {
        if self.l == 0xff {
            self.l = 0x00;
            self.h += 1;
        }
        else {
            self.l += 1;
        }
        let location = self.get_hl();
        self.memory_map.store_8bit_full_address(location.into(), self.a);
    }

    fn ldbca(&mut self) {
        let location = self.get_bc();
        self.memory_map.store_8bit_full_address(location.into(), self.a);
        self.cycle_counter += 4;
    }

    fn lddea(&mut self) {
        let location = self.get_de();
        self.memory_map.store_8bit_full_address(location.into(), self.a);
        self.cycle_counter += 4;
    }

    fn ld_to_memory(&mut self) {
        self.memory_counter += 1;
        let location = self.memory_map.cardridge.memory.get(self.memory_counter).unwrap().clone();
        self.memory_map.store_8bit(location, self.a);
        self.memory_counter += 1;
        self.cycle_counter += 12;
    }

    fn ld_to_memory_c(&mut self) {
        self.memory_map.store_8bit(self.c, self.a);
        self.memory_counter += 1;
        self.cycle_counter += 8;
    }


    fn ld_from_memory(&mut self) {
        self.memory_counter += 1;
        let location = self.memory_map.cardridge.memory.get(self.memory_counter).unwrap().clone();
        self.a = self.memory_map.get_8bit(location);
        self.memory_counter += 1;
        self.cycle_counter += 12;
    }

    fn ld_from_memory_c(&mut self) {
        self.a = self.memory_map.get_8bit(self.c);
        self.memory_counter += 1;
        self.cycle_counter += 8;
    }

    fn ld_from_cardridge(&mut self, opcode: u8) {
        self.memory_counter += 1;
        let register = u8::from(opcode & CPU_SECOND);
        let value = self.memory_map.cardridge.memory.get(self.memory_counter).unwrap().clone();
        self.store_value_into_register(value, register);
        self.memory_counter += 1;
        self.cycle_counter += 8;
    }


    fn ldrr(&mut self, opcode: u8) {
        let first = u8::from(opcode & CPU_FIRST);
        let second = u8::from(opcode & CPU_SECOND);
        let value = self.get_value_from_register(first);
        self.store_value_into_register(value, second);
        self.memory_counter += 1;
        self.cycle_counter += 4;
    }

    fn add(&mut self, opcode: u8) {
        let register = u8::from(opcode & CPU_FIRST);
        let value_from_reg = self.get_value_from_register(register);
        let value_overflow = self.a.overflowing_add(value_from_reg);
        self.set_flag_z(value_overflow.0);
        self.set_flag_n(false);
        self.set_flag_h(self.a, value_from_reg);
        self.set_flag_c(value_overflow.1);
        self.a = value_overflow.0;
        self.memory_counter += 1;
        self.cycle_counter += 4;
    }

    fn adc(&mut self, opcode: u8) {
        let register = u8::from(opcode & CPU_FIRST);
        let value_from_reg = self.get_value_from_register(register);
        let value_overflow = self.a.overflowing_add(value_from_reg + self.get_c_value());
        self.set_flag_z(value_overflow.0);
        self.set_flag_n(false);
        self.set_flag_h(self.a, value_from_reg);
        self.set_flag_c(value_overflow.1);
        self.a = value_overflow.0;
        self.memory_counter += 1;
        self.cycle_counter += 4;
    }

    fn set_flag_z(&mut self, result: u8) {
        if result == 0 {
            self.f |= 0x80;
        }
        else {
            self.f &= 0x7f;
        }
    }

    fn set_flag_n(&mut self, set: bool) {
        if set {
            self.f |= 0x40;
        }
        else {
            self.f &= 0xbf;
        }
    }

    fn set_flag_h(&mut self, first: u8, second: u8) {
        if (((first & 0xf) + (second & 0xf)) & 0x10) == 0x10 {
            self.f |= 0x20;
        }
        else {
            self.f &= 0xdf;
        }
    }

    fn set_flag_c(&mut self, set: bool) {
        if set {
            self.f |= 0x10;
        }
        else {
            self.f &= 0xef;
        }
    }

    fn get_flag_z(&self) -> bool {
        let num = self.f & 0x80;
        return num == 0x80;
    }

    fn get_flag_n(&self) -> bool {
        let num = self.f & 0x40;
        return num == 0x40;
    }

    fn get_flag_h(&self) -> bool {
        let num = self.f & 0x20;
        return num == 0x20;
    }

    fn get_flag_c(&self) -> bool {
        let num = self.f & 0x10;
        return num == 0x10;
    }

    fn get_c_value(&self) -> u8 {
        if self.get_flag_c(){
            return 1
        }
        0
    }

    fn default(&mut self, byte: u8) {
        self.a = byte;
        self.memory_counter += 1;
        self.cycle_counter += 1;
    }

    fn get_value_from_register(&mut self, register: u8) -> u8 {
        match register {
            0b000 => self.b,
            0b001 => self.c,
            0b010 => self.d,
            0b011 => self.e,
            0b100 => self.h,
            0b101 => self.l,
            0b110 => self.get_memory_hl(),
            0b111 => self.a,
            _ => self.get_value_from_register(register >> 3)
        }
    }

    fn store_value_into_register(&mut self, value: u8, register: u8) {
        match register {
            0b000 => self.b = value,
            0b001 => self.c = value,
            0b010 => self.d = value,
            0b011 => self.e = value,
            0b100 => self.h = value,
            0b101 => self.l = value,
            0b110 => self.store_hl_memory(value),
            0b111 => self.a = value,
            _ => self.store_value_into_register(value, register >> 3),
        }
    }

    fn get_memory_hl(&mut self) -> u8 {
        let hl = self.get_hl();
        self.memory_map.get_8bit_full_address(hl.into())
    }

    fn store_hl_memory(&mut self, value: u8) {
        let hl = self.get_hl();
        self.memory_map.store_8bit_full_address(hl.into(), value);
    }

    fn get_bc(&mut self) -> u16 {
        let mut b:u16 = self.b.into();
        b = b << 8;
        let c: u16 = self.c.into();
        self.cycle_counter += 4;
        return b + c;

    }

    fn get_de(&mut self) -> u16 {
        let mut d:u16 = self.d.into();
        d = d << 8;
        let e: u16 = self.e.into();
        self.cycle_counter += 4;
        return d + e;

    }

    fn get_hl(&mut self) -> u16 {
        let mut h:u16 = self.h.into();
        h = h << 8;
        let l: u16 = self.l.into();
        self.cycle_counter += 4;
        return h + l;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn get_cpu() -> Cpu {
        let vec1:Vec<u8> = vec![0x40, 0x41, 0x42];
        let cardridge = Cardridge{
            memory: vec1,
        };
        Cpu {
            b: 1,
            c: 2,
            d: 3,
            e: 4,
            h: 5,
            l: 6,
            a: 7,

            f: 0,
            cycle_counter: 0,
            memory_counter: 0,
            memory_map: MemoryMap::new(cardridge)
        }
    }
    #[test]
    fn test_get_from_reg() -> Result<(), String> {
        let mut cpu = get_cpu();
        assert_eq!(cpu.get_value_from_register(0b000), 1);
        assert_eq!(cpu.get_value_from_register(0b001), 2);
        assert_eq!(cpu.get_value_from_register(0b010), 3);
        assert_eq!(cpu.get_value_from_register(0b011), 4);
        assert_eq!(cpu.get_value_from_register(0b100), 5);
        assert_eq!(cpu.get_value_from_register(0b101), 6);
        assert_eq!(cpu.get_value_from_register(0b111), 7);
        Ok(())
    }

    #[test]
    fn test_store_in_reg()-> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.store_value_into_register(8,0b000);
        cpu.store_value_into_register(9,0b001);
        cpu.store_value_into_register(10,0b010);
        cpu.store_value_into_register(11,0b011);
        cpu.store_value_into_register(12,0b100);
        cpu.store_value_into_register(13,0b101);
        cpu.store_value_into_register(14,0b111);


        assert_eq!(cpu.get_value_from_register(0b000), 8);
        assert_eq!(cpu.get_value_from_register(0b001), 9);
        assert_eq!(cpu.get_value_from_register(0b010), 10);
        assert_eq!(cpu.get_value_from_register(0b011), 11);
        assert_eq!(cpu.get_value_from_register(0b100), 12);
        assert_eq!(cpu.get_value_from_register(0b101), 13);
        assert_eq!(cpu.get_value_from_register(0b111), 14);
        Ok(())
    }

    #[test]
    fn test_memory_counter()-> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.start_cycle();
        assert_eq!(cpu.memory_counter, 3);
        Ok(())
    }

    #[test]
    fn test_ldrr()-> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.run_opcode(0x41);
        cpu.run_opcode(0x50);
        assert_eq!(cpu.b, 2);
        assert_eq!(cpu.d, 2);
        Ok(())
    }

    #[test]
    fn test_add()-> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.run_opcode(0x80);       // add b to a
        assert_eq!(cpu.a, 8);
        assert_eq!(cpu.get_flag_z(), false);
        assert_eq!(cpu.get_flag_n(), false);
        assert_eq!(cpu.get_flag_h(), false);
        assert_eq!(cpu.get_flag_c(), false);
        Ok(())
    }

    #[test]
    fn test_add_zero_carry_halfcarry()-> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.b = 0xff - 6;
        cpu.run_opcode(0x80);       // add b to a
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.get_flag_z(), true);
        assert_eq!(cpu.get_flag_n(), false);
        assert_eq!(cpu.get_flag_h(), true);
        assert_eq!(cpu.get_flag_c(), true);
        Ok(())
    }

    #[test]
    fn test_add_carry()-> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.b = 0xf0;
        cpu.a = 0xf0;
        cpu.run_opcode(0x80);       // add b to a
        assert_eq!(cpu.a, cpu.b.overflowing_add(0xf0).0);
        assert_eq!(cpu.get_flag_z(), false);
        assert_eq!(cpu.get_flag_n(), false);
        assert_eq!(cpu.get_flag_h(), false);
        assert_eq!(cpu.get_flag_c(), true);
        Ok(())
    }

    #[test]
    fn test_adc() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.f = 0xf0;
        cpu.run_opcode(0x88);      // add b to a with carry
        assert_eq!(cpu.a, 9);
        assert_eq!(cpu.get_flag_c(), false);
        Ok(())
    }

    #[test]
    fn test_ld_from_memory() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.run_opcode(0xf0);      // add b to a with carry
        assert_eq!(cpu.a, 0);
        Ok(())
    }



    #[test]
    fn test_ld_from_cardridge() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.run_opcode(0x3e);      // add b to a with carry
        assert_eq!(cpu.a, 65);
        cpu = get_cpu();
        cpu.run_opcode(0x06);
        assert_eq!(cpu.b, 65);
        Ok(())
    }


    #[test]
    fn test_ld_to_and_from_memory() -> Result<(), String> {
        let vec1:Vec<u8> = vec![0xe0, 0x80, 0xf0, 0x80];
        let cardridge = Cardridge{
            memory: vec1,
        };
        let mut cpu = get_cpu();
        cpu.memory_map = MemoryMap::new(cardridge);
        cpu.run_opcode(0xe0);      // add b to a with carry
        cpu.a = 0xff;
        cpu.run_opcode(0xf0);
        assert_eq!(cpu.a, 7);
        Ok(())
    }

    #[test]
    fn test_ld_to_and_from_memory_c() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.c = 0x99;
        cpu.run_opcode(0xe2);      // add b to a with carry
        cpu.a = 0xff;
        cpu.run_opcode(0xf2);
        assert_eq!(cpu.a, 7);
        Ok(())
    }

    #[test]
    fn test_get_hl()-> Result<(), String> {
        let mut cpu = get_cpu();
        assert_eq!(cpu.get_hl(), 0x0506);
        Ok(())
    }

    #[test]
    fn test_get_store_hl()-> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.run_opcode(0x70);
        cpu.run_opcode(0x56);
        assert_eq!(cpu.d, cpu.b);
        Ok(())
    }

    #[test]
    fn test_multiple_ld_to_memory()-> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.run_opcode(0x02);
        cpu.run_opcode(0x12);
        cpu.run_opcode(0x22);
        cpu.run_opcode(0x32);


        assert_eq!(cpu.memory_map.get_8bit_full_address(0x0102), cpu.a);
        assert_eq!(cpu.memory_map.get_8bit_full_address(0x0304), cpu.a);
        assert_eq!(cpu.memory_map.get_8bit_full_address(0x0507), cpu.a);
        assert_eq!(cpu.memory_map.get_8bit_full_address(0x0506), cpu.a);
        assert_eq!(cpu.memory_map.get_8bit_full_address(0xffff), 0);
        Ok(())
    }


}