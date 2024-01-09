use sdl2::libc::OVERLAYFS_SUPER_MAGIC;
use serde::de::value::Error;

use crate::cardridge::Cardridge;
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
    stack_counter: u16,
    stopped: bool,
    pub memory_map: MemoryMap
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
            stack_counter: 0,
            stopped: false,
            memory_map: MemoryMap::new(the_cardridge),
        };
        cpu
    }

    pub fn start_cycle(&mut self) {
        if !self.stopped {
            loop {
                if self.memory_counter >= self.memory_map.cardridge.memory.len().try_into().unwrap() {
                    return
                }
                let number = self.get_from_cardridge();
                self.run_opcode(number);
            }
        }
    }

    fn get_from_cardridge(&mut self) -> u8 {
        return *self.memory_map.cardridge.memory.get(self.memory_counter).unwrap();
    }

    fn run_opcode(&mut self, opcode: u8) {
        match opcode {
            0x00         => self.nop(),
            0x01         => self.ld_bc(),
            0x02         => self.ldbca(),
            0x03         => self.incbc(),
            0x07         => self.rcla(),
            0x0b         => self.decbc(),
            0x10         => self.stop(),
            0x11         => self.ld_de(),
            0x12         => self.lddea(),
            0x13         => self.incde(),
            0x1b         => self.decde(),
            0x21         => self.ld_hl(),
            0x22         => self.ldhlp(),
            0x23         => self.inchl(),
            0x27         => self.daa(),
            0x2b         => self.dechl(),
            0x2f         => self.cpl(),
            0x31         => self.ld_sp(),
            0x32         => self.ldhlm(),
            0x33         => self.incsp(),
            0x3b         => self.decsp(),
            0x3f         => self.ccf(),
            0x0a         => self.ldabc(),
            0x1a         => self.ldade(),
            0x2a         => self.ldahlp(),
            0x37         => self.scf(),
            0x3a         => self.ldahlm(),
            0xea         => self.ld_a16_a(),
            0xfa         => self.ld_a_a16(),
            0xf0         => self.ld_from_memory(),
            0xf2         => self.ld_from_memory_c(),
            0xe0         => self.ld_to_memory(),
            0xe2         => self.ld_to_memory_c(),
            0x40..= 0x7f => self.ldrr(opcode),
            0x80..= 0x87 => self.add(opcode),
            0x88..= 0x8f => self.adc(opcode),
            0x90..= 0x97 => self.sub(opcode),
            0x98..= 0x9f => self.sbc(opcode),
            0xa0..= 0xa7 => self.and(opcode),
            0xa8..= 0xaf => self.xor(opcode),
            0xb0..= 0xb7 => self.or(opcode),
            0xb8..= 0xbf => self.cp(opcode),
            t if t & 0xc7 == 0x06 => self.ld_from_cardridge(opcode),
            t if t & 0xc7 == 0x04 => self.inc(opcode),
            t if t & 0xc7 == 0x05 => self.dec(opcode),
            _ => self.default(opcode),

        }
    }

    fn rcla(&mut self) {
        self.memory_counter += 1;
        let result = self.a >> 7;

        self.a = self.a << 1;
        self.set_flag_c(result == 1);
        self.cycle_counter +=4;
    }

    fn scf(&mut self) {
        self.set_flag_c(true);
        self.set_flag_h(false);
        self.set_flag_n(false);
        self.memory_counter += 1;
        self.cycle_counter += 4;
    }

    fn cpl(&mut self) {
        self.a = !self.a;
        self.set_flag_n(true);
        self.set_flag_h(true);
        self.memory_counter += 1;
        self.cycle_counter += 4;
    }

    fn ccf(&mut self) {
        self.f  = self.f ^ 0x10;
        self.set_flag_n(false);
        self.set_flag_h(false);
        self.memory_counter += 1;
        self.cycle_counter += 4;
    }

    fn incbc(&mut self) {
        let overflow = self.c.overflowing_add(1);
        self.c = overflow.0;
        if overflow.1 {
            self.b = self.b.overflowing_add(1).0;
        }
        self.memory_counter += 1;
        self.cycle_counter += 8;
    }

    fn incde(&mut self) {
        let overflow = self.e.overflowing_add(1);
        self.e = overflow.0;
        if overflow.1 {
            self.d = self.d.overflowing_add(1).0;
        }
        self.memory_counter += 1;
        self.cycle_counter += 8;
    }

    fn inchl(&mut self) {
        let overflow = self.l.overflowing_add(1);
        self.l = overflow.0;
        if overflow.1 {
            self.h = self.h.overflowing_add(1).0;
        }
        self.memory_counter += 1;
        self.cycle_counter += 8;
    }

    fn incsp(&mut self) {
        let overflow = self.stack_counter.overflowing_add(1);
        self.memory_counter += 1;
        self.cycle_counter += 8;
    }

    fn decbc(&mut self) {
        let overflow = self.c.overflowing_sub(1);
        self.c = overflow.0;
        if overflow.1 {
            self.b = self.b.overflowing_sub(1).0;
        }
        self.memory_counter += 1;
        self.cycle_counter += 8;
    }

    fn decde(&mut self) {
        let overflow = self.e.overflowing_sub(1);
        self.e = overflow.0;
        if overflow.1 {
            self.d = self.d.overflowing_sub(1).0;
        }
        self.memory_counter += 1;
        self.cycle_counter += 8;
    }

    fn dechl(&mut self) {
        let overflow = self.l.overflowing_sub(1);
        self.l = overflow.0;
        if overflow.1 {
            self.h = self.h.overflowing_sub(1).0;
        }
        self.memory_counter += 1;
        self.cycle_counter += 8;
    }

    fn decsp(&mut self) {
        let overflow = self.stack_counter.overflowing_sub(1);
        self.memory_counter += 1;
        self.cycle_counter += 8;
    }

    fn ld_bc(&mut self) {
        self.memory_counter += 1;
        self.b = self.get_from_cardridge();
        self.memory_counter += 1;
        self.c = self.get_from_cardridge();
        self.memory_counter += 1;
        self.cycle_counter  += 12;
    }

    fn ld_de(&mut self) {
        self.memory_counter += 1;
        self.d = self.get_from_cardridge();
        self.memory_counter += 1;
        self.e = self.get_from_cardridge();
        self.memory_counter += 1;
        self.cycle_counter  += 12;
    }

    fn ld_hl(&mut self) {
        self.memory_counter += 1;
        self.h = self.get_from_cardridge();
        self.memory_counter += 1;
        self.l = self.get_from_cardridge();
        self.memory_counter += 1;
        self.cycle_counter  += 12;
    }

    fn ld_sp(&mut self) {
        self.memory_counter += 1;
        let mut high: u16 = self.get_from_cardridge().into();
        self.memory_counter += 1;
        let low: u16 = self.get_from_cardridge().into();
        high = high << 8;
        self.stack_counter = high + low;
        self.memory_counter += 1;
        self.cycle_counter  += 12;
    }

    fn ld_a16_a(&mut self) {
        self.memory_counter += 1;
        let mut high: u16 = self.get_from_cardridge().into();
        self.memory_counter += 1;
        let low:u16 = self.get_from_cardridge().into();
        high = high << 8;
        self.memory_map.store_8bit_full_address((high + low).into(), self.a);
        self.memory_counter += 1;
        self.cycle_counter += 16;
    }

    fn ld_a_a16(&mut self) {
        self.memory_counter += 1;
        let mut high: u16 = self.get_from_cardridge().into();
        self.memory_counter += 1;
        let low:u16 = self.get_from_cardridge().into();
        high = high << 8;
        self.a = self.memory_map.get_8bit_full_address((high + low).into());
        self.memory_counter += 1;
        self.cycle_counter += 16;
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
        self.cycle_counter += 4;
        self.memory_counter += 1;
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
        self.cycle_counter += 4;
        self.memory_counter += 1;
    }

    fn ldbca(&mut self) {
        let location = self.get_bc();
        self.memory_map.store_8bit_full_address(location.into(), self.a);
        self.cycle_counter += 4;
        self.memory_counter += 1;
    }

    fn ldabc(&mut self) {
        let location = self.get_bc();
        self.a = self.memory_map.get_8bit_full_address(location.into());
        self.cycle_counter += 4;
        self.memory_counter += 1;
    }

    fn ldade(&mut self) {
        let location = self.get_de();
        self.a = self.memory_map.get_8bit_full_address(location.into());
        self.cycle_counter += 4;
        self.memory_counter += 1;
    }

    fn ldahlp(&mut self) {
        if self.l == 0xff {
            self.l = 0x00;
            self.h += 1;
        }
        else {
            self.l += 1;
        }
        let location = self.get_hl();
        self.a = self.memory_map.get_8bit_full_address(location.into());
        self.cycle_counter += 4;
        self.memory_counter += 1;
    }

    fn ldahlm(&mut self) {
        if self.l == 0x00 {
            self.l = 0xff;
            self.h -= 1;
        }
        else {
            self.l -= 1;
        }
        let location = self.get_hl();
        self.a = self.memory_map.get_8bit_full_address(location.into());
        self.cycle_counter += 4;
        self.memory_counter += 1;
    }


    fn lddea(&mut self) {
        let location = self.get_de();
        self.memory_map.store_8bit_full_address(location.into(), self.a);
        self.cycle_counter += 4;
        self.memory_counter += 1;
    }

    fn ld_to_memory(&mut self) {
        self.memory_counter += 1;
        let location = self.get_from_cardridge().clone();
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
        let location = self.get_from_cardridge().clone();
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
        let value = self.get_from_cardridge();
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

    fn sub(&mut self, opcode: u8) {
        let register = u8::from(opcode & CPU_FIRST);
        let value_from_reg = self.get_value_from_register(register);
        let value_overflow = self.a.overflowing_sub(value_from_reg);
        self.set_flag_z(value_overflow.0);
        self.set_flag_n(true);
        self.set_flag_h_neg(self.a, value_from_reg);
        self.set_flag_c(value_overflow.1);
        self.a = value_overflow.0;
        self.memory_counter += 1;
        self.cycle_counter += 4;
    }

    fn sbc(&mut self, opcode: u8) {
        let register = u8::from(opcode & CPU_FIRST);
        let value_from_reg = self.get_value_from_register(register);
        let value_overflow = self.a.overflowing_sub(value_from_reg + self.get_c_value());
        self.set_flag_z(value_overflow.0);
        self.set_flag_n(true);
        self.set_flag_h_neg(self.a, value_from_reg);
        self.set_flag_c(value_overflow.1);
        self.a = value_overflow.0;
        self.memory_counter += 1;
        self.cycle_counter += 4;
    }

    fn add(&mut self, opcode: u8) {
        let register = u8::from(opcode & CPU_FIRST);
        let value_from_reg = self.get_value_from_register(register);
        let value_overflow = self.a.overflowing_add(value_from_reg);
        self.set_flag_z(value_overflow.0);
        self.set_flag_n(false);
        self.set_flag_h_pos(self.a, value_from_reg);
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
        self.set_flag_h_pos(self.a, value_from_reg);
        self.set_flag_c(value_overflow.1);
        self.a = value_overflow.0;
        self.memory_counter += 1;
        self.cycle_counter += 4;
    }

    fn and(&mut self, opcode: u8) {
        let register = u8::from(opcode & CPU_FIRST);
        let value_from_reg = self.get_value_from_register(register);
        self.a = self.a & value_from_reg;

        self.f = 0x20;
        self.set_flag_z(self.a);
    }

    fn xor(&mut self, opcode: u8) {
        let register = u8::from(opcode & CPU_FIRST);
        let value_from_reg = self.get_value_from_register(register);
        self.a = self.a ^ value_from_reg;
        self.f = 0;
        self.set_flag_z(self.a);
    }

    fn or(&mut self, opcode: u8) {
        let register = u8::from(opcode & CPU_FIRST);
        let value_from_reg = self.get_value_from_register(register);
        self.a = self.a | value_from_reg;
        self.f = 0x20;
        self.set_flag_z(self.a);
    }

    fn cp(&mut self, opcode: u8) {
        let register = u8::from(opcode & CPU_FIRST);
        let value_from_reg = self.get_value_from_register(register);
        let value_overflow = self.a.overflowing_sub(value_from_reg);
        self.set_flag_z(value_overflow.0);
        self.set_flag_n(true);
        self.set_flag_h_neg(self.a, value_from_reg);
        self.set_flag_c(value_overflow.1);
        self.memory_counter += 1;
        self.cycle_counter += 4;
    }

    fn inc(&mut self, opcode: u8) {
        let register = u8::from(opcode & CPU_SECOND);
        let value = self.get_value_from_register(register);
        let overflow = value.overflowing_add(1);
        self.set_flag_z(overflow.0);
        self.set_flag_n(false);
        self.set_flag_h_pos(value, 1);
        self.store_value_into_register(overflow.0, register);
        self.memory_counter += 1;
        self.cycle_counter += 4;
    }

    fn dec(&mut self, opcode: u8) {
        let register = u8::from(opcode & CPU_SECOND);
        let value = self.get_value_from_register(register);
        let overflow = value.overflowing_sub(1);
        self.set_flag_z(overflow.0);
        self.set_flag_n(true);
        self.set_flag_h_neg(value, 1);
        self.store_value_into_register(overflow.0, register);
        self.memory_counter += 1;
        self.cycle_counter += 4;
    }

    fn daa(&mut self) {
        let mut overflow = false;
        if self.get_flag_n() {
            if self.get_flag_h() {
                let result = self.a.overflowing_sub(0x6);
                self.a = result.0;
            }

            if self.get_flag_c() {
                let result = self.a.overflowing_sub(0x60);
                self.a = result.0;
            }
        }
        else {
            if self.get_flag_h() || self.a & 0xf > 0x9 {
                let result = self.a.overflowing_add(0x6);
                self.a = result.0;
                overflow |= result.1;
            }

            if self.get_flag_c() || self.a > 0x9f {
                let result = self.a.overflowing_add(0x60);
                self.a = result.0;
                overflow |= result.1;
            }
        }
        self.set_flag_z(self.a);
        self.set_flag_c(overflow);
        self.set_flag_h(false);
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

    fn set_flag_h_pos(&mut self, first: u8, second: u8) {
        if (((first & 0xf) + (second & 0xf)) & 0x10) == 0x10 {
            self.f |= 0x20;
        }
        else {
            self.f &= 0xdf;
        }
    }

    fn set_flag_h_neg(&mut self, first: u8, second: u8) {
        let value = (first &0xf).overflowing_sub(second & 0xf);
        if value.1 {
            self.f |= 0x20;
        }
        else {
            self.f &= 0xdf;
        }
    }

    fn set_flag_h(&mut self, set: bool){
        if set {
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
        self.cycle_counter += 4;
        self.memory_map.get_8bit_full_address(hl.into())
    }

    fn store_hl_memory(&mut self, value: u8) {
        let hl = self.get_hl();
        self.cycle_counter += 4;
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

    fn nop(&mut self) {
        self.cycle_counter += 4;
        self.memory_counter += 1;
    }

    fn stop(&mut self) {
        self.cycle_counter += 4;
        self.memory_counter += 2;
        self.stopped = true;
        !todo!("make renderer and others also stop")
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
            stack_counter: 0,
            stopped: false,
            memory_map: MemoryMap::new(cardridge)
        }
    }

    #[test]
    fn test_rcla() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.a=1;
        cpu.rcla();
        assert_eq!(2, cpu.a);
        assert!(!cpu.get_flag_c());
        cpu.a =0xff;
        cpu.rcla();
        assert_eq!(0xfe, cpu.a);
        assert!(cpu.get_flag_c());

        Ok(())
    }

    #[test]
    fn test_ld_bc() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.ld_bc();
        assert_eq!(0x41, cpu.b);
        assert_eq!(0x42, cpu.c);
        Ok(())
    }

    #[test]
    fn test_incbc() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.c = 255;
        cpu.incbc();
        assert_eq!(2, cpu.b);
        assert_eq!(0, cpu.c);
        cpu.c = 255;
        cpu.b = 255;
        cpu.incbc();
        assert_eq!(0, cpu.b);
        assert_eq!(0, cpu.c);
        cpu.incbc();
        assert_eq!(0, cpu.b);
        assert_eq!(1, cpu.c);
        Ok(())
    }

    #[test]
    fn test_decbc() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.c = 0;
        cpu.decbc();
        assert_eq!(0, cpu.b);
        assert_eq!(255, cpu.c);
        cpu.c = 0;
        cpu.b = 0;
        cpu.decbc();
        assert_eq!(255, cpu.b);
        assert_eq!(255, cpu.c);
        cpu.decbc();
        assert_eq!(255, cpu.b);
        assert_eq!(254, cpu.c);
        Ok(())
    }

    #[test]
    fn test_scf() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.run_opcode(0x37);
        assert_eq!(true, cpu.get_flag_c());
        assert_eq!(false, cpu.get_flag_h());
        assert_eq!(false, cpu.get_flag_n());
        cpu.f = 0xff;
        cpu.run_opcode(0x37);
        assert_eq!(true, cpu.get_flag_c());
        assert_eq!(false, cpu.get_flag_h());
        assert_eq!(false, cpu.get_flag_n());
        Ok(())
    }

    #[test]
    fn test_cpl() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.run_opcode(0x2f);
        assert_eq!(cpu.a, 0xff - 0x07);
        Ok(())
    }

    #[test]
    fn test_ccf() -> Result<(), String> {
        let mut cpu = get_cpu();
        assert_eq!(cpu.get_flag_c(), false);
        cpu.run_opcode(0x3f);
        assert_eq!(cpu.get_flag_c(), true);
        cpu.run_opcode(0x3f);
        assert_eq!(cpu.get_flag_c(), false);
        cpu.f = 0xff;
        cpu.run_opcode(0x3f);
        assert_eq!(cpu.f, 0x8f);
        Ok(())
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

    #[test]
    fn test_multiple_from_memory()-> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.memory_map.store_8bit_full_address(0x0102, 10);
        cpu.memory_map.store_8bit_full_address(0x0304, 20);
        cpu.memory_map.store_8bit_full_address(0x0507, 30);
        cpu.memory_map.store_8bit_full_address(0x0506, 40);


        cpu.run_opcode(0x0a);
        assert_eq!(cpu.a, 10);
        cpu.run_opcode(0x1a);
        assert_eq!(cpu.a, 20);
        cpu.run_opcode(0x2a);
        assert_eq!(cpu.a, 30);
        cpu.run_opcode(0x3a);
        assert_eq!(cpu.a, 40);
        cpu.b = 0xff;
        cpu.c = 0xff;
        cpu.run_opcode(0x0a);
        assert_eq!(cpu.a, 0);
        cpu.l = 0xff;
        cpu.run_opcode(0x2a);
        assert_eq!(cpu.get_hl(), 0x0600);
        cpu.run_opcode(0x3a);
        assert_eq!(cpu.get_hl(), 0x05ff);
        Ok(())
    }

    #[test]
    fn test_ld_to_and_from_a16()-> Result<(), String> {
        let vec1:Vec<u8> = vec![0xea, 0xff, 0x80, 0xfe, 0xff, 0x80];
        let cardridge = Cardridge{
            memory: vec1,
        };

        let mut cpu = get_cpu();
        cpu.memory_map.cardridge = cardridge;
        cpu.run_opcode(0xea);
        cpu.a = 0xff;
        cpu.run_opcode(0xfa);
        assert_eq!(cpu.a, 7);
        assert_eq!(cpu.memory_map.get_8bit(0x80), cpu.a);
        Ok(())
    }

    #[test]
    fn test_h_flag() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.a = 0xf8;
        cpu.b = 0x8;
        cpu.run_opcode(0x80);
        assert!(cpu.get_flag_h());
        Ok(())
    }

    #[test]
    fn test_h_flag_and_sub() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.a = 0xf8;
        cpu.b = 0x9;
        cpu.run_opcode(0x90);
        assert!(cpu.get_flag_h());
        assert_eq!(cpu.a, 0xef);
        cpu.run_opcode(0x90);
        assert!(!cpu.get_flag_h());
        assert_eq!(cpu.a, 0xe6);

        Ok(())
    }

    #[test]
    fn test_sbc() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.a = 0x8;
        cpu.b = 0x9;
        cpu.run_opcode(0x90);
        assert_eq!(cpu.a, 0xff);
        cpu.run_opcode(0x98);
        assert!(!cpu.get_flag_h());
        assert_eq!(cpu.a, 0xff-0xa);

        Ok(())
    }

    #[test]
    fn test_and_xor_or() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.a = 0xfe;
        cpu.b = 0xdd;
        cpu.run_opcode(0xa0);
        assert_eq!(cpu.a, 0xdc);
        cpu.run_opcode(0xa8);
        assert_eq!(cpu.a, 0x01);
        cpu.run_opcode(0xb0);
        assert_eq!(cpu.a, 0xdd);
        Ok(())
    }

    #[test]
    fn test_inc() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.run_opcode(0x04);
        assert_eq!(cpu.b, 0x2);
        cpu.a = 0xf;
        cpu.run_opcode(0x3c);
        assert!(cpu.get_flag_h());
        Ok(())
    }

    #[test]
    fn test_dec() -> Result<(), String> {
        let mut cpu = get_cpu();
        cpu.run_opcode(0x05);
        assert_eq!(cpu.b, 0x0);
        cpu.a = 0x10;
        cpu.run_opcode(0x3d);
        assert!(cpu.get_flag_h());
        Ok(())
    }

    #[test]
    fn test_daa() -> Result<(), String> {
        let mut cpu = get_cpu();
        //cpu.f = 0xd0;
        cpu.a = 0x7e;
        cpu.daa();
        assert_eq!(cpu.a, 0x84);
        assert!(!cpu.get_flag_c());
        cpu.f = 0xf0;
        cpu.a = 0x89;
        cpu.daa();
        assert_eq!(cpu.a, 0x23);
        cpu.f = 0xf0;
        cpu.a = 0x90;
        cpu.b = 0x09;
        cpu.run_opcode(0x90);
        cpu.daa();
        assert_eq!(cpu.a, 0x81);
        Ok(())
    }
}