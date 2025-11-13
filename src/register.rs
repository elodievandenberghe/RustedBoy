pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}
#[derive(Copy, Clone)]
pub enum CpuFlags {
    Z = 0b10000000, //zero flag,
    N = 0b01000000, //subtraction flag,
    H = 0b00100000, //half carry flag
    C = 0b00010000, //carry flag
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0x01,
            f: CpuFlags::Z as u8 | CpuFlags::N as u8 | CpuFlags::H as u8 | CpuFlags::C as u8,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            pc: 0x0100,
            sp: 0xFFFE,
        }
    }

    pub fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | ((self.f & 0xF0) as u16)
    }

    pub fn get_bc(&self) -> u16 {
        return ((self.b as u16) << 8) | (self.c as u16);
    }
    pub fn get_de(&self) -> u16 {
        return ((self.d as u16) << 8) | (self.e as u16);
    }
    pub fn get_hl(&self) -> u16 {
        return ((self.h as u16) << 8) | (self.l as u16);
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = (value & 0x00F0) as u8;
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }
    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }
    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
    }
    pub fn get_flag(&self, flag: CpuFlags) -> bool {
        let mask = flag as u8;
        self.f & mask > 0
    }
    pub fn set_flag(&mut self, flag: CpuFlags, set: bool) {
        let mask = flag as u8;
        if set {
            self.f |= mask;
        } else {
            self.f &= !mask
        }
    }
    pub fn increment_pc(&mut self, value: u16) {
        self.pc = self.pc.wrapping_add(value);
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registers_initial_values() {
        let regs = Registers::new();
        assert_eq!(regs.a, 0x01);
        assert_eq!(regs.b, 0x00);
        assert_eq!(regs.c, 0x13);
        assert_eq!(regs.d, 0x00);
        assert_eq!(regs.e, 0xD8);
        assert_eq!(regs.h, 0x01);
        assert_eq!(regs.l, 0x4D);
        assert_eq!(regs.pc, 0x0100);
        assert_eq!(regs.sp, 0xFFFE);
        // Flags should be all set initially (Z, N, H, C)
        assert_eq!(regs.f & 0xF0, 0xF0);
    }

    #[test]
    fn test_get_set_af() {
        let mut regs = Registers::new();
        regs.set_af(0x1230);
        assert_eq!(regs.a, 0x12);
        assert_eq!(regs.f, 0x30);
        assert_eq!(regs.get_af(), 0x1230);
    }

    #[test]
    fn test_get_set_bc() {
        let mut regs = Registers::new();
        regs.set_bc(0xBEEF);
        assert_eq!(regs.b, 0xBE);
        assert_eq!(regs.c, 0xEF);
        assert_eq!(regs.get_bc(), 0xBEEF);
    }

    #[test]
    fn test_get_set_de() {
        let mut regs = Registers::new();
        regs.set_de(0xDEAD);
        assert_eq!(regs.d, 0xDE);
        assert_eq!(regs.e, 0xAD);
        assert_eq!(regs.get_de(), 0xDEAD);
    }

    #[test]
    fn test_get_set_hl() {
        let mut regs = Registers::new();
        regs.set_hl(0xCAFE);
        assert_eq!(regs.h, 0xCA);
        assert_eq!(regs.l, 0xFE);
        assert_eq!(regs.get_hl(), 0xCAFE);
    }

    #[test]
    fn test_get_flag() {
        let mut regs = Registers::new();
        regs.f = CpuFlags::Z as u8 | CpuFlags::H as u8;
        assert!(regs.get_flag(CpuFlags::Z));
        assert!(regs.get_flag(CpuFlags::H));
        assert!(!regs.get_flag(CpuFlags::N));
        assert!(!regs.get_flag(CpuFlags::C));
    }

    #[test]
    fn test_set_flag() {
        let mut regs = Registers::new();

        // Clear all flags first
        regs.f = 0x00;

        // Set Z and H flags
        regs.set_flag(CpuFlags::Z, true);
        regs.set_flag(CpuFlags::H, true);

        // Check that only Z and H are set
        assert!(regs.get_flag(CpuFlags::Z));
        assert!(regs.get_flag(CpuFlags::H));
        assert!(!regs.get_flag(CpuFlags::N));
        assert!(!regs.get_flag(CpuFlags::C));

        // Clear H flag
        regs.set_flag(CpuFlags::H, false);
        assert!(regs.get_flag(CpuFlags::Z));
        assert!(!regs.get_flag(CpuFlags::H));
    }
}
