use super::font::FONT_SET as FONT_SET;
use super::RomLoader;

pub struct Cpu {
    opcode: u16,
    v: [u8; super::REGISTER_COUNT], 
    i: u16, // range 0x000 - 0xFFF
    sound_timer: u8,
    delay_timer: u8,
    pc: u16, // range 0x000 - 0xFFF
    stack: [u16; super::STACK_SIZE],
    sp: u16, // stack pointer
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
            // 0xF is the carry over register
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
            0x0000 => {
                match self.opcode & 0x000F {
                    0x0000 => { // 0x00E0, clear screen
                        // fill in
                    }
                    0x000E => { // 0x00EE, returns from subroutine
                        // fill in
                        // likely grab the PC from the stack
                    }
                    _ => {
                        println!("Unknown opcode: {}", self.opcode);
                    }
                }
            }
            0x2000 => { // 2NNN, execute subroutine at NNN
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = self.opcode & 0x0FFF;
            }
            0x8000 => {
                match self.opcode & 0x000F {
                    0x0004 => { // 0x8XY4, add VY to VX
                        let y: u16 = (self.opcode & 0x00F0) >> 4;
                        let x: u16 = (self.opcode & 0x0F00) >> 8;
                        let mut result: u16 = (self.v[x as usize] + self.v[y as usize]) as u16;
                        // set carry bit
                        if result > 255 {
                            self.v[0xF] = 1;
                            result -= 256;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.v[x as usize] = result as u8; // note that by subtracting 256 we know that this will fit under 255
                    }
                    _ => {
                        println!("Unknown opcode: {}", self.opcode);
                    }
                }
            }
            0xA000 => { // ANNN, set i to address NNN
                // println!("uh hi"); // DEBUG
                self.i = self.opcode & 0x0FFF;
            }
            0xF000 => {
                match self.opcode & 0x00FF {
                    0x0033 => { // 0xFX33, store binary decimal representation of VX at self.i, self.i+1, and self.i+2
                        let x: u16 = (self.opcode & 0x0F00) >> 8;
                        let value_x: u8 = self.v[x as usize];
                        self.memory[self.i as usize] = value_x / 100;
                        self.memory[(self.i + 1) as usize] = (value_x / 10) % 10;
                        self.memory[(self.i + 2) as usize] = (value_x % 100) % 10;
                    }
                    _ => {
                        println!("Unknown opcode: {}", self.opcode);
                    }
                }
            }
            _ => {
                println!("Unknown opcode: {}", self.opcode);
            }
        }
        self.update_timer();
        self.pc += 2;
    }
}