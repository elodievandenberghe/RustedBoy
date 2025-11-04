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
                self.registers.a = self.registers.b;
                1
            }
            0x06 => {
                self.registers.b = self.bus.read_data(self.registers.pc.wrapping_add(1));
                2
            }
            0x02 => {
                self.bus
                    .write_data(self.registers.get_bc(), self.registers.a);
                2
            }
            _ => {
                panic!("unimplemented instruction")
            }
        }
    }
}
