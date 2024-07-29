#[cfg(test)]
mod test;

use std::io;
use std::io::Read;
use thiserror::Error;
use crate::instruction::operand::{AddressingMode, ComplexAddressing, ImmediateAddressing, Operand};
use crate::math::dynamic_number::{Signed, Size, Unsigned};

impl ComplexAddressing {
    fn requires_offset(code: u8) -> Option<bool> {
        Some(Self::MODES.get(code as usize)?.requires_offset)
    }

    fn requires_index_register(code: u8) -> Option<bool> {
        Some(Self::MODES.get(code as usize)?.requires_index_register)
    }
}

impl AddressingMode {
    fn requires_register(code: u8) -> Option<bool> {
        Some(Self::MODES.get(code as usize)?.requires_register)
    }

    fn requires_immediate(code: u8) -> Option<bool> {
        Some(Self::MODES.get(code as usize)?.requires_immediate)
    }
}

#[derive(Debug, Error)]
pub(crate) enum DecodeIoError {
    #[error("Could not retrieve addressing byte")]
    AddressingByte,
    #[error("Could not retrieve immediate value")]
    ImmediateValue,
    #[error("Could not retrieve immediate relative offset")]
    ImmediateOffset
}

#[derive(Debug, Error)]
pub(crate) enum DecodeError {
    #[error("Failed to read data for decoding")]
    Io { #[source] source: io::Error, error: DecodeIoError }
}

impl Operand {
    pub(crate) fn decode(input: &mut impl Read) -> Result<Self, DecodeError> {
        // Try to decode the addressing byte.
        let mut buffer = [0u8; 1];
        input
            .read_exact(&mut buffer)
            .map_err(|source| DecodeError::Io { source, error: DecodeIoError::AddressingByte })?;
        
        let addressing_mode = buffer[0] >> 6;
        let size = Size::from_power((buffer[0] & 0b00_11_0000) >> 4);
        // This segment should be used to either encode the offset immediate length or to encode a register.
        let end_segment = buffer[0] & 0b00_00_1111;
        
        // Decode the addressing mode.
        let mode = match addressing_mode {
            AddressingMode::REGISTER_CODE => AddressingMode::Register { register: end_segment },
            
            AddressingMode::IMMEDIATE_CODE
            | AddressingMode::RELATIVE_CODE => {
                let immediate_size = Size::from_power(end_segment >> 2);
                let immediate = Self::decode_immediate(input, immediate_size).map_err(|source| DecodeError::Io { source, error: DecodeIoError::ImmediateOffset })?;
                
                match addressing_mode {
                    AddressingMode::IMMEDIATE_CODE => AddressingMode::Immediate { mode: ImmediateAddressing::Immediate { immediate                       }},
                    AddressingMode::RELATIVE_CODE =>  AddressingMode::Immediate { mode: ImmediateAddressing::Relative  { offset: Signed::from(immediate) }},
                    // Addressing mode wasn't modified. The arms here are the same as the parent statement.
                    _ => unreachable!()
                }
            },
            
            AddressingMode::COMPLEX_CODE => { 
                let complex_mode = Self::decode_complex(input)?;
                AddressingMode::Complex { mode: complex_mode, base: end_segment } 
            },
            // There are 4 possible addressing modes in the first byte. This match covers all of them and the code is 2 
            // bits which guarantees this is unreachable.
            _ => unreachable!()
        };
        
        Ok(Self { size, mode })
    }

    fn decode_immediate(input: &mut impl Read, size: Size) -> io::Result<Unsigned> {
        todo!()
    }
    
    fn decode_complex(input: &mut impl Read) -> Result<ComplexAddressing, DecodeError> {
        todo!()
    }
}