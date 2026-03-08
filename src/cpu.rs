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

    fn alu_and(&mut self, value: u8) {
        self.registers.a &= value;
        self.registers.set_flag(CpuFlags::Z, self.registers.a == 0);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.set_flag(CpuFlags::H, true);
        self.registers.set_flag(CpuFlags::C, false);
    }

    fn alu_xor(&mut self, value: u8) {
        self.registers.a ^= value;
        self.registers.set_flag(CpuFlags::Z, self.registers.a == 0);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::C, false);
    }

    fn alu_or(&mut self, value: u8) {
        self.registers.a |= value;
        self.registers.set_flag(CpuFlags::Z, self.registers.a == 0);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::C, false);
    }

    fn alu_cp(&mut self, value: u8) {
        let r = self.registers.a.wrapping_sub(value);
        self.registers
            .set_flag(CpuFlags::C, self.registers.a < value);
        self.registers
            .set_flag(CpuFlags::H, (self.registers.a & 0xF) < (value & 0xF));
        self.registers.set_flag(CpuFlags::Z, r == 0);
        self.registers.set_flag(CpuFlags::N, true);
    }

    fn alu_add(&mut self, value: u8) {
        let a = self.registers.a;
        let r = a.wrapping_add(value);
        self.registers.set_flag(CpuFlags::Z, r == 0);
        self.registers
            .set_flag(CpuFlags::H, ((a & 0xF) + (value & 0xF)) > 0xF);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers
            .set_flag(CpuFlags::C, (a as u16) + (value as u16) > 0xFF);

        self.registers.a = r;
    }

    fn alu_adc(&mut self, value: u8) {
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
        let a = self.registers.a;
        let r = a.wrapping_sub(value);
        self.registers.set_flag(CpuFlags::Z, r == 0);
        self.registers
            .set_flag(CpuFlags::H, (a & 0xF) < (value & 0xF));
        self.registers.set_flag(CpuFlags::N, true);
        self.registers
            .set_flag(CpuFlags::C, (a as u16) < (value as u16 as u16));
        self.registers.a = r;
    }

    fn alu_sbc(&mut self, value: u8) {
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
        self.registers.set_flag(CpuFlags::Z, false);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.a = result;
    }

    fn alu_rrc(&mut self) {
        let result = self.registers.a.rotate_right(1);
        self.registers
            .set_flag(CpuFlags::C, self.registers.a & 0x1 == 0x1);
        self.registers.set_flag(CpuFlags::Z, false);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.a = result;
    }

    fn alu_rla(&mut self) {
        let old_a = self.registers.a;
        let carry = if self.registers.get_flag(CpuFlags::C) {
            1
        } else {
            0
        };
        let new_carry = (old_a & 0x80) != 0;
        let result = (old_a << 1) | carry;

        self.registers.a = result;
        self.registers.set_flag(CpuFlags::Z, false);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::C, new_carry);
    }

    fn alu_rra(&mut self) {
        let old_a = self.registers.a;
        let carry = if self.registers.get_flag(CpuFlags::C) {
            1
        } else {
            0
        };
        let new_carry = (old_a & 0x01) != 0;
        let result = carry | (old_a >> 1);

        self.registers.a = result;
        self.registers.set_flag(CpuFlags::Z, false);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::C, new_carry);
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
        self.registers.set_flag(CpuFlags::N, false);
        result
    }

    fn alu_daa(&mut self) {
        let old_value = self.registers.a;
        let mut adjustment = 0;
        let new_value;
        if self.registers.get_flag(CpuFlags::N) {
            if self.registers.get_flag(CpuFlags::H) {
                adjustment += 6;
            }
            if self.registers.get_flag(CpuFlags::C) {
                adjustment += 0x60;
            }
            new_value = old_value - adjustment;
        } else {
            if self.registers.get_flag(CpuFlags::H) || old_value & 0x0F > 0x09 {
                adjustment += 0x06;
            }
            if self.registers.get_flag(CpuFlags::C) || old_value > 0x99 {
                adjustment += 0x60;
            }
            new_value = old_value.wrapping_add(adjustment);
        }
        self.registers.a = new_value;
        self.registers.set_flag(CpuFlags::Z, new_value == 0);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(
            CpuFlags::C,
            self.registers.get_flag(CpuFlags::C) || adjustment >= 0x60,
        );
    }

    pub fn execute(&mut self, opcode: u8) -> u16 {
        match opcode {
            0x00 => {
                /* NOP */
                1
            }
            0x01 => {
                /* LD BC, d16 */
                let upper_byte = self.bus.read_data(self.registers.pc.wrapping_add(2));
                let lower_byte = self.bus.read_data(self.registers.pc.wrapping_add(1));
                self.registers
                    .set_bc(((upper_byte as u16) << 8) | (lower_byte as u16));
                3
            }
            0x02 => {
                /* LD (BC), A */
                self.bus
                    .write_data(self.registers.get_bc(), self.registers.a);
                1
            }
            0x03 => {
                /* INC BC */
                self.registers
                    .set_bc(self.registers.get_bc().wrapping_add(1));
                1
            }
            0x04 => {
                /* INC B */
                self.registers.b = self.registers.b.wrapping_add(1);
                1
            }
            0x05 => {
                /* DEC B */
                self.registers.b = self.registers.b.wrapping_sub(1);
                1
            }
            0x06 => {
                /* LD B, d8 */
                self.registers.b = self.bus.read_data(self.registers.pc.wrapping_add(1));
                2
            }
            0x07 => {
                /* RLCA */
                self.alu_rlc();
                1
            }
            0x08 => {
                /* LD (n16), SP */
                let addr = (self.bus.read_data(self.registers.pc.wrapping_add(2)) as u16) << 8
                    | (self.bus.read_data(self.registers.pc.wrapping_add(1)) as u16);
                self.bus.write_data(addr, self.registers.sp as u8);
                self.bus
                    .write_data(addr.wrapping_add(1), (self.registers.sp >> 8) as u8);
                3
            }
            0x09 => {
                /* ADD HL, BC */
                self.alu_add_16(self.registers.bc());
                1
            }
            0x0A => {
                /* LD A, (BC) */
                self.registers.a = self.bus.read_data(self.registers.bc());
                1
            }
            0x0B => {
                /* DEC BC */
                self.registers.set_bc(self.registers.bc().wrapping_sub(1));
                1
            }
            0x0C => {
                /* INC C */
                self.registers.c = self.increment_reg(self.registers.c);
                1
            }
            0x0D => {
                /* DEC C */
                self.registers.c = self.decrement_reg(self.registers.c);
                1
            }
            0x0E => {
                /* LD C, d8 */
                self.registers.c = self.bus.read_data(self.registers.pc.wrapping_add(1));
                2
            }
            0x0F => {
                /* RRCA */
                self.alu_rrc();
                1
            }
            0x11 => {
                //LD DE, d16
                let value = (self.bus.read_data(self.registers.pc + 2) as u16) << 8
                    | (self.bus.read_data(self.registers.pc + 1) as u16);
                self.registers.set_de(value);
                3
            }
            0x12 => {
                //LD (DE), A
                self.bus.write_data(self.registers.de(), self.registers.a);
                1
            }
            0x13 => {
                //INC DE
                self.registers.set_de(self.registers.de().wrapping_add(1));
                1
            }
            0x14 => {
                //INC D
                self.registers.d = self.increment_reg(self.registers.d);
                1
            }
            0x15 => {
                //DEC D
                self.registers.d = self.decrement_reg(self.registers.d);
                1
            }
            0x16 => {
                //LD D, d8
                self.registers.d = self.bus.read_data(self.registers.pc.wrapping_add(1));
                2
            }
            0x17 => {
                //RLA
                self.alu_rla();
                1
            }
            0x18 => {
                //JR s8
                let value_to_jump = self.bus.read_data(self.registers.pc.wrapping_add(1)) as i8;
                self.registers.pc =
                    (self.registers.pc.wrapping_add(2)).wrapping_add_signed(value_to_jump as i16);
                2
            }
            0x19 => {
                //ADD HL, DE
                self.alu_add_16(self.registers.get_de());
                1
            }
            0x1A => {
                self.registers.a = self.bus.read_data(self.registers.get_de());
                1
            }
            0x1B => {
                // DEC DE
                self.registers
                    .set_de(self.registers.get_de().wrapping_sub(1));
                1
            }
            0x1C => {
                // INC E
                self.increment_reg(self.registers.e);
                1
            }
            0x1D => {
                // INC E
                self.decrement_reg(self.registers.e);
                1
            }
            0x1E => {
                /* LD E, d8 */
                self.registers.e = self.bus.read_data(self.registers.pc.wrapping_add(1));
                2
            }
            0x1F => {
                //RRA
                self.alu_rra();
                1
            }
            0x20 => {
                //JR NZ, s8
                if !self.registers.get_flag(CpuFlags::Z) {
                    let value_to_jump = self.bus.read_data(self.registers.pc.wrapping_add(1)) as i8;
                    self.registers.pc = (self.registers.pc.wrapping_add(2))
                        .wrapping_add_signed(value_to_jump as i16);
                    0
                } else {
                    2
                }
            }
            0x21 => {
                let value = (self.bus.read_data(self.registers.pc + 2) as u16) << 8
                    | (self.bus.read_data(self.registers.pc + 1) as u16);
                self.registers.set_hl(value);
                3
            }
            0x22 => {
                //LD HL+ A
                self.bus
                    .write_data(self.registers.get_hl(), self.registers.a);
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_add(1));
                1
            }
            0x23 => {
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_add(1));
                1
            }
            0x24 => {
                // INC H
                self.registers.h = self.increment_reg(self.registers.h);
                1
            }
            0x25 => {
                // DEC H
                self.registers.h = self.decrement_reg(self.registers.h);
                1
            }
            0x26 => {
                // LD, d8
                self.registers.h = self.bus.read_data(self.registers.pc.wrapping_add(1));
                2
            }
            0x27 => {
                // DAA
                self.alu_daa();
                1
            }
            0x28 => {
                // jJR Z, s8
                if self.registers.get_flag(CpuFlags::Z) {
                    let value_to_jump = self.bus.read_data(self.registers.pc.wrapping_add(1)) as i8;
                    self.registers.pc = (self.registers.pc.wrapping_add(2))
                        .wrapping_add_signed(value_to_jump as i16);
                }
                2
            }
            0x29 => {
                // ADD HL, HL
                let hl = self.registers.get_hl();
                self.alu_add_16(hl);
                1
            }

            0x2A => {
                // LD A, (HL+)
                let addr = self.registers.get_hl();
                self.registers.a = self.bus.read_data(addr);
                self.registers.set_hl(addr.wrapping_add(1));
                1
            }

            0x2B => {
                // DEC HL
                let hl = self.registers.get_hl();
                self.registers.set_hl(hl.wrapping_sub(1));
                1
            }

            0x2C => {
                // INC L
                self.registers.l = self.increment_reg(self.registers.l);
                1
            }

            0x2D => {
                // DEC L
                self.registers.l = self.decrement_reg(self.registers.l);
                1
            }
            0x2E => {
                /* LD L, d8 */
                self.registers.l = self.bus.read_data(self.registers.pc.wrapping_add(1));
                2
            }
            0x2F => {
                self.registers.a = !self.registers.a;
                self.registers.set_flag(CpuFlags::N, true);
                self.registers.set_flag(CpuFlags::H, true);
                1
            }
            0x30 => {
                // JR NC, s8
                if !self.registers.get_flag(CpuFlags::C) {
                    let value_to_jump = self.bus.read_data(self.registers.pc.wrapping_add(1)) as i8;
                    self.registers.pc = (self.registers.pc.wrapping_add(2))
                        .wrapping_add_signed(value_to_jump as i16);
                    0
                } else {
                    2
                }
            }

            0x31 => {
                // LD SP, d16
                let upper = self.bus.read_data(self.registers.pc.wrapping_add(2));
                let lower = self.bus.read_data(self.registers.pc.wrapping_add(1));
                self.registers.sp = ((upper as u16) << 8) | lower as u16;
                3
            }

            0x32 => {
                // LD (HL-), A
                let addr = self.registers.get_hl();
                self.bus.write_data(addr, self.registers.a);
                self.registers.set_hl(addr.wrapping_sub(1));
                1
            }

            0x33 => {
                // INC SP
                self.registers.sp = self.registers.sp.wrapping_add(1);
                1
            }

            0x34 => {
                // INC (HL)
                let addr = self.registers.get_hl();
                let value = self.bus.read_data(addr);
                let result = self.increment_reg(value);
                self.bus.write_data(addr, result);
                1
            }

            0x35 => {
                // DEC (HL)
                let addr = self.registers.get_hl();
                let value = self.bus.read_data(addr);
                let result = self.decrement_reg(value);
                self.bus.write_data(addr, result);
                1
            }

            0x36 => {
                // LD (HL), d8
                let value = self.bus.read_data(self.registers.pc.wrapping_add(1));
                self.bus.write_data(self.registers.get_hl(), value);
                2
            }

            0x37 => {
                // SCF
                self.registers.set_flag(CpuFlags::C, true);
                self.registers.set_flag(CpuFlags::N, false);
                self.registers.set_flag(CpuFlags::H, false);
                1
            }

            0x38 => {
                // JR C, s8
                if self.registers.get_flag(CpuFlags::C) {
                    let offset = self.bus.read_data(self.registers.pc.wrapping_add(1)) as i8;
                    self.registers.pc =
                        (self.registers.pc.wrapping_add(2)).wrapping_add_signed(offset as i16);
                    return 0;
                }
                2
            }

            0x39 => {
                // ADD HL, SP
                self.alu_add_16(self.registers.sp);
                1
            }

            0x3A => {
                // LD A, (HL-)
                let addr = self.registers.get_hl();
                self.registers.a = self.bus.read_data(addr);
                self.registers.set_hl(addr.wrapping_sub(1));
                1
            }

            0x3B => {
                // DEC SP
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                1
            }

            0x3C => {
                // INC A
                self.registers.a = self.increment_reg(self.registers.a);
                1
            }

            0x3D => {
                // DEC A
                self.registers.a = self.decrement_reg(self.registers.a);
                1
            }
            0x3E => {
                /* LD A, d8 */
                self.registers.a = self.bus.read_data(self.registers.pc.wrapping_add(1));
                2
            }
            0x3F => {
                // CCF
                self.registers
                    .set_flag(CpuFlags::C, !self.registers.get_flag(CpuFlags::C));
                1
            }
            0x40 => {
                // LD B,B
                self.registers.b = self.registers.b;
                1
            }

            0x41 => {
                // LD B,C
                self.registers.b = self.registers.c;
                1
            }

            0x42 => {
                // LD B,D
                self.registers.b = self.registers.d;
                1
            }

            0x43 => {
                // LD B,E
                self.registers.b = self.registers.e;
                1
            }

            0x44 => {
                // LD B,H
                self.registers.b = self.registers.h;
                1
            }

            0x45 => {
                // LD B,L
                self.registers.b = self.registers.l;
                1
            }

            0x46 => {
                // LD B,(HL)
                let value = self.bus.read_data(self.registers.get_hl());
                self.registers.b = value;
                1
            }

            0x47 => {
                // LD B,A
                self.registers.b = self.registers.a;
                1
            }

            0x48 => {
                // LD C,B
                self.registers.c = self.registers.b;
                1
            }

            0x50 => {
                // LD D,B
                self.registers.d = self.registers.b;
                1
            }

            0x51 => {
                // LD D,C
                self.registers.d = self.registers.c;
                1
            }

            0x52 => {
                // LD D,D
                self.registers.d = self.registers.d;
                1
            }

            0x53 => {
                // LD D,E
                self.registers.d = self.registers.e;
                1
            }

            0x54 => {
                // LD D,H
                self.registers.d = self.registers.h;
                1
            }

            0x55 => {
                // LD D,L
                self.registers.d = self.registers.l;
                1
            }

            0x56 => {
                // LD D,(HL)
                let value = self.bus.read_data(self.registers.get_hl());
                self.registers.d = value;
                1
            }

            0x57 => {
                // LD D,A
                self.registers.d = self.registers.a;
                1
            }

            0x58 => {
                // LD E,B
                self.registers.e = self.registers.b;
                1
            }

            0x59 => {
                // LD E,C
                self.registers.e = self.registers.c;
                1
            }

            0x5A => {
                // LD E,D
                self.registers.e = self.registers.d;
                1
            }

            0x5B => {
                // LD E,E
                self.registers.e = self.registers.e;
                1
            }

            0x5C => {
                // LD E,H
                self.registers.e = self.registers.h;
                1
            }

            0x5D => {
                // LD E,L
                self.registers.e = self.registers.l;
                1
            }

            0x5E => {
                // LD E,(HL)
                let value = self.bus.read_data(self.registers.get_hl());
                self.registers.e = value;
                1
            }

            0x5F => {
                // LD E,A
                self.registers.e = self.registers.a;
                1
            }

            0x60 => {
                // LD H,B
                self.registers.h = self.registers.b;
                1
            }

            0x61 => {
                // LD H,C
                self.registers.h = self.registers.c;
                1
            }

            0x62 => {
                // LD H,D
                self.registers.h = self.registers.d;
                1
            }

            0x63 => {
                // LD H,E
                self.registers.h = self.registers.e;
                1
            }

            0x64 => {
                // LD H,H
                self.registers.h = self.registers.h;
                1
            }

            0x65 => {
                // LD H,L
                self.registers.h = self.registers.l;
                1
            }

            0x66 => {
                // LD H,(HL)
                let value = self.bus.read_data(self.registers.get_hl());
                self.registers.h = value;
                1
            }

            0x67 => {
                // LD H,A
                self.registers.h = self.registers.a;
                1
            }

            0x68 => {
                // LD L,B
                self.registers.l = self.registers.b;
                1
            }

            0x69 => {
                // LD L,C
                self.registers.l = self.registers.c;
                1
            }

            0x6A => {
                // LD L,D
                self.registers.l = self.registers.d;
                1
            }

            0x6B => {
                // LD L,E
                self.registers.l = self.registers.e;
                1
            }

            0x6C => {
                // LD L,H
                self.registers.l = self.registers.h;
                1
            }

            0x6D => {
                // LD L,L
                self.registers.l = self.registers.l;
                1
            }

            0x6E => {
                // LD L,(HL)
                let value = self.bus.read_data(self.registers.get_hl());
                self.registers.l = value;
                1
            }

            0x6F => {
                // LD L,A
                self.registers.l = self.registers.a;
                1
            }

            0x70 => {
                // LD (HL), B
                let addr = self.registers.get_hl();
                self.bus.write_data(addr, self.registers.b);
                1
            }

            0x71 => {
                // LD (HL), C
                let addr = self.registers.get_hl();
                self.bus.write_data(addr, self.registers.c);
                1
            }

            0x72 => {
                // LD (HL), D
                let addr = self.registers.get_hl();
                self.bus.write_data(addr, self.registers.d);
                1
            }

            0x73 => {
                // LD (HL), E
                let addr = self.registers.get_hl();
                self.bus.write_data(addr, self.registers.e);
                1
            }

            0x74 => {
                // LD (HL), H
                let addr = self.registers.get_hl();
                self.bus.write_data(addr, self.registers.h);
                1
            }

            0x75 => {
                // LD (HL), L
                let addr = self.registers.get_hl();
                self.bus.write_data(addr, self.registers.l);
                1
            }

            // 0x76 HALT skipped
            0x77 => {
                // LD (HL), A
                let addr = self.registers.get_hl();
                self.bus.write_data(addr, self.registers.a);
                1
            }

            0x78 => {
                // LD A,B
                self.registers.a = self.registers.b;
                1
            }

            0x79 => {
                // LD A,C
                self.registers.a = self.registers.c;
                1
            }

            0x7A => {
                // LD A,D
                self.registers.a = self.registers.d;
                1
            }

            0x7B => {
                // LD A,E
                self.registers.a = self.registers.e;
                1
            }

            0x7C => {
                // LD A,H
                self.registers.a = self.registers.h;
                1
            }

            0x7D => {
                // LD A,L
                self.registers.a = self.registers.l;
                1
            }

            0x7E => {
                // LD A,(HL)
                let addr = self.registers.get_hl();
                self.registers.a = self.bus.read_data(addr);
                1
            }

            0x7F => {
                // LD A,A
                self.registers.a = self.registers.a;
                1
            }

            0x49 => {
                // LD C,C
                self.registers.c = self.registers.c;
                1
            }

            0x4A => {
                // LD C,D
                self.registers.c = self.registers.d;
                1
            }

            0x4B => {
                // LD C,E
                self.registers.c = self.registers.e;
                1
            }

            0x4C => {
                // LD C,H
                self.registers.c = self.registers.h;
                1
            }

            0x4D => {
                // LD C,L
                self.registers.c = self.registers.l;
                1
            }

            0x4E => {
                // LD C,(HL)
                let value = self.bus.read_data(self.registers.get_hl());
                self.registers.c = value;
                1
            }

            0x4F => {
                // LD C,A
                self.registers.c = self.registers.a;
                1
            }
            0x78 => {
                /* LD A, B */
                self.registers.a = self.registers.b;
                1
            }
            0x80 => {
                /* ADD A, B */
                self.alu_add(self.registers.b);
                1
            }
            0x81 => {
                /* ADD A, C */
                self.alu_add(self.registers.c);
                1
            }
            0x82 => {
                /* ADD A, D */
                self.alu_add(self.registers.d);
                1
            }
            0x83 => {
                /* ADD A, E */
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
            0x88 => {
                // ADC A, B
                self.alu_adc(self.registers.b);
                1
            }
            0x89 => {
                // ADC A, C
                self.alu_adc(self.registers.c);
                1
            }
            0x8A => {
                // ADC A, D
                self.alu_adc(self.registers.d);
                1
            }
            0x8B => {
                // ADC A, E
                self.alu_adc(self.registers.e);
                1
            }
            0x8C => {
                // ADC A, H
                self.alu_adc(self.registers.h);
                1
            }
            0x8D => {
                // ADC A, L
                self.alu_adc(self.registers.l);
                1
            }
            0x8E => {
                // ADC A, (HL)
                let value = self.bus.read_data(self.registers.get_hl());
                self.alu_adc(value);
                1
            }
            0x8F => {
                // ADC A, A
                self.alu_adc(self.registers.a);
                1
            }
            0x90 => {
                /* SUB B */
                self.alu_sub(self.registers.b);
                1
            }
            0x91 => {
                /* SUB C */
                self.alu_sub(self.registers.c);
                1
            }
            0x92 => {
                /* SUB D */
                self.alu_sub(self.registers.d);
                1
            }
            0x93 => {
                /* SUB E */
                self.alu_sub(self.registers.e);
                1
            }
            0x94 => {
                /* SUB H */
                self.alu_sub(self.registers.h);
                1
            }
            0x95 => {
                /* SUB L */
                self.alu_sub(self.registers.l);
                1
            }
            0x96 => {
                /* SUB (HL) */
                let value = self.bus.read_data(self.registers.get_hl());
                self.alu_sub(value);
                1
            }
            0x97 => {
                /* SUB A */
                self.alu_sub(self.registers.a);
                1
            }
            0x98 => {
                // SBC A, B
                let value = self.registers.b;
                self.alu_sbc(value);
                1
            }
            0x99 => {
                // SBC A, C
                let value = self.registers.c;
                self.alu_sbc(value);
                1
            }
            0x9A => {
                // SBC A, D
                let value = self.registers.d;
                self.alu_sbc(value);
                1
            }
            0x9B => {
                // SBC A, E
                let value = self.registers.e;
                self.alu_sbc(value);
                1
            }
            0x9C => {
                // SBC A, H
                let value = self.registers.h;
                self.alu_sbc(value);
                1
            }
            0x9D => {
                // SBC A, L
                let value = self.registers.l;
                self.alu_sbc(value);
                1
            }
            0x9E => {
                // SBC A, (HL)
                let value = self.bus.read_data(self.registers.get_hl());
                self.alu_sbc(value);
                1
            }
            0x9F => {
                // SBC A, A
                let value = self.registers.a;
                self.alu_sbc(value);
                1
            }
            0xA0 => {
                // AND B
                self.alu_and(self.registers.b);
                1
            }
            0xA1 => {
                // AND C
                self.alu_and(self.registers.c);
                1
            }
            0xA2 => {
                // AND D
                self.alu_and(self.registers.d);
                1
            }
            0xA3 => {
                // AND E
                self.alu_and(self.registers.e);
                1
            }
            0xA4 => {
                // AND H
                self.alu_and(self.registers.h);
                1
            }
            0xA5 => {
                // AND L
                self.alu_and(self.registers.l);
                1
            }
            0xA6 => {
                // AND (HL)
                let value = self.bus.read_data(self.registers.get_hl());
                self.alu_and(value);
                1
            }
            0xA7 => {
                // AND A
                self.alu_and(self.registers.a);
                1
            }

            0xA8 => {
                // XOR B
                self.alu_xor(self.registers.b);
                1
            }
            0xA9 => {
                // XOR C
                self.alu_xor(self.registers.c);
                1
            }
            0xAA => {
                // XOR D
                self.alu_xor(self.registers.d);
                1
            }
            0xAB => {
                // XOR E
                self.alu_xor(self.registers.e);
                1
            }
            0xAC => {
                // XOR H
                self.alu_xor(self.registers.h);
                1
            }
            0xAD => {
                // XOR L
                self.alu_xor(self.registers.l);
                1
            }
            0xAE => {
                // XOR (HL)
                let value = self.bus.read_data(self.registers.get_hl());
                self.alu_xor(value);
                1
            }
            0xAF => {
                // XOR A
                self.alu_xor(self.registers.a);
                1
            }

            0xA0 => {
                // AND B
                self.alu_and(self.registers.b);
                1
            }
            0xB0 => {
                // OR B
                self.alu_or(self.registers.b);
                1
            }
            0xB1 => {
                // OR C
                self.alu_or(self.registers.c);
                1
            }
            0xB2 => {
                // OR D
                self.alu_or(self.registers.d);
                1
            }
            0xB3 => {
                // OR E
                self.alu_or(self.registers.e);
                1
            }
            0xB4 => {
                // OR H
                self.alu_or(self.registers.h);
                1
            }
            0xB5 => {
                // OR L
                self.alu_or(self.registers.l);
                1
            }
            0xB6 => {
                // OR (HL)
                let value = self.bus.read_data(self.registers.get_hl());
                self.alu_or(value);
                1
            }
            0xB7 => {
                // OR A
                self.alu_or(self.registers.a);
                1
            }
            0xB8 => {
                // CP B
                self.alu_cp(self.registers.b);
                1
            }
            0xB9 => {
                // CP C
                self.alu_cp(self.registers.c);
                1
            }
            0xBA => {
                // CP D
                self.alu_cp(self.registers.d);
                1
            }
            0xBB => {
                // CP E
                self.alu_cp(self.registers.e);
                1
            }
            0xBC => {
                // CP H
                self.alu_cp(self.registers.h);
                1
            }
            0xBD => {
                // CP L
                self.alu_cp(self.registers.l);
                1
            }
            0xBE => {
                // CP (HL)
                let value = self.bus.read_data(self.registers.get_hl());
                self.alu_cp(value);
                1
            }
            0xBF => {
                // CP A
                self.alu_cp(self.registers.a);
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
