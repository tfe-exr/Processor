//! Unsized absolute number. 
//! While Rust has u8, u16... for absolute values, it does not have a simple enum for variable length
//! absolute integers. 

// TODO: Use TryFrom<u8..u16..u32..u64> on Data

// Constants

pub const BYTE_SIZE: u8 = 1;
pub const WORD_SIZE: u8 = 2;
pub const DUAL_SIZE: u8 = 4;
pub const QUAD_SIZE: u8 = 8;

// Implementations

/// Absolute modes.
/// Base type variants for representing an absolute value.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum Type {
	#[default]
	Byte,
	Word,
	Dual,
	Quad
}

impl From<Data> for Type {
	fn from(value: Data) -> Self {
		match value {
			Data::Byte(_) => Self::Byte,
			Data::Word(_) => Self::Word,
			Data::Dual(_) => Self::Dual,
			Data::Quad(_) => Self::Quad
		}
	}
}

impl Type {
	/// Create from the number of bytes.
	pub fn from_bytes(bytes: u8) -> Option<Self> {
		Some(match bytes {
			BYTE_SIZE => Type::Byte,
			WORD_SIZE => Type::Word,
			DUAL_SIZE => Type::Dual,
			QUAD_SIZE => Type::Quad,
			_ => return None
		})
	}
	
	/// Create from an exponent of 2. The maximum supported exponent is 3.
	pub fn from_exponent(exponent: u8) -> Option<Self> {
		Self::from_bytes(2 ^ exponent)
	}
}

/// Variable absolute data type.
/// Complete variants that annotate numbers with their type in the same enum allowing for the data type to be changed
/// during runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data {
	Byte(u8),
	Word(u16),
	Dual(u32),
	Quad(u64)
}

impl Data {
	pub fn to_le_bytes(&self) -> Vec<u8> {
		let mut bytes = Vec::new();
		
		match self {
			Self::Byte(number) => bytes.extend(number.to_le_bytes()),
			Self::Word(number) => bytes.extend(number.to_le_bytes()),
			Self::Dual(number) => bytes.extend(number.to_le_bytes()),
			Self::Quad(number) => bytes.extend(number.to_le_bytes()),
		}
		
		bytes
	}
}

impl From<Type> for Data {
	fn from(value: Type) -> Self {
		match value {
			Type::Byte => Self::Byte(0),
			Type::Word => Self::Word(0),
			Type::Dual => Self::Dual(0),
			Type::Quad => Self::Quad(0)
		}
	}
}

// TODO: Implement unit tests