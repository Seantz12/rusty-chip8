struct Cpu {
    opcode: u16,
    v: [u8; 16],
    i: u16,
    sound_timer: u8,
    delay_timer: u8,
    pc: u16,
    sp: u8,
    memory: [u8; 4096]
}

fn main() {
    println!("Hello, world!");
}
