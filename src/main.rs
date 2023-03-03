mod cpu;
mod cardridge;
mod memory_map;

fn main() {
    let vec1:Vec<u8> = vec![0x40, 0x41, 0x42];
    let cardridge = cardridge::Cardridge{
        memory: vec1,
    };
    let mut cpu = cpu::Cpu::new(cardridge);
    
    cpu.start_cycle();
    println!("{}", cpu.a());
}
