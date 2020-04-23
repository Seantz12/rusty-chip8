use std::fs;

pub struct RomLoader {
    data: [u8; super::ROM_SIZE],
    file_name: String,
    length: usize
}

impl RomLoader {
    pub fn new(file_name: String) -> RomLoader{
        let mut data = [0u8; super::ROM_SIZE];
        let mut length = 0usize;
        match fs::read(&file_name) {
            Ok(bytes) => {
                // println!("loading in file name..."); // DEBUG
                for (i, byte) in bytes.iter().enumerate() {
                    // println!("byte: {}", byte); // DEBUG
                    data[i] = *byte;
                    length += 1;
                }
            }
            Err(error) => {
                panic!("There was something wrong!\n{}", error);
            }
        }
        RomLoader {
            data: data,
            file_name: file_name,
            length: length
        }
    }

    pub fn get_data(&self) -> [u8; super::ROM_SIZE] {
        self.data
    }

    pub fn get_length(&self) -> usize {
        self.length
    }
}