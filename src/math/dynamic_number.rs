pub mod unprefixed;
pub mod arithmetic;
pub mod size;

use std::io;
use thiserror::Error;

/// # Power
/// The power is a representation of this primitive data type which when set to the power of 2 gives the size in bytes.
/// The power only has its 2 least significant bits used and the rest are discarded.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    U8,
    U16,
    U32,
    U64
}

impl Size {
    /// Construct this enum from an exponent of the power of 2.
    pub fn from_power(size: u8) -> Self {
        match size & 0b000000_11 {
            0 => Self::U8,
            1 => Self::U16,
            2 => Self::U32,
            3 => Self::U64,
            _ => unreachable!()
        }
    }

    /// Convert this enum representation to a power of 2.
    pub fn to_power(self) -> u8 {
        match self {
            Self::U8 => 0,
            Self::U16 => 1,
            Self::U32 => 2,
            Self::U64 => 3
        }
    }
}

impl From<Unsigned> for Size {
    fn from(value: Unsigned) -> Self {
        match value {
            Unsigned::U8(_) => Self::U8,
            Unsigned::U16(_) => Self::U16,
            Unsigned::U32(_) => Self::U32,
            Unsigned::U64(_) => Self::U64
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unsigned {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64)
}

impl From<Unsigned> for u8 {
    fn from(value: Unsigned) -> Self {
        match value {
            Unsigned::U8(value) => value,
            Unsigned::U16(value) => value as u8,
            Unsigned::U32(value) => value as u8,
            Unsigned::U64(value) => value as u8,
        }
    }
}

impl From<Unsigned> for u16 {
    fn from(value: Unsigned) -> Self {
        match value {
            Unsigned::U8(value) => value as u16,
            Unsigned::U16(value) => value,
            Unsigned::U32(value) => value as u16,
            Unsigned::U64(value) => value as u16,
        }
    }
}

impl From<Unsigned> for u32 {
    fn from(value: Unsigned) -> Self {
        match value {
            Unsigned::U8(value) => value as u32,
            Unsigned::U16(value) => value as u32,
            Unsigned::U32(value) => value,
            Unsigned::U64(value) => value as u32,
        }
    }
}

impl From<Unsigned> for u64 {
    fn from(value: Unsigned) -> Self {
        match value {
            Unsigned::U8(value) => value as u64,
            Unsigned::U16(value) => value as u64,
            Unsigned::U32(value) => value as u64,
            Unsigned::U64(value) => value
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signed {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64)
}

impl From<Signed> for i8 {
    fn from(value: Signed) -> Self {
        match value {
            Signed::I8(value) => value,
            Signed::I16(value) => value as i8,
            Signed::I32(value) => value as i8,
            Signed::I64(value) => value as i8,
        }
    }
}

impl From<Signed> for i16 {
    fn from(value: Signed) -> Self {
        match value {
            Signed::I8(value) => value as i16,
            Signed::I16(value) => value,
            Signed::I32(value) => value as i16,
            Signed::I64(value) => value as i16,
        }
    }
}

impl From<Signed> for i32 {
    fn from(value: Signed) -> Self {
        match value {
            Signed::I8(value) => value as i32,
            Signed::I16(value) => value as i32,
            Signed::I32(value) => value,
            Signed::I64(value) => value as i32,
        }
    }
}

impl From<Signed> for i64 {
    fn from(value: Signed) -> Self {
        match value {
            Signed::I8(value) => value as i64,
            Signed::I16(value) => value as i64,
            Signed::I32(value) => value as i64,
            Signed::I64(value) => value
        }
    }
}