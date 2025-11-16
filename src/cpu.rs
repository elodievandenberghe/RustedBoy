use crate::memorybus::MemoryBus;
use crate::register::CpuFlags;
use crate::register::Registers;
pub struct Cpu {
    registers: Registers,
    pub bus: MemoryBus,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            bus: MemoryBus::new(),
        }
    }
    pub fn step(&mut self) {
        let opcode = self.bus.read_data(self.registers.pc);
        let bytes = self.execute(opcode);
        self.registers.increment_pc(bytes);
    }

    fn alu_add(&mut self, value: u8) {
        let c = if self.registers.get_flag(CpuFlags::C) == true {
            1
        } else {
            0
        };
        let a = self.registers.a;
        let r = a.wrapping_add(value).wrapping_add(c);
        self.registers.set_flag(CpuFlags::Z, r == 0);
        self.registers
            .set_flag(CpuFlags::H, ((a & 0xF) + (value & 0xF) + c) > 0xF);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers
            .set_flag(CpuFlags::C, (a as u16) + (value as u16) + (c as u16) > 0xFF);

        self.registers.a = r;
    }

    fn alu_add_16(&mut self, value: u16) {
        let hl = self.registers.hl();
        let r = self.registers.hl().wrapping_add(value);
        self.registers
            .set_flag(CpuFlags::H, ((hl & 0x0FFF) + (value & 0x0FFF)) > 0x0FFF);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers
            .set_flag(CpuFlags::C, (hl as u32) + (value as u32) > 0xFFFF);
        self.registers.set_hl(r);
    }

    fn alu_sub(&mut self, value: u8) {
        let c = if self.registers.get_flag(CpuFlags::C) == true {
            1
        } else {
            0
        };
        let a = self.registers.a;
        let r = a.wrapping_sub(value).wrapping_sub(c);
        self.registers.set_flag(CpuFlags::Z, r == 0);
        self.registers
            .set_flag(CpuFlags::H, (a & 0xF) < (value & 0xF) + c);
        self.registers.set_flag(CpuFlags::N, true);
        self.registers
            .set_flag(CpuFlags::C, (a as u16) < (value as u16 + c as u16));
        self.registers.a = r;
    }

    fn alu_rlc(&mut self) {
        let result = self.registers.a.rotate_left(1);
        self.registers
            .set_flag(CpuFlags::C, self.registers.a & 0x80 == 0x80);
        self.registers.set_flag(CpuFlags::Z, result == 0);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.a = result;
    }

    fn decrement_reg(&mut self, reg: u8) -> u8 {
        let result = reg.wrapping_sub(1);
        self.registers.set_flag(CpuFlags::Z, result == 0);
        self.registers.set_flag(CpuFlags::H, (reg & 0x0F) == 0);
        self.registers.set_flag(CpuFlags::N, true);
        result
    }
    fn increment_reg(&mut self, reg: u8) -> u8 {
        let result = reg.wrapping_add(1);
        self.registers.set_flag(CpuFlags::Z, result == 0);
        self.registers
            .set_flag(CpuFlags::H, (reg & 0x0F) + 1 > 0x0F);
        self.registers.set_flag(CpuFlags::N, true);
        result
    }

    pub fn execute(&mut self, opcode: u8) -> u16 {
        match opcode {
            0x00 => {
                /*no operation :3*/
                1
            }
            0x01 => {
                /*LD BC, d16*/

                let upper_nibble = self.bus.read_data(self.registers.pc.wrapping_add(2));
                let lower_nibble = self.bus.read_data(self.registers.pc.wrapping_add(1));

                self.registers
                    .set_bc(((upper_nibble as u16) << 8) | (lower_nibble as u16));
                3
            }
            0x03 => {
                /*INC BC*/
                self.registers
                    .set_bc(self.registers.get_bc().wrapping_add(1));
                1
            }
            0x04 => {
                //INC B
                self.registers.b = self.registers.b.wrapping_add(1);
                1
            }
            0x05 => {
                //DEC B
                self.registers.b = self.registers.b.wrapping_sub(1);
                1
            }
            0x07 => {
                //RLCA
                self.alu_rlc();
                1
            }
            0x08 => {
                //LD (n16), SP
                let addr = (self.bus.read_data(self.registers.pc.wrapping_add(1)) as u16) << 8
                    | (self.bus.read_data(self.registers.pc.wrapping_add(2)) as u16);
                self.bus.write_data(addr, (self.registers.sp as u8) & 0xFF);
                self.bus
                    .write_data(addr.wrapping_add(1), (self.registers.sp as u8) >> 8);
                3
            }
            0x09 => {
                //ADD HL, BC
                self.alu_add_16(self.registers.bc());
                1
            }
            0x0A => {
                //LD A, BC
                self.registers.a = self.bus.read_data(self.registers.bc());
                1
            }
            0x0B => {
                //DEC BC
                self.registers.set_bc(self.registers.bc().wrapping_sub(1));
                1
            }
            0x0C => {
                //INC C
                self.registers.c = self.increment_reg(self.registers.c);
                1
            }
            0x0D => {
                //DEC C
                self.registers.c = self.decrement_reg(self.registers.c);
                1
            }
            0x02 => {
                /* LD (BC), A - Store the value of register A into the memory address in BC */
                self.bus
                    .write_data(self.registers.get_bc(), self.registers.a);
                1
            }
            0x78 => {
                /* LD A, B - Load the value of register B into register A */
                self.registers.a = self.registers.b;
                1
            }
            0x06 => {
                /* LD B, d8 - Load an immediate 8-bit value into register B */
                self.registers.b = self.bus.read_data(self.registers.pc.wrapping_add(1));
                2
            }
            0x3E => {
                /*LD A, d8, load 8-bit immediate value into a*/
                self.registers.a = self.bus.read_data(self.registers.pc.wrapping_add(1));
                2
            }
            0x2E => {
                /*LD A, d8, load 8-bit immediate value into l*/
                self.registers.l = self.bus.read_data(self.registers.pc.wrapping_add(1));
                2
            }
            0x1E => {
                /*LD A, d8, load 8-bit immediate value into e*/
                self.registers.e = self.bus.read_data(self.registers.pc.wrapping_add(1));
                2
            }
            0x0E => {
                /*LD A, d8, load 8-bit immediate value into c*/
                self.registers.c = self.bus.read_data(self.registers.pc.wrapping_add(1));
                2
            }
            0x80 => {
                /*ADD A, B, Add contents of register B to contents of register A, store result in A*/
                self.alu_add(self.registers.b);
                1
            }
            0x81 => {
                /*ADD A, C, Add contents of register B to contents of register A, store result in A*/
                self.alu_add(self.registers.c);
                1
            }
            0x82 => {
                /*ADD A, D, Add contents of register B to contents of register A, store result in A*/
                self.alu_add(self.registers.d);
                1
            }
            0x83 => {
                /*ADD A, E, Add contents of register B to contents of register A, store result in A*/
                self.alu_add(self.registers.e);
                1
            }
            0x84 => {
                /* ADD A, H */
                self.alu_add(self.registers.h);
                1
            }
            0x85 => {
                /* ADD A, L */
                self.alu_add(self.registers.l);
                1
            }
            0x86 => {
                /* ADD A, (HL) */
                let value = self.bus.read_data(self.registers.get_hl());
                self.alu_add(value);
                1
            }
            0x87 => {
                /* ADD A, A */
                self.alu_add(self.registers.a);
                1
            }
            0x90 => {
                /*SUB B*/
                self.alu_sub(self.registers.b);
                1
            }
            0x91 => {
                // SUB C
                self.alu_sub(self.registers.c);
                1
            }
            0x92 => {
                // SUB D
                self.alu_sub(self.registers.d);
                1
            }
            0x93 => {
                // SUB E
                self.alu_sub(self.registers.e);
                1
            }
            0x94 => {
                // SUB H
                self.alu_sub(self.registers.h);
                1
            }
            0x95 => {
                // SUB L
                self.alu_sub(self.registers.l);
                1
            }
            0x96 => {
                // SUB (HL)
                let value = self.bus.read_data(self.registers.get_hl());
                self.alu_sub(value);
                1
            }
            0x97 => {
                // SUB A
                self.alu_sub(self.registers.a);
                1
            }
            _ => {
                panic!("Unimplemented instruction");
            }
        }
    }
}
#[cfg(test)]
mod cpu_tests {
    use super::*;

