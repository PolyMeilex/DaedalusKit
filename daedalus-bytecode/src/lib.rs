use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive as _;
use std::io::{self, Cursor, Read, Write};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Bytecode {
    bytecode: Vec<u8>,
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytecode
    }

    pub fn next_available_address(&self) -> u32 {
        self.bytecode.len() as u32
    }

    pub fn block<'a>(&mut self, i: impl IntoIterator<Item = &'a Instruction>) -> u32 {
        let mut block = self.block_builder();
        block.extend(i);
        block.addr()
    }

    pub fn block_builder(&mut self) -> BytecodeBlockBuilder<'_> {
        BytecodeBlockBuilder {
            addr: self.next_available_address(),
            bytecode: &mut self.bytecode,
        }
    }

    pub fn decode(mut r: impl Read) -> io::Result<Self> {
        let len = r.read_u32::<LittleEndian>()? as usize;
        let mut bytecode = vec![0; len];
        r.read_exact(&mut bytecode)?;

        Ok(Self { bytecode })
    }

    pub fn encode(&self, mut w: impl Write) -> io::Result<usize> {
        w.write_u32::<LittleEndian>(self.bytecode.len() as u32)
            .unwrap();
        w.write_all(&self.bytecode).unwrap();

        Ok(std::mem::size_of::<u32>() + std::mem::size_of::<u8>() * self.bytecode.len())
    }

    pub fn instructions(&self) -> impl Iterator<Item = Instruction> + '_ {
        let mut r = Cursor::new(&self.bytecode);
        std::iter::from_fn(move || {
            if r.position() as usize >= self.bytecode.len() {
                None
            } else {
                Some(Instruction::decode(&mut r).unwrap())
            }
        })
    }
}

#[derive(Debug)]
pub struct BytecodeBlockBuilder<'a> {
    addr: u32,
    bytecode: &'a mut Vec<u8>,
}

impl<'a> BytecodeBlockBuilder<'a> {
    fn encode(&mut self, i: &Instruction) {
        i.encode(&mut self.bytecode).unwrap();
    }

    pub fn push_instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.encode(&instruction);
        self
    }

    pub fn extend<'b>(&mut self, i: impl IntoIterator<Item = &'b Instruction>) -> &mut Self {
        for i in i {
            self.encode(i);
        }
        self
    }

    pub fn ret(&mut self) -> &mut Self {
        self.encode(&Instruction::ret());
        self
    }

    pub fn var_assign_int(&mut self, (array_symbol, id): (u32, u8), value: i32) -> &mut Self {
        self.encode(&Instruction::push_int(value));
        if id == 0 {
            self.encode(&Instruction::push_var(array_symbol));
        } else {
            self.encode(&Instruction::push_var_array(array_symbol, id));
        }
        self.encode(&Instruction::mov_int());
        self
    }

    pub fn instance_assign_int(&mut self, (array_symbol, id): (u32, u8), value: i32) -> &mut Self {
        self.encode(&Instruction::push_int(value));
        if id == 0 {
            self.encode(&Instruction::push_var(array_symbol));
        } else {
            self.encode(&Instruction::push_var_array(array_symbol, id));
        }
        self.encode(&Instruction::mov_int());
        self
    }

    pub fn addr(&self) -> u32 {
        self.addr
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub data: InstructionData,
}

impl Instruction {
    pub fn decode(mut r: impl Read) -> std::io::Result<Self> {
        let opcode = Opcode::from_u8(r.read_u8()?).unwrap();
        let data = match opcode {
            Opcode::Call | Opcode::Bz | Opcode::B => {
                let a = r.read_u32::<LittleEndian>()?;
                InstructionData::Address(a)
            }
            Opcode::PushInt => {
                let i = r.read_i32::<LittleEndian>()?;
                InstructionData::Immediate(i)
            }
            Opcode::CallExtern | Opcode::PushVar | Opcode::PushVarInstance | Opcode::GMovI => {
                let s = r.read_u32::<LittleEndian>()?;
                InstructionData::Symbol(s)
            }
            Opcode::PushVV => {
                let symbol = r.read_u32::<LittleEndian>()?;
                let index = r.read_u8()?;
                InstructionData::SymbolIndex { symbol, index }
            }
            _ => InstructionData::None,
        };

        Ok(Self { opcode, data })
    }

    pub fn encode(&self, mut w: impl Write) -> std::io::Result<usize> {
        w.write_u8(self.opcode as u8)?;
        match self.data {
            InstructionData::Address(i) | InstructionData::Symbol(i) => {
                w.write_u32::<LittleEndian>(i)?;
            }
            InstructionData::Immediate(i) => {
                w.write_i32::<LittleEndian>(i)?;
            }
            InstructionData::SymbolIndex { symbol, index } => {
                w.write_u32::<LittleEndian>(symbol)?;
                w.write_u8(index)?;
            }
            InstructionData::None => {}
        }
        Ok(self.size())
    }

    pub fn size(&self) -> usize {
        let data_size = match self.data {
            InstructionData::Address(_) => std::mem::size_of::<u32>(),
            InstructionData::Immediate(_) => std::mem::size_of::<u32>(),
            InstructionData::Symbol(_) => std::mem::size_of::<u32>(),
            InstructionData::SymbolIndex { .. } => {
                std::mem::size_of::<u32>() + std::mem::size_of::<u8>()
            }
            InstructionData::None => 0,
        };

        std::mem::size_of::<u8>() + data_size
    }

    pub fn push_int(immediate: i32) -> Self {
        Self {
            opcode: Opcode::PushInt,
            data: InstructionData::Immediate(immediate),
        }
    }

