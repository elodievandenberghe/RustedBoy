use crate::memorybus::MemoryBus;
use crate::register::Registers;
pub struct Cpu {
    registers: Registers,
    memorybus: MemoryBus,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            memorybus: MemoryBus::new(),
        }
    }
    pub fn execute(opcode: u8) {
        match opcode {
            0x00 => { /* */ }
            _ => {}
        }
    }
}
