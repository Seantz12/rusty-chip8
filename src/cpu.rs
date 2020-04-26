extern crate rand;
extern crate device_query;

use rand::Rng;
use device_query::{DeviceQuery, DeviceState, Keycode};

use super::font::FONT_SET as FONT_SET;
use super::RomLoader;
use super::keys::convert_input;

const OPCODE_SIZE: u16 = 2;

pub struct DisplayData<'a> {
    pub display: &'a [[u8; super::WIDTH]; super::HEIGHT]
}

pub struct Cpu {
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
    keypad: [bool; super::KEYPAD_SIZE],
    device_state: DeviceState
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut memory = [0u8; super::RAM_SIZE];
        for i in 0..FONT_SET.len() {
            memory[i] = FONT_SET[i];
        }
        Cpu {
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
            keypad: [false; super::KEYPAD_SIZE],
            device_state: DeviceState::new()
        }
    }

    pub fn get_draw_flag(&self) -> bool {
        self.draw_flag.clone()
    }

    pub fn get_display(&self) -> DisplayData {
        DisplayData {
            display: &self.display
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

    pub fn get_input(&mut self) -> Option<u8> {
        self.keypad = [false; super::KEYPAD_SIZE];
        let keys:Vec<Keycode> = self.device_state.get_keys();
        for key in keys.iter() {
            match convert_input(key) {
                Some(keycode) => {
                    // self.v[reg_x as usize] = keycode;
                    if keycode == 0x10 {
                        std::process::exit(0);
                    }
                    self.keypad[keycode as usize] = true;
                    println!("keypad input");
                    for key in self.keypad.iter() {
                        print!("{}", key);
                    }
                    println!();
                    return Some(keycode);
                }
                None => {
                    println!("invalid key");
                }
            }
        }
        return None;
    }

    pub fn emulate_cycle(&mut self) {
        // fetch -> decode -> execute -> update -> repeat
        self.get_input();
        self.draw_flag = false;
        // opcodes are two BYTES long, so need to fetch current byte plus one more byte and encode that
        let opcode = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16);
        self.decode_opcode(opcode);
        // println!("OPCODE: {:04x?}", opcode); // DEBUG
        self.update_timer();
    }

    fn decode_opcode(&mut self, opcode: u16) {
        // possible patterns of opcode variables:
        // NNN: last three digits
        // kk: last two digits
        // X: second digit
        // Y: third digit
        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let n = (opcode & 0x000F) as u8;
        let digits = ((opcode & 0xF000) >> 12, (opcode & 0x0F00) >> 8,
                      (opcode & 0x00F0) >> 4,  (opcode & 0x000F));
        match digits {
            (0x0, 0x0, 0xE, 0x0) => self.execute_00E0(),
            (0x0, 0x0, 0xE, 0xE) => self.execute_00EE(),
            (0x1,   _,   _,   _) => self.execute_1NNN(nnn),
            (0x2,   _,   _,   _) => self.execute_2NNN(nnn),
            (0x3,   _,   _,   _) => self.execute_3XKK(x, kk),
            (0x4,   _,   _,   _) => self.execute_4XKK(x, kk),
            (0x5,   _,   _, 0x0) => self.execute_5XY0(x, y),
            (0x6,   _,   _,   _) => self.execute_6XKK(x, kk),
            (0x7,   _,   _,   _) => self.execute_7XKK(x, kk),
            (0x8,   _,   _, 0x0) => self.execute_8XY0(x, y),
            (0x8,   _,   _, 0x1) => self.execute_8XY1(x, y),
            (0x8,   _,   _, 0x2) => self.execute_8XY2(x, y),
            (0x8,   _,   _, 0x3) => self.execute_8XY3(x, y),
            (0x8,   _,   _, 0x4) => self.execute_8XY4(x, y),
            (0x8,   _,   _, 0x5) => self.execute_8XY5(x, y),
            (0x8,   _,   _, 0x6) => self.execute_8XY6(x, y),
            (0x8,   _,   _, 0x7) => self.execute_8XY7(x, y),
            (0x8,   _,   _, 0xE) => self.execute_8XYE(x, y),
            (0x9,   _,   _, 0x0) => self.execute_9XY0(x, y),
            (0xA,   _,   _,   _) => self.execute_ANNN(nnn),
            (0xB,   _,   _,   _) => self.execute_BNNN(nnn),
            (0xC,   _,   _,   _) => self.execute_CXKK(x, kk),
            (0xD,   _,   _,   _) => self.execute_DXYN(x, y, n),
            (0xE,   _, 0x9, 0xE) => self.execute_EX9E(x),
            (0xE,   _, 0xA, 0x1) => self.execute_EXA1(x),
            (0xF,   _, 0x0, 0x7) => self.execute_FX07(x),
            (0xF,   _, 0x0, 0xA) => self.execute_FX0A(x),
            (0xF,   _, 0x1, 0x5) => self.execute_FX15(x),
            (0xF,   _, 0x1, 0x8) => self.execute_FX18(x),
            (0xF,   _, 0x1, 0xE) => self.execute_FX1E(x),
            (0xF,   _, 0x2, 0x9) => self.execute_FX29(x),
            (0xF,   _, 0x3, 0x3) => self.execute_FX33(x),
            (0xF,   _, 0x5, 0x5) => self.execute_FX55(x),
            (0xF,   _, 0x6, 0x5) => self.execute_FX65(x),
            _ => panic!("Error: invalid opcode {:04x?}", opcode)
        }
     
    }

    fn next_instruction(&mut self) {
        self.pc += OPCODE_SIZE;
    }

    fn skip_instruction(&mut self) {
        self.pc += OPCODE_SIZE * 2;
    }

    fn jump_instruction(&mut self, location: u16) {
        self.pc = location;
    }

    fn skip_if_true(&mut self, condition: bool) {
        if condition {
            self.skip_instruction();
        } else {
            self.next_instruction();
        }
    }

    // Clear Screen
    fn execute_00E0(&mut self) {
        for y in 0..super::HEIGHT {
            for x in 0..super::WIDTH {
                self.display[y][x] = 0;
            }
        }
        self.draw_flag = true;
        self.next_instruction();
    }

    // Return from subroutine in stack
    fn execute_00EE(&mut self) {
        self.sp -= 1;
        self.jump_instruction(self.stack[self.sp as usize]);
    }

    // Jump to NNN
    fn execute_1NNN(&mut self, NNN: u16) {
        self.jump_instruction(NNN);
    }

    // Execute subroutine at NNN
    fn execute_2NNN(&mut self, NNN: u16) {
        self.stack[self.sp as usize] = self.pc + 2;
        self.sp += 1;
        self.jump_instruction(NNN);
    }

    // Compare VX to KK, if the same, skip next instruction
    fn execute_3XKK(&mut self, X: u8, KK: u8) {
        self.skip_if_true(self.v[X as usize] == KK);
    }

    // Compare VX to KK, if not the same, skip next instruction
    fn execute_4XKK(&mut self, X: u8, KK: u8) {
        self.skip_if_true(self.v[X as usize] != KK);
    }

    // Compare VX and VY, skip if they are the same
    fn execute_5XY0(&mut self, X: u8, Y: u8) {
       self.skip_if_true(self.v[X as usize] == self.v[Y as usize]); 
    }

    // Set VX = KK
    fn execute_6XKK(&mut self, X: u8, KK: u8) {
        self.v[X as usize] = KK;
        self.next_instruction();
    }

    // Set VX = VX + KK
    fn execute_7XKK(&mut self, X: u8, KK: u8) {
        if self.v[X as usize] > (255 - KK) {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.v[X as usize] = self.v[X as usize].wrapping_add(KK);
        self.next_instruction();
    }

    // Set VX = VY
    fn execute_8XY0(&mut self, X: u8, Y: u8) {
        self.v[X as usize] = self.v[Y as usize];
        self.next_instruction();
    }

    // Set VX = VX | VY
    fn execute_8XY1(&mut self, X: u8, Y: u8) {
        self.v[X as usize] |= self.v[Y as usize];
        self.next_instruction();
    }
    
    // Set VX &= VY
    fn execute_8XY2(&mut self, X: u8, Y: u8) {
        self.v[X as usize] &= self.v[Y as usize];
        self.next_instruction();
    }

    // Set VX ^= VY
    fn execute_8XY3(&mut self, X: u8, Y: u8) {
        self.v[X as usize] ^= self.v[Y as usize];
        self.next_instruction();
    }

    // Set VX += VY
    fn execute_8XY4(&mut self, X: u8, Y: u8) {
        let mut result: u16 = self.v[X as usize] as u16 + self.v[Y as usize] as u16;
        // set carry bit
        if result > 255 {
            self.v[0xF] = 1;
            result %= 256;
        } else {
            self.v[0xF] = 0;
        }
        self.v[X as usize] = result as u8; // note that by subtracting 256 we know that this will fit under 255
        self.next_instruction();
    }

    // Set VX -= VY
    fn execute_8XY5(&mut self, X: u8, Y: u8) {
        let value_x = self.v[X as usize];
        let value_y = self.v[Y as usize];
        // 0xF is NOT borrow
        if value_x > value_y {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.v[X as usize] = value_x.wrapping_sub(value_y);
        self.next_instruction();
    }

    // Set VX /= 2
    // Set VF = 1 if LSB of VX is 1
    fn execute_8XY6(&mut self, X: u8, Y: u8) {
        let lsb: u8 = self.v[X as usize] & 0x0F;
        if lsb == 1 {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.v[X as usize] /= 2;
        self.next_instruction();
    }

    // Set VX -= VY
    fn execute_8XY7(&mut self, X: u8, Y: u8) {
        let value_x = self.v[X as usize];
        let value_y = self.v[Y as usize];
        // 0xF is NOT borrow
        if value_y > value_x {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.v[X as usize] = value_y.wrapping_sub(value_x);
        self.next_instruction();
    }

    // Set VX *= 2
    // If MSB of VS is 1, set VF 1
    fn execute_8XYE(&mut self, X: u8, Y: u8) {
        let msb: u8 = (self.v[X as usize] & 0x80) >> 7;
        if msb == 1 {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.v[X as usize].wrapping_mul(2);
        self.next_instruction();
    }

    // Skip next instruction if VX != VY
    fn execute_9XY0(&mut self, X: u8, Y: u8) {
        self.skip_if_true(self.v[X as usize] != self.v[Y as usize]);
    }

    // Set I to value NNN
    fn execute_ANNN(&mut self, NNN: u16) {
        self.i = NNN;
        self.next_instruction();
    }

    // Set PC to NNN + V0
    fn execute_BNNN(&mut self, NNN: u16) {
        self.jump_instruction(self.v[0] as u16 + NNN);
    }

    // Set X to random byte & KK
    fn execute_CXKK(&mut self, X: u8, KK: u8) {
        let random_number: u8 = rand::thread_rng().gen_range(0, 255);
        self.v[X as usize] = random_number & KK as u8; 
        self.next_instruction();
    }

    // Draw at position VX and VY with width 8 pixels, height N pixels
    // VF is changed to 1 if any pixels are changed
    // Row of 8 pixels are read as bitcoded starting from memory location I
    fn execute_DXYN(&mut self, X: u8, Y: u8, N: u8) {
        let value_x: u8 = self.v[X as usize];
        let value_y: u8 = self.v[Y as usize];
        self.v[0xF] = 0; // default to 0
        // Taken from Starr Horne's implementation
        for byte in 0..N {
            let y = (self.v[Y as usize] + byte) as usize % super::HEIGHT;
            for bit in 0..8 {
                let x = (self.v[X as usize] as u16 + bit) as usize % super::WIDTH;
                let color = (self.memory[(self.i + byte as u16) as usize] >> (7 - bit)) & 1;
                self.v[0xF] |= color & self.display[y][x];
                self.display[y][x] ^= color;
            }
        }
        self.draw_flag = true;
        self.next_instruction();
    }

    // Skip next instruction if key at VX is pressed
    fn execute_EX9E(&mut self, X: u8) {
        let key: u8 = self.v[X as usize];
        self.skip_if_true(self.keypad[key as usize]);
    }

    // Skip next instruction if key at VX is not pressed
    fn execute_EXA1(&mut self, X: u8) {
        let key: u8 = self.v[X as usize];
        self.skip_if_true(!self.keypad[key as usize]);
    }

    // Set VX = delay timer value
    fn execute_FX07(&mut self, X: u8) {
        self.v[X as usize] = self.delay_timer;
        self.next_instruction();
    }

    // Wait for key press then store value in VX
    fn execute_FX0A(&mut self, X: u8) {
        let new_key_pressed = loop {
            if let Some(key) = self.get_input() {
                break key;
            }
        };
        self.v[X as usize] = new_key_pressed;
        self.next_instruction();
    }

    // Set delay timer to VX
    fn execute_FX15(&mut self, X: u8) {
        self.delay_timer = self.v[X as usize];
        self.next_instruction();
    }

    // Set sound timer to VX
    fn execute_FX18(&mut self, X: u8) {
        self.sound_timer = self.v[X as usize];
        self.next_instruction();
    }

    // Set I += VX
    fn execute_FX1E(&mut self, X: u8) {
        self.i += self.v[X as usize] as u16;
        self.next_instruction();
    }

    // Set I = location of sprite digit for digit at VX
    fn execute_FX29(&mut self, X: u8) {
        self.i = self.v[X as usize] as u16 * 5;
        self.next_instruction();
    }

    // Store binary decimal representation of VX starting at I
    fn execute_FX33(&mut self, X: u8) {
        let value_x = self.v[X as usize];
        self.memory[self.i as usize] = value_x / 100;
        self.memory[(self.i + 1) as usize] = (value_x % 100) / 10;
        self.memory[(self.i + 2) as usize] = value_x % 10;
        self.next_instruction();
    }

    // Store registers from V0 to VX into memory starting at I
    fn execute_FX55(&mut self, X: u8) {
        for register in 0..(X + 1) {
            self.memory[(self.i + register as u16) as usize] = self.v[register as usize];
        }
        self.next_instruction();
    }

    // Store registers from V0 to VX from memory starting at I
    fn execute_FX65(&mut self, X: u8) {
        for register in 0..(X + 1) {
            self.v[register as usize] = self.memory[(self.i + register as u16) as usize];
        }
        self.next_instruction();
    }
}