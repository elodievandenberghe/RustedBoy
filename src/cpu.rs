struct CPU {
    registers: Registers,
    pc: u16,
    sp: u16,
    bus: MemoryBus,
}

struct MemoryBus {
    memory: [u8; 0xFFFF],
}

impl MemoryBus {
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }
}

enum LoadByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,
}
enum LoadByteSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    D8,
    HLI,
}
enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
}

pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    f: FlagRegister,
}

impl Registers {
    fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }
}

struct FlagRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

const ZERO_FLAG_POS: u8 = 7;
const SUBTRACT_FLAG_POS: u8 = 6;
const HALF_CARRY_FLAG_POS: u8 = 5;
const CARRY_FLAG_POS: u8 = 4;

impl From<FlagRegister> for u8 {
    fn from(flag: FlagRegister) -> u8 {
        ((flag.zero as u8) << ZERO_FLAG_POS)
            | ((flag.subtract as u8) << SUBTRACT_FLAG_POS)
            | ((flag.half_carry as u8) << HALF_CARRY_FLAG_POS)
            | ((flag.carry as u8) << CARRY_FLAG_POS)
    }
}

impl From<u8> for FlagRegister {
    fn from(byte: u8) -> Self {
        FlagRegister {
            zero: (byte >> ZERO_FLAG_POS) & 1 != 0,
            subtract: (byte >> SUBTRACT_FLAG_POS) & 1 != 0,
            half_carry: (byte >> HALF_CARRY_FLAG_POS) & 1 != 0,
            carry: (byte >> CARRY_FLAG_POS) & 1 != 0,
        }
    }
}

enum Instruction {
    JP(JumpTest),
    ADD(ArithmeticTarget),
    LD(LoadType),
    PUSH(StackTarget),
    POP(StackTarget),
}

enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl CPU {
    fn step(&mut self) {
        let mut instruction_byte = self.bus.read_byte(self.pc);
        let prefixed = instruction_byte == 0xCB;

        if prefixed {
            instruction_byte = self.bus.read_byte(self.pc + 1);
        }

        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed)
        {
            self.execute(instruction)
        } else {
            panic!(
                "Unknown instruction found for: 0x{}{:02x}",
                if prefixed { "cb" } else { "" },
                instruction_byte
            );
        };

        self.pc = next_pc;
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::JP(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.f.zero,
                    JumpTest::Zero => self.registers.f.zero,
                    JumpTest::NotCarry => !self.registers.f.carry,
                    JumpTest::Carry => self.registers.f.carry,
                    JumpTest::Always => true,
                };
                self.jump(jump_condition)
            }
            Instruction::LD(load_type) => match load_type {
                LoadType::Byte(target, source) => {
                    let source_value = match source {
                        LoadByteSource::A => self.registers.a,
                        LoadByteSource::D8 => self.read_next_byte(),
                        LoadByteSource::HLI => self.bus.read_byte(self.registers.get_hl()),
                        _ => {
                            panic!("TODO: implement other sources")
                        }
                    };
                    match target {
                        LoadByteTarget::A => self.registers.a = source_value,
                        LoadByteTarget::HLI => {
                            self.bus.write_byte(self.registers.get_hl(), source_value)
                        }
                        _ => {
                            panic!("TODO: implement other targets")
                        }
                    };
                    match source {
                        LoadByteSource::D8 => self.pc.wrapping_add(2),
                        _ => self.pc.wrapping_add(1),
                    }
                }
                _ => {
                    panic!("TODO: implement other load types")
                }
            },
            Instruction::PUSH(target) => {
                let value = match target {
                    StackTarget::BC => self.registers.get_bc(),
                    _ => {
                        panic!("Support more targets")
                    }
                };
                self.push(value);
                self.pc.wrapping_add(1)
            }
            Instruction::POP(target) => {
                let result = self.pop();
                match target {
                    StackTarget::BC => self.registers.set_bc(result),
                    _ => {
                        panic!("TODO: support more targets")
                    }
                };
                self.pc.wrapping_add(1);
            }
            Instruction::ADD(target) => {
                match target {
                    ArithmeticTarget::C => {
                        let value = self.registers.c;
                        let new_value = self.add(value);
                        self.registers.a = new_value;
                        self.pc.wrapping_add(1)
                    }
                    _ => {
                        // TODO: implement other ADD targets
                        self.pc.wrapping_add(1)
                    }
                }
            }
        }
    }

    fn jump(&self, should_jump: bool) -> u16 {
        if should_jump {
            let lo = self.bus.read_byte(self.pc + 1) as u16;
            let hi = self.bus.read_byte(self.pc + 2) as u16;
            (hi << 8) | lo
        } else {
            self.pc.wrapping_add(3)
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let (result, did_overflow) = self.registers.a.overflowing_add(value);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;

        result
    }

    fn push(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, ((value & 0xFF00) >> 8) as u8);

        self.sp = self.sp.wrapping_sub(1);
        self.bus.writing_byte(self.sp, value(value & 0xFF) as u8);
    }
    fn pop(&mut self) -> u16 {
        let lsb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        let msb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        (msb << 8) | lsb
    }
}

impl Instruction {
    fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
        if prefixed {
            Self::from_byte_prefixed(byte)
        } else {
            Self::from_byte_not_prefixed(byte)
        }
    }

    fn from_byte_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            0x00 => {
                // Example: RLC B (Rotate Left Circular)
                // TODO: Add actual implementation when needed
                None
            }
            _ => None,
        }
    }

    fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            0xC3 => Some(Instruction::JP(JumpTest::Always)), // JP a16
            0xC2 => Some(Instruction::JP(JumpTest::NotZero)), // JP NZ,a16
            0xCA => Some(Instruction::JP(JumpTest::Zero)),   // JP Z,a16
            0xD2 => Some(Instruction::JP(JumpTest::NotCarry)), // JP NC,a16
            0xDA => Some(Instruction::JP(JumpTest::Carry)),  // JP C,a16
            0x81 => Some(Instruction::ADD(ArithmeticTarget::C)), // ADD A,C
            _ => None,
        }
    }
}
