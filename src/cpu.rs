pub struct Cpu {
    opcode: u16,
    v: [u8; 16], // array syntax: [type, length]
    i: u16,
    sound_timer: u8,
    delay_timer: u8,
    pc: u16,
    sp: u8,
    memory: [u8; 4096]
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            opcode: 0,
            v: [0; 16],
            i: 0x200,
            sound_timer: 0,
            delay_timer: 0,
            pc: 0x200,
            sp: 0,
            memory: [0; 4096]
        }
    }
}