use crate::cardridge::{Cardridge, self};

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
    cardridge: Cardridge
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
            cardridge: the_cardridge
        };
        cpu
    }

    pub fn start_cycle(&mut self) {
        loop {
            if self.memory_counter >= self.cardridge.memory.len().try_into().unwrap() {
                return
            }
            let number = self.cardridge.memory.get(self.memory_counter).unwrap();
            self.run_opcode(*number);
        }
    }

    fn run_opcode(&mut self, opcode: u8) {
        match opcode {
            0x40..= 0x7f => self.ldrr(opcode),
            0x80..= 0x8f => self.add(opcode),
            _ => self.default(opcode),

        }
    }

    pub fn a(&self) -> u8 {
        self.a
    }

    fn ldrr(&mut self, opcode: u8) {
        let first = u8::from(opcode & CPU_FIRST);
        let second = u8::from(opcode & CPU_SECOND);
        let value = self.get_value_from_register(first);
        self.store_value_into_register(second, value);
        self.memory_counter += 1;
        self.cycle_counter += 1;
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
        self.cycle_counter += 1;
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

    fn default(&mut self, byte: u8) {
        self.a = byte;
        self.memory_counter += 1;
        self.cycle_counter += 1;
    }

    fn get_value_from_register(&self, register: u8) -> u8 {
        match register {
            0b000 => self.b,
            0b001 => self.c,
            0b010 => self.d,
            0b011 => self.e,
            0b100 => self.h,
            0b101 => self.l,
            0b111 => self.a,
            _ => 8
        }
    }

    fn store_value_into_register(&mut self, register: u8, value: u8) {
        match register {
            0b000 => self.b = value,
            0b001 => self.c = value,
            0b010 => self.d = value,
            0b011 => self.e = value,
            0b100 => self.h = value,
            0b101 => self.l = value,
            0b111 => self.a = value,
            _ => self.a = self.a,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn get_cpu() -> Cpu {
        let vec1:Vec<u8> = vec![0x40, 0x41, 0x42];
        let cardridge = cardridge::Cardridge{
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
            cardridge: cardridge
        }
    }
    #[test]
    fn test_get_from_reg() -> Result<(), String> {
        let cpu = get_cpu();
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
        cpu.store_value_into_register(0b000, 8);
        cpu.store_value_into_register(0b001, 9);
        cpu.store_value_into_register(0b010, 10);
        cpu.store_value_into_register(0b011, 11);
        cpu.store_value_into_register(0b100, 12);
        cpu.store_value_into_register(0b101, 13);
        cpu.store_value_into_register(0b111, 14);


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
        assert_eq!(cpu.b, 2);
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


}