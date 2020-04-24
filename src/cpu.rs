extern crate rand;
extern crate device_query;

use rand::Rng;
use device_query::{DeviceQuery, DeviceState, Keycode};

use super::font::FONT_SET as FONT_SET;
use super::RomLoader;
use super::keys::convert_input;

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
    draw_flag: bool,
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
            draw_flag: false,
            keypad: [false; super::KEYPAD_SIZE]
        }
    }

    pub fn get_draw_flag(&self) -> bool {
        self.draw_flag.clone()
    }

    pub fn get_display(&self) -> [[u8; super::WIDTH]; super::HEIGHT] {
        self.display.clone()
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
        self.draw_flag = false;
        self.opcode = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16);
        println!("OPCODE: {:04x?}", self.opcode); // DEBUG
        match  self.opcode & 0xF000  {
            0x0000 => {
                match self.opcode & 0x000F {
                    0x0000 => { // 0x00E0, clear screen
                        for y in 0..super::HEIGHT {
                            for x in 0..super::WIDTH {
                                self.display[y][x] = 0;
                            }
                        }
                        self.draw_flag = true;
                        self.pc += 2;
                    }
                    0x000E => { // 0x00EE, returns from subroutine
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                    }
                    _ => {
                        println!("Unknown opcode 1: {:x?}", self.opcode);
                        panic!();
                        // self.pc += 2;
                    }
                }
            }
            0x1000 => { // 0x1NNN, jump to NNN
                // println!("pc before: {:x?}", self.pc);
                self.pc = self.opcode & 0x0FFF;
                // self.pc += 2;
                // println!("pc after: {:x?}", self.pc);
            }
            0x2000 => { // 0x2NNN, execute subroutine at NNN
                self.stack[self.sp as usize] = self.pc + 2;
                self.sp += 1;
                self.pc = self.opcode & 0x0FFF;
            }
            0x3000 => { // 0x3XKK, compare VX to KK, if the same skip next instruction
                let reg_x: u16 = (self.opcode & 0x0F00) >> 8;
                let value_x: u8 = self.v[reg_x as usize];
                println!("hol up reg {}", reg_x);
                println!("hol up {}", value_x);
                let kk: u16 = self.opcode & 0x00FF;
                println!("hol up kk {:x}", kk);
                if value_x == kk as u8 {
                    println!("skip!");
                    self.pc += 4;
                } else {
                    println!("no skip!");
                    self.pc += 2;
                }
            }
            0x4000 => { // 0x4XKK, compare VX to KK if different skip next instruction
                let reg_x: u16 = (self.opcode & 0x0F00) >> 8;
                let value_x: u8 = self.v[reg_x as usize];
                let kk: u16 = self.opcode & 0x00FF;
                if value_x != kk as u8 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x5000 => { // 0x5XY0, skip next instruction if VX == VY
                let reg_x: u16 = (self.opcode & 0x0F00) >> 8;
                let value_x: u8 = self.v[reg_x as usize];
                let reg_y: u16 = (self.opcode & 0x00F0) >> 4;
                let value_y: u8 = self.v[reg_y as usize];
                if value_x == value_y {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x6000 => { // 0x6XKK, set VX = KK
                let reg_x: u16 = (self.opcode & 0x0F00) >> 8;
                let kk: u16 = self.opcode & 0x00FF;
                self.v[reg_x as usize] = kk as u8;
                self.pc += 2;
            }
            0x7000 => { // 0x7xkk, VX = VX + KK
                let reg_x: u16 = (self.opcode & 0x0F00) >> 8;
                let kk: u16 = self.opcode & 0x00FF;
                let mut value: u16 = self.v[reg_x as usize] as u16 + kk;
                println!("7 reg {}", reg_x);
                println!("{}", value);
                if value > 255 {
                    self.v[0xF] = 1; // set carry
                    value -= 256;
                } else {
                    self.v[0xF] = 0;
                }
                self.v[reg_x as usize] = value as u8;
                self.pc += 2;
            }
            0x8000 => {
                let reg_x: u16 = (self.opcode & 0x0F00) >> 8;
                let value_x: u8 = self.v[reg_x as usize];
                let reg_y: u16 = (self.opcode & 0x00F0) >> 4;
                let value_y: u8 = self.v[reg_y as usize];
                match self.opcode & 0x000F {
                    0x0000 => { // 0x8XY0, store value of VY to VX
                        self.v[reg_x as usize] = value_y;
                        self.pc += 2;
                    }
                    0x0001 => { // 0x8XY1, VX = VX | VY
                        self.v[reg_x as usize] |= value_y;
                        self.pc += 2;
                    }
                    0x0002 => { // 0x8XY2, VX &= VY
                        self.v[reg_x as usize] &= value_y;
                        self.pc += 2;
                    }
                    0x0003 => { // 0x8XY3, VX ^= VY
                        self.v[reg_x as usize] ^= value_y;
                        self.pc += 2;
                    }
                    0x0004 => { // 0x8XY4, add VY to VX
                        let mut result: u16 = value_x as u16 + value_y as u16;
                        // set carry bit
                        if result > 255 {
                            self.v[0xF] = 1;
                            result -= 256;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.v[reg_x as usize] = result as u8; // note that by subtracting 256 we know that this will fit under 255
                        self.pc += 2;
                    }
                    0x0005 => { // 0x8XY5, sub VY from VX
                        // 0xF is NOT borrow
                        if value_x > value_y {
                            // self.v[reg_x as usize] = value_x - value_y;
                            self.v[0xF] = 1;
                        } else {
                            // self.v[reg_x as usize] = 255 - (value_y - value_x);
                            // diff = value_y - value_x;
                            self.v[0xF] = 0;
                        }
                        self.v[reg_x as usize] = value_x.wrapping_sub(value_y);
                        self.pc += 2;
                    }
                    0x0006 => { // 0x8XY6 if VX LSB is 1, VF = 1. VX /= 2
                        let lsb: u8 = value_x & 0x0F;
                        if lsb == 1 {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.v[reg_x as usize] /= 2;
                        self.pc += 2;
                    }
                    0x0007 => { // 0x8XY7, VX = VY - VX, VF is NOT borrow
                        if value_y > value_x {
                            // self.v[reg_x as usize] = value_y - value_x;
                            self.v[0xF] = 1;
                        } else {
                            // self.v[reg_x as usize] = 255 - (value_x - value_y);
                            // diff = value_y - value_x;
                            self.v[0xF] = 0;
                        }
                        self.v[reg_x as usize] = value_y.wrapping_sub(value_x);
                        self.pc += 2
                    }
                    0x000E => { // 0x8XYE, if MSB VX = 1, then VF is set to 1, otherwise 0 VX *= 2
                        let msb: u8 = (value_x & 0x80) >> 7;
                        if msb == 1 {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.v[reg_x as usize] = self.v[reg_x as usize].wrapping_mul(2);
                        self.pc += 2;
                    }
                    _ => {
                        println!("Unknown opcode 2: {:x?}", self.opcode);
                        panic!();
                        // self.pc += 2;
                    }
                }
            }
            0x9000 => { // 0x9XY0, skip next instruction if VX != VY
                let reg_x: u16 = (self.opcode & 0x0F00) >> 8;
                let value_x: u8 = self.v[reg_x as usize];
                let reg_y: u16 = (self.opcode & 0x00F0) >> 4;
                let value_y: u8 = self.v[reg_y as usize];
                if value_x != value_y {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0xA000 => { // 0xANNN, set i to address NNN
                // println!("uh hi"); // DEBUG
                self.i = self.opcode & 0x0FFF;
                self.pc += 2;
            }
            0xB000 => { // 0xBNNN, set pc to address NNN + V0
                let nnn: u16 = self.opcode & 0x0FFF;
                self.pc = self.v[0] as u16 + nnn;
            }
            0xC000 => { // 0xCXKK, set X to random byte & KK
                let reg_x: u16 = (self.opcode & 0x0F00) >> 8;
                let random_number: u8 = rand::thread_rng().gen_range(0, 255);
                let kk: u16 = self.opcode & 0x00FF;
                self.v[reg_x as usize] = random_number & kk as u8; 
                self.pc += 2;
            }
            0xD000 => { // 0xDXYN, draw at position (VX, VY) with width 8 pixels and height N pixels
                // VF is changed to 1 if any pixels are changed
                // Row 8 pixels are read as bitcoded starting from memory location i
                let reg_x: u16 = (self.opcode & 0x0F00) >> 8;
                let value_x: u8 = self.v[reg_x as usize];
                let reg_y: u16 = (self.opcode & 0x00F0) >> 4;
                let value_y: u8 = self.v[reg_y as usize];
                let n: u16 = self.opcode & 0x000F;
                let mut pixels: u8;
                self.v[0xF] = 0; // default to 0
                // Taken from Starr Horne's implementation
                for byte in 0..n {
                    let y = (self.v[reg_y as usize] as u16 + byte) as usize % super::HEIGHT;
                    for bit in 0..8 {
                        let x = (self.v[reg_x as usize] as u16 + bit) as usize % super::WIDTH;
                        let color = (self.memory[(self.i + byte) as usize] >> (7 - bit)) & 1;
                        self.v[0xF] |= color & self.display[y][x];
                        self.display[y][x] ^= color;
                    }
                }
                // for y in 0..n {
                //     pixels = self.memory[(self.i + y) as usize];
                //     for x in 0..8 {
                //         // scan through pixels one at a time
                //         // println!("hey i'm supposed to be drawing 11?"); // DEBUG
                //         let mut pos_x: usize;
                //         let mut pos_y: usize;
                //         if value_x + x > 63  {
                //             pos_x = (value_x + x % 64) as usize;
                //         } else {
                //             pos_x = (value_x + x) as usize;
                //         }
                //         if value_y as u16 + y > 31  {
                //             pos_y = (value_y as u16 + y % 32) as usize;
                //         } else {
                //             pos_y = (value_y as u16 + y) as usize;
                //         }
                //         self.display[pos_y][pos_x] ^= 1;
                //         if pixels & (0x80 >> x) != 0 {
                //             // set draw to true
                //             if self.display[pos_y][pos_x] == 1 {
                //                 // collision detected
                //                 self.v[0xF] = 1;
                //             }
                //         }
                //     }
                // }
                self.draw_flag = true;
                self.pc += 2;
            }
            0xE000 => {
                match self.opcode & 0x00FF {
                    0x009E => { // 0xEX9E, skip next instruction if key in VX is pressed
                        let reg_x: u16 = (self.opcode & 0x0F00) >> 8;
                        let key: u8 = self.v[reg_x as usize];
                        if self.keypad[key as usize] {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    0x00A1 => { // 0xEXA1, skip next instruction if key in VX is not pressed
                        let reg_x: u16 = (self.opcode & 0x0F00) >> 8;
                        let key: u8 = self.v[reg_x as usize];
                        if !self.keypad[key as usize] {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        println!("Unknown opcode 3: {:x?}", self.opcode);
                        panic!();
                        // self.pc += 2;
                    }
                }
            }
            0xF000 => {
                let reg_x: u16 = (self.opcode & 0x0F00) >> 8;
                let value_x: u8 = self.v[reg_x as usize];
                match self.opcode & 0x00FF {
                    0x0007 => { // 0xFX07, Set VX = delay timer value
                        self.v[reg_x as usize] = self.delay_timer;
                        self.pc += 2;
                    }
                    0x000A => { // 0xFX0A, wait for key press then store that value into X
                        let device_state = DeviceState::new();
                        loop {
                            let keys:Vec<Keycode> = device_state.get_keys();
                            let mut exit_flag = false;
                            for key in keys.iter() {
                                match convert_input(key) {
                                    Some(keycode) => {
                                        self.v[reg_x as usize] = keycode;
                                        exit_flag = true;
                                        break;
                                    }
                                    None => {
                                        println!("invalid key");
                                    }
                                }
                            }
                            if exit_flag {
                                break;
                            }
                            // do something with them
                        }
                        self.pc += 2;
                    }
                    0x0015 => { // 0xFX15, set DT to VX
                        self.delay_timer = value_x;
                        self.pc += 2;
                    }
                    0x0018 => { // 0xFX18, set ST to VX
                        self.sound_timer = value_x;
                        self.pc += 2;
                    }
                    0x001E => { // 0xFX1E, I += VX
                        self.i += value_x as u16;
                        self.pc += 2;
                    }
                    0x0029 => { // 0xFX29, I = location of sprite for digit VX
                        self.i = value_x as u16 * 5;
                        self.pc += 2;
                    }
                    0x0033 => { // 0xFX33, store binary decimal representation of VX at self.i, self.i+1, and self.i+2
                        self.memory[self.i as usize] = value_x / 100;
                        self.memory[(self.i + 1) as usize] = (value_x % 100) / 10;
                        self.memory[(self.i + 2) as usize] = value_x % 10;
                        self.pc += 2;
                    }
                    0x0055 => { // 0xFX55, Store registers from V0 to VX into I
                        for register in 0..(reg_x + 1) {
                            self.memory[(self.i + register) as usize] = self.v[register as usize];
                        }
                        self.pc += 2;
                    }
                    0x0065 => { // 0xFX65 store registers from V0 to VX from I
                        for register in 0..(reg_x + 1) {
                            self.v[register as usize] = self.memory[(self.i + register) as usize];
                        }
                        self.pc += 2;
                    }
                    _ => {
                        println!("Unknown opcode 4: {:x?}", self.opcode);
                        panic!();
                        // self.pc += 2;
                    }
                }
            }
            _ => {
                println!("Unknown opcode 5: {:x?}", self.opcode);
                panic!();
                        // self.pc += 2;
            }
        }
        self.update_timer();
    }
}