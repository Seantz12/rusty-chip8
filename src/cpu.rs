use super::font::FONT_SET as FONT_SET;
use super::RomLoader;

pub struct Cpu {
    opcode: u16,
    v: [u8; super::REGISTER_COUNT], 
    i: u16, // range 0x000 - 0xFFF
    sound_timer: u8,
    delay_timer: u8,
    pc: u16, // range 0x000 - 0xFFF
    stack: [u8; super::STACK_SIZE],
    sp: u8, // stack pointer
    memory: [u8; super::RAM_SIZE],
    display: [[u8; super::WIDTH]; super::HEIGHT],
    keypad: [bool; super::KEYPAD_SIZE]
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut memory = [0u8; super::RAM_SIZE];
        for i in 0..FONT_SET.len() {
            memory[i] = FONT_SET[i];
        }
        Cpu {
            opcode: 0,
            v: [0; super::REGISTER_COUNT],
            i: super::INITIAL_PC,
            sound_timer: 0,
            delay_timer: 0,
            pc: super::INITIAL_PC,
            stack : [0; super::STACK_SIZE],
            sp: 0,
            memory: memory,
            display: [[0; super::WIDTH]; super::HEIGHT],
            keypad: [false; super::KEYPAD_SIZE]
        }
    }

    pub fn load_program(&mut self, rom_loader: &RomLoader) {
        let program = rom_loader.get_data();
        let length = rom_loader.get_length();
        for i in 0..length {
            // println!("testing byte: {}", program[i]); // DEBUG
            self.memory[i + super::INITIAL_PC as usize] = program[i];
        }
    }

    pub fn update_timer(&mut self) {
        if self.delay_timer > 0  {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0  {
            if self.sound_timer == 1  {
                println!("this would be a SOUND");
            }
            self.sound_timer -= 1;
        }
    }

    pub fn emulate_cycle(&mut self) {
        // fetch -> decode -> execute -> update -> repeat
        // opcodes are two BYTES long, so need to fetch current byte plus one more byte and encode that
        self.opcode = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16);
        match  self.opcode & 0xF000  {
            0xA000 => { // ANNN, set i to address NNN
                // println!("uh hi"); // DEBUG
                self.i = self.opcode & 0x0FFF;
            }
            _ => {
                println!("Unknown opcode: {}", self.opcode);
            }
        }
        self.update_timer();
        self.pc += 2;
    }
}