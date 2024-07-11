use std::io;
use std::io::Read;
use arrseq_memory::dynamic_number;
use crate::operand;
use crate::operand::dynamic::Dynamic;
use crate::operand::register::Register;

pub mod dynamic;
pub mod register;

/// Named of the 2 supported operands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Name {
    /// Register only operands.
    Register,
    
    /// Dynamically addressed operand. This operand could potentially refer to one of many things.
    Dynamic
}

/// Metadata for the operand involving the size of the operands, addressing mode, and more.
///
/// Some fields are privately initiated to ensure the validity of the data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Meta {
    /// The size of the data being referenced by the operand(s).
    pub size: dynamic_number::Size,

    /// The name of the operand in which to store the result in.
    pub result: operand::Name,

    /// This data does not control the encoder and can be used to indicate any boolean based value.
    pub custom_data: bool,

    /// The encoded code of the dynamic operand.
    dynamic_code: u8,
}

impl Meta {
    /// # Result
    /// Instance of [Self] as long as the dynamic code is valid otherwise [Err(operand::dynamic::InvalidCodeError)] is
    /// returned.
    pub fn new(size: dynamic_number::Size, result: Name, custom_data: bool, dynamic_code: u8) -> Result<Self, dynamic::InvalidCodeError> {
        if !Dynamic::is_valid(dynamic_code) { return Err(dynamic::InvalidCodeError) }
        Ok(Self { size, result, custom_data, dynamic_code })
    }

    pub fn encode(self) -> u8 {
        let mut encoded = dynamic_number::Size::from(self.size).exponent_representation() << 6;
        encoded |= (matches!(self.result, Name::Dynamic) as u8) << 5;
        encoded |= self.dynamic_code << 1;
        encoded |= self.custom_data as u8;
        encoded
    }

    /// # Result
    /// This function has no error because the dynamic code is never invalid. Valid dynamic codes are 4 bits.
    pub fn decode(encoded: u8) -> Self {
        let size = dynamic_number::Size::from_exponent_representation(encoded >> 6).unwrap();
        let result = if (encoded >> 5) & 0b0000000_1 == 1 { Name::Dynamic } else { Name::Register };
        let dynamic_code = encoded >> 1 & 0b000_1111_0;
        let custom_data = encoded & 0b0000000_1 == 1;
        Self { size, result, dynamic_code, custom_data }
    }

    pub fn dynamic_code(self) -> u8 {
        self.dynamic_code
    }
}

/// The register and dynamic operand in one structure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegisterAndDynamic {
    /// The operand in which the result should be copied to.
    pub result: Name,
    pub register: Register,
    pub dynamic: Dynamic,
}

/// Enum containing the valid combinations of the operand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Combination {
    RegisterAndDynamic(RegisterAndDynamic),
    /// Exclusively the register operand.
    Register(Register),
    /// Exclusively the dynamic  operand.
    Dynamic(Dynamic)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Operands {
    /// The size of the data that the operands refer to.
    pub size: dynamic_number::Size,
    
    /// The operands in their valid combination.
    pub combination: Combination
}

#[derive(Debug)]
pub enum DecodeError {
    InvalidDynamicCode(dynamic::InvalidCodeError),
    Read(io::Error)
}

impl Operands {
    /// ```
    /// use arrseq_instruction::operand;
    /// use arrseq_instruction::operand::{Combination, Operands, RegisterAndDynamic};
    /// use arrseq_instruction::operand::dynamic::Dynamic;
    /// use arrseq_instruction::operand::register::Register;
    /// use arrseq_memory::dynamic_number;
    ///
    /// let operands = Operands {
    ///     size: dynamic_number::Size::Word,
    ///     combination: Combination::RegisterAndDynamic(RegisterAndDynamic {
    ///         result: operand::Name::Register,
    ///         register: Register::Accumulator,
    ///         dynamic: Dynamic::Constant(dynamic_number::Unsigned::Word(10))
    ///     })
    /// };
    /// ```
    pub fn decode(input: &mut impl Read) -> Result<Self, DecodeError> {
        let mut buffer = [0u8; 1];
        let meta = Meta::decode(input.read_exact(&mut buffer).map)
        todo!()
    }
}