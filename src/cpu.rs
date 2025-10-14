#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register16 {
    BC,
    DE,
    HL,
    SP,
    AF,
}


pub struct CPU {
    pub a: Byte,
    pub b: Byte,
    pub c: Byte,
    pub d: Byte,
    pub e: Byte,
    pub h: Byte,
    pub l: Byte,
    pub f: Byte,
    pub s: Byte,
    pub sp: Word,
    pub pc: Word,
}
