use std::fs;

pub struct RomLoader {
    data: [u8; super::ROM_SIZE],
    file_name: String
}

impl RomLoader {
    pub fn new(file_name: String) -> RomLoader{
        let mut data = [0u8; super::ROM_SIZE];
        match fs::read(&file_name) {
            Ok(bytes) => {
                println!("loading in file name...");
                for (i, byte) in bytes.iter().enumerate() {
                    data[i] = *byte;
                }
            }
            Err(error) => {
                panic!("There was something wrong!\n{}", error);
            }
        }
        RomLoader {
            data: data,
            file_name: file_name
        }
    }
}