use std::fs;

pub struct Cardridge {
    pub memory: Vec<u8>,
}

pub fn main() {
    let path = fs::read("test.gb").unwrap();
    for line in path {
        println!("{}", line);
    }
}