use std::fs::File;
use std::io::Read;
pub struct MemoryBus {
    pub data: [u8; 0x10000],
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus { data: [0; 0x10000] }
    }
    pub fn read_data(&self, address: u16) -> u8 {
        self.data[address as usize]
    }
    pub fn write_data(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }

    pub fn extract_rom(&mut self, path: String) -> std::io::Result<Vec<u8>> {
        let mut file = File::open(path)?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let len = buffer.len();

        self.data[0..len].copy_from_slice(&buffer[0..len]);

        Ok(buffer)
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_read_and_write_data() {
        let mut bus = MemoryBus::new();
        bus.write_data(0x1234, 0xAB);
        let value = bus.read_data(0x1234);

        assert_eq!(value, 0xAB)
    }
    #[test]
    fn test_read_rom() {
        let mut bus = MemoryBus::new();

        bus.extract_rom(String::from("Tetris (World).gb"))
            .expect("file not found");

        let mut file = File::open(String::from("Tetris (World).gb")).expect("file not found");

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("Failed to read ROM file");

        assert_eq!(bus.data[0..buffer.len()], buffer[..])
    }
}
