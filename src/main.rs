mod cpu;
mod font;
mod rom_loader;

use cpu::Cpu;
use rom_loader::RomLoader;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
const KEYPAD_SIZE: usize = 16;
const INITIAL_PC: u16 = 0x200;
const ROM_SIZE: usize = 4096; // completely arbitrary number

fn main() {
    // TODO: setup graphics and input and sound
    let args: Vec<String> = std::env::args().collect();
    let rom_loader = RomLoader::new(args[1].clone());
    let mut cpu = Cpu::new();
    cpu.load_program(&rom_loader);
    loop {
        cpu.emulate_cycle();
    }
}