    pub fn push_var(symbol: u32) -> Self {
        Self {
            opcode: Opcode::PushVar,
            data: InstructionData::Immediate(symbol as i32),
        }
    }

    pub fn mov_int() -> Self {
        Self {
            opcode: Opcode::MovInt,
            data: InstructionData::None,
        }
    }

    pub fn push_var_array(symbol: u32, index: u8) -> Self {
        Self {
            opcode: Opcode::PushVV,
            data: InstructionData::SymbolIndex { symbol, index },
        }
    }

    pub fn push_var_instance(symbol: u32) -> Self {
        Self {
            opcode: Opcode::PushVarInstance,
            data: InstructionData::Symbol(symbol),
        }
    }

    pub fn call_extern(symbol: u32) -> Self {
        Self {
            opcode: Opcode::CallExtern,
            data: InstructionData::Symbol(symbol),
        }
    }

    pub fn call(symbol: u32) -> Self {
        Self {
            opcode: Opcode::Call,
            data: InstructionData::Symbol(symbol),
        }
    }

    pub fn negate() -> Self {
        Self {
            opcode: Opcode::Negate,
            data: InstructionData::None,
        }
    }

    pub fn ret() -> Self {
        Self {
            opcode: Opcode::Return,
            data: InstructionData::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstructionData {
    Address(u32),
    Immediate(i32),
    Symbol(u32),
    SymbolIndex { symbol: u32, index: u8 },
    None,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum Opcode {
    /// Add `a` and `b` and put the result back onto the stack.
    Add = 0,

    /// Subtract `b` from `a` and put the result back onto the stack.
    Sub = 1,

    /// Multiply `a` and `b` and put the result back onto the stack.
    Mul = 2,

    /// Divide `a` by `b` and put the result back onto the stack.
    Div = 3,

    /// Divide `a` by `b` and put the remainder back onto the stack.
    Mod = 4,

    /// Compute the bitwise or of `a` and `b` and put the result back onto the stack.
    Or = 5,

    /// Compute the bitwise and of `a` and `b` and put the result back onto the stack.
    /// a & b
    AndB = 6,

    /// Test if `a` is less than `b` and put `1` or `0` onto the stack if
    /// the test is true or false respectively.
    Lt = 7,

    /// Test if `a` is greater than `b` and put `1` or `0` onto the stack
    /// if the test is true or false respectively.
    Gt = 8,

    /// Write `b` to `x` as an integer.
    MovInt = 9,

    /// Test if `a == 1` or `b == 1` and put `1` or `0` onto the stack if
    /// the test is true or false respectively.
    Orr = 11,

    /// Test if `a == 1` and `b == 1` and put `1` or `0` onto the stack if
    /// the test is true or false respectively.
    And = 12,

    /// Left shift  `a` by `b` bits and put the result back onto the stack.
    Lsl = 13,

    /// Right shift  `a` by `b` bits and put the result back onto the stack.
    Lsr = 14,

    /// Test if `a` is less than or equal to `b` and put `1` or `0` onto the
    /// stack if the test is true or false respectively.
    Lte = 15,

    /// Test if `a` is equal to `b` and put `1` or `0` onto the
    /// stack if the test is true or false respectively.
    Eq = 16,

    /// Test if `a` is not equal to `b` and put `1` or `0` onto the
    /// stack if the test is true or false respectively.
    Neq = 17,

    /// Test if `a` is greater than or equal to `b` and put `1` or `0` onto the
    /// stack if the test is true or false respectively.
    Gte = 18,

    /// Add `x` and `b` and assign the result back to `x`.
    /// `x` must be a reference to an integer.
    AddMovI = 19,

    /// Subtract `b` from `x` and assign the result back to `x`.
    /// `x` must be a reference to an integer.
    SubMovI = 20,

    /// Multiply `x` from `b` and assign the result back to `x`.
    /// `x` must be a reference to an integer.
    MulMovI = 21,

    /// Divide `x` by `b` and assign the result back to `x`.
    /// `x` must be a reference to an integer.
    DivMovI = 22,

    /// Compute `+a` and put the result back onto the stack.
    Plus = 30,

    /// Compute `-a` and put the result back onto the stack.
    Negate = 31,

    /// Compute `!a` and put the result back onto the stack.
    Not = 32,

    /// Compute the bitwise complement `a` and put the result back onto the stack.
    Cmpl = 33,

    /// Do nothing.
    Nop = 45,

    /// Return from the currently running function
    Return = 60,

    /// Call the function at the address provided in the instruction.
    Call = 61,

    /// Call the external function at the symbol index provided in the instruction.
    CallExtern = 62,

    /// Push the immediate value provided in the instruction onto the stack as an integer.
    PushInt = 64,

    /// Push the symbol with the index provided in the instruction onto the stack as a reference.
    PushVar = 65,

    /// Push the instance with the symbol index provided in the instruction onto the stack as a reference.
    PushVarInstance = 67,

    /// Write `m` to `x` as a string.
    MovS = 70,

    /// Write `m` to `x` as a string reference; not implemented.
    MovSs = 71,

    /// Write `b` to `x` as a function reference.
    MovVF = 72,

    /// Write `b` to `x` as a floating point number.
    MovF = 73,

    /// Write `y` to `x` as an instance reference.
    MovVI = 74,

    /// Immediately jump to the instruction at the address provided in the instruction.
    B = 75,

    /// Jump to the instruction at the address provided in the instruction if `a == 0`.
    Bz = 76,

    /// Set the global instance reference to the instance with the symbol index provided in the instruction.
    GMovI = 80,

    /// Push the element at the given index of the symbol with the index provided in the
    /// instruction onto the stack as a reference.
    PushVV = 245,
}