    #[test]
    fn test_nop_instruction() {
        let mut cpu = Cpu::new();
        let pc_before = cpu.registers.pc;
        cpu.bus.write_data(pc_before, 0x00); // NOP
        cpu.step();
        assert_eq!(cpu.registers.pc, pc_before + 1);
    }
    #[test]
    fn test_ld_a_to_b_instruction() {
        let mut cpu = Cpu::new();
        cpu.registers.b = 0x42;
        cpu.registers.a = 0x51;
        cpu.bus.write_data(cpu.registers.pc, 0x78);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x42)
    }
    #[test]
    fn test_ld_value_into_b() {
        let mut cpu = Cpu::new();
        cpu.registers.b = 0x42;
        cpu.bus.write_data(cpu.registers.pc, 0x06);
        cpu.bus.write_data(cpu.registers.pc + 1, 0xF);
        cpu.step();
        assert_eq!(cpu.registers.b, 0xF)
    }
    #[test]
    fn test_ld_a_into_address_space_bc() {
        let mut cpu = Cpu::new();
        cpu.registers.set_bc(0xFF);
        cpu.bus.write_data(0xFF, 0x1);
        cpu.bus.write_data(cpu.registers.pc, 0x02);
        cpu.step();
        assert_eq!(cpu.registers.a, cpu.bus.read_data(cpu.registers.get_bc()))
    }
    #[test]
    fn test_add_b_to_a() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x01;
        cpu.registers.b = 0x42;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x80);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x01 + 0x42);
        assert_eq!(cpu.registers.f, 0x0);
    }
    #[test]
    fn test_add_b_to_a_hc_flag() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0xF;
        cpu.registers.b = 0xF;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x80);
        cpu.step();
        assert_eq!(cpu.registers.a, 0xF + 0xF);
        assert_eq!(cpu.registers.f, CpuFlags::H as u8);
    }
    #[test]
    fn test_add_b_to_a_c_flag() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0xFF;
        cpu.registers.b = 0xFF;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x80);
        cpu.step();
        assert_eq!(cpu.registers.a, 0xFE);
        assert_eq!(cpu.registers.get_flag(CpuFlags::C), true);
        assert_eq!(cpu.registers.get_flag(CpuFlags::H), true);
    } // ADD A, C
    #[test]
    fn test_add_c_to_a() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x10;
        cpu.registers.c = 0x20;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x81);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x30);
        assert_eq!(cpu.registers.f, 0x0);
    }

    #[test]
    fn test_add_c_to_a_hc_flag() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0xF;
        cpu.registers.c = 0x1;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x81);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x10);
        assert_eq!(cpu.registers.f, 0b00100000);
    }

    #[test]
    fn test_add_c_to_a_c_flag() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0xFF;
        cpu.registers.c = 0x01;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x81);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x00);
        assert_eq!(cpu.registers.get_flag(CpuFlags::C), true);
        assert_eq!(cpu.registers.get_flag(CpuFlags::H), true);
    }

    // ADD A, D
    #[test]
    fn test_add_d_to_a() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x10;
        cpu.registers.d = 0x20;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x82);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x30);
        assert_eq!(cpu.registers.f, 0x0);
    }

    // ADD A, E
    #[test]
    fn test_add_e_to_a() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x01;
        cpu.registers.e = 0x02;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x83);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x03);
        assert_eq!(cpu.registers.f, 0x0);
    }

    // ADD A, H
    #[test]
    fn test_add_h_to_a() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x05;
        cpu.registers.h = 0x05;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x84);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x0A);
        assert_eq!(cpu.registers.f, 0x0);
    }

    // ADD A, L
    #[test]
    fn test_add_l_to_a() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x02;
        cpu.registers.l = 0x03;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x85);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x05);
        assert_eq!(cpu.registers.f, 0x0);
    }

    // ADD A, (HL)
    #[test]
    fn test_add_hl_to_a() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x01;
        cpu.registers.set_hl(0x1000);
        cpu.bus.write_data(0x1000, 0x02);
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x86);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x03);
        assert_eq!(cpu.registers.f, 0x0);
    }

    // ADD A, A
    #[test]
    fn test_add_a_to_a() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x03;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x87);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x06);
        assert_eq!(cpu.registers.f, 0x0);
    }
    #[test]
    fn test_sub_no_carry() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x0A;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x97);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x00);
        assert_eq!(cpu.registers.f, 0xC0); // Z and N flags set, H and C cleared
    }

    #[test]
    fn test_sub_b_a() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0xF;
        cpu.registers.b = 0x5;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x90);
        cpu.step();
        assert_eq!(cpu.registers.a, 0xA);
        assert_eq!(cpu.registers.f, 0x40); // Z and N flags
    }
    #[test]
    fn test_sub_c() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x8;
        cpu.registers.c = 0x3;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x91);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x5);
        assert_eq!(cpu.registers.f & 0x40, 0x40); // N set
    }

    #[test]
    fn test_sub_d() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x7;
        cpu.registers.d = 0x2;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x92);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x5);
        assert_eq!(cpu.registers.f & 0x40, 0x40); // N set
    }

    #[test]
    fn test_sub_e() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0xA;
        cpu.registers.e = 0xA;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x93);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x00);
        assert_eq!(cpu.registers.f & 0xC0, 0xC0); // Z and N set
    }

    #[test]
    fn test_sub_h() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0xF;
        cpu.registers.h = 0x1;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x94);
        cpu.step();
        assert_eq!(cpu.registers.a, 0xE);
        assert_eq!(cpu.registers.f & 0x40, 0x40); // N set
    }

    #[test]
    fn test_sub_l() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x3;
        cpu.registers.l = 0x3;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x95);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x00);
        assert_eq!(cpu.registers.f & 0xC0, 0xC0); // Z and N set
    }

    #[test]
    fn test_sub_hl() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x5;
        cpu.registers.h = 0x10;
        cpu.registers.l = 0x00;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.get_hl(), 0x2); // memory at HL
        cpu.bus.write_data(cpu.registers.pc, 0x96);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x3);
        assert_eq!(cpu.registers.f & 0x40, 0x40); // N set
    }

    #[test]
    fn test_sub_a() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x7;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x97);
        cpu.step();
        assert_eq!(cpu.registers.a, 0x00);
        assert_eq!(cpu.registers.f & 0xC0, 0xC0); // Z and N set
    }
    #[test]
    fn test_sub_b_carry() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x05;
        cpu.registers.b = 0x0A; // A < B -> carry set
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x90); // SUB B
        cpu.step();
        assert_eq!(cpu.registers.a, 0xFB); // 5 - 10 = -5 = 0xFB
        assert_eq!(cpu.registers.f & 0x10, 0x10); // C flag set
        assert_eq!(cpu.registers.f & 0x40, 0x40); // N flag set
    }

    #[test]
    fn test_sub_b_half_carry() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x10;
        cpu.registers.b = 0x01; // borrow from bit 4
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x90); // SUB B
        cpu.step();
        assert_eq!(cpu.registers.a, 0x0F);
        assert_eq!(cpu.registers.f & 0x20, 0x20); // H flag set
        assert_eq!(cpu.registers.f & 0x40, 0x40); // N flag set
        assert_eq!(cpu.registers.f & 0x10, 0x00); // C flag not set
    }

    #[test]
    fn test_sub_a_no_carry_no_half() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x15;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.pc, 0x97); // SUB A
        cpu.step();
        assert_eq!(cpu.registers.a, 0x00);
        assert_eq!(cpu.registers.f & 0xC0, 0xC0); // Z and N set
        assert_eq!(cpu.registers.f & 0x30, 0x00); // H and C cleared
    }

    #[test]
    fn test_sub_hl_carry_half() {
        let mut cpu = Cpu::new();
        cpu.registers.a = 0x10;
        cpu.registers.h = 0x00;
        cpu.registers.l = 0x01;
        cpu.registers.f = 0x00;
        cpu.bus.write_data(cpu.registers.get_hl(), 0x11); // value > A
        cpu.bus.write_data(cpu.registers.pc, 0x96); // SUB (HL)
        cpu.step();
        assert_eq!(cpu.registers.a, 0xFF); // 0x10 - 0x11 = -1 = 0xFF
        assert_eq!(cpu.registers.f & 0x10, 0x10); // C set
        assert_eq!(cpu.registers.f & 0x20, 0x20); // H set
        assert_eq!(cpu.registers.f & 0x40, 0x40); // N set
        assert_eq!(cpu.registers.f & 0x80, 0x00); // Z cleared
    }
}
