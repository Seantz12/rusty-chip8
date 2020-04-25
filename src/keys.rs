extern crate device_query;
use device_query::Keycode;

pub fn convert_input(key: &Keycode) -> Option<u8> {
    match key {
        Keycode::Key1 => Some(0x0),
        Keycode::Key2 => Some(0x1),
        Keycode::Key3 => Some(0x2),
        Keycode::Key4 => Some(0x3),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0x7),
        Keycode::A => Some(0x8),
        Keycode::S => Some(0x9),
        Keycode::D => Some(0xA),
        Keycode::F => Some(0xB),
        Keycode::Z => Some(0xC),
        Keycode::X => Some(0xD),
        Keycode::C => Some(0xE),
        Keycode::V => Some(0xF),
        Keycode::Escape => Some(0x10),
        _   => None
    }
}