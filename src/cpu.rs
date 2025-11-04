use crate::memorybus::MemoryBus;
use crate::register::Registers;
pub struct Cpu {
    registers: Registers,
    bus: MemoryBus,
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
        let cycles = self.execute(opcode);
        self.registers.increment_pc(cycles);
    }

    pub fn execute(&mut self, opcode: u8) -> u16 {
        match opcode {
            0x00 => {
                /*no operation :3*/
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
            0x02 => {
                /* LD (BC), A - Store the value of register A into the memory address in BC */
                self.bus
                    .write_data(self.registers.get_bc(), self.registers.a);
                2
            }
            _ => {
                panic!("unimplemented instruction");
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
}
