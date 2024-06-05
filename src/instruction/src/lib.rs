//! Binary instruction format is as follows.
//!
//! Driver 0:
//! - Extension: Operation's extension.
//! - Synchronise: Ensure execution is synchronous in respect to other processors.
//! - Destination Dynamic: Base the result location off the dynamic operand.
//!
//! Driver 1
//! - Operation: Operation to execute.
//! - Addressing: Dynamic operand's addressing method.
//! - immediate exponent: Addressing method's control parameter.
//!
//! Data:
//! - Width: Operating data size.
//! - Static Operand: Static register operand.
//! - Dynamic Operand: Dynamically addressable operand.
//!
//! Immediate 0..8 quantized to 0, 2, 4 and 8.

#![allow(clippy::unusual_byte_groupings)]

pub mod absolute;
pub mod operand;
pub mod operation;

use std::io;
use std::io::Read;
use crate::operand::{AllPresent, Dynamic, FromCodesError, Operand, Operands};
use crate::operation::{Extension, ExtensionFromCodeInvalid, Operation};

// region: Binary instruction bit masks
pub const DRIVER0_EXTENSION_MASK           : u8 = 0b111111_0_0;
pub const DRIVER0_SYNCHRONISE_MASK         : u8 = 0b000000_1_0;
pub const DRIVER0_DYNAMIC_DESTINATION      : u8 = 0b000000_0_1;
pub const DRIVER1_OPERATION_MASK           : u8 = 0b1111_00_00;
pub const DRIVER1_ADDRESSING_MASK          : u8 = 0b0000_11_00;
pub const DRIVER1_ADDRESSING_PARAMETER_MASK: u8 = 0b0000_00_11;
pub const DATA_WIDTH_MASK                  : u8 = 0b11_000_000;
pub const DATA_STATIC_OPERAND_MASK         : u8 = 0b00_111_000;
pub const DATA_DYNAMIC_OPERAND_MASK        : u8 = 0b00_000_111;
// endregion

/// Structured data from the driver bytes. All data generated by inherent functions are unchecked. Contains utility
/// functions for coding driver bytes.
pub struct Driver {
	/// Operation extension
	pub extension: u8,
	pub operation: u8,
	pub synchronise: bool,
	/// Whether to store the data where the dynamic operand points if its addressing mode supports it.
	pub dynamic_destination: bool,
	/// Addressing mode of the dynamic operand
	pub addressing: u8,
	/// To determine how many bytes the immediate is.
	pub immediate_exponent: u8
}

impl Driver {
	pub fn extract_extension(driver0: u8) -> u8 {
		(DRIVER0_EXTENSION_MASK & driver0) >> 2
	}

	/// Only the first 6 bits of the extension is used.
	pub fn set_extension(driver0: u8, extension: u8) -> u8 {
		let layer = (0b00_111111 & extension) << 2;
		(!DRIVER0_EXTENSION_MASK & driver0) | layer
	}

	pub fn extract_synchronise(driver0: u8) -> bool {
		// Value will always be 1 bit.
		let bit = (DRIVER0_SYNCHRONISE_MASK & driver0) >> 1;
		bit == 1
	}

	pub fn set_synchronise(driver0: u8, lock: bool) -> u8 {
		let layer = (lock as u8) << 1;
		(!DRIVER0_SYNCHRONISE_MASK & driver0) | layer
	}

	pub fn extract_dynamic_destination(driver0: u8) -> bool {
		// Value will always be 1 bit.
		(DRIVER0_DYNAMIC_DESTINATION & driver0) == 1
	}

	pub fn set_dynamic_destination(driver0: u8, dynamic_destination: bool) -> u8 {
		(!DRIVER0_DYNAMIC_DESTINATION & driver0) | dynamic_destination as u8
	}

	pub fn extract_operation(driver1: u8) -> u8 {
		(DRIVER1_OPERATION_MASK & driver1) >> 4
	}

	/// Only the first 4 bits of the operation is used.
	pub fn set_operation(driver1: u8, operation: u8) -> u8 {
		let layer = (0b0000_1111 & operation) << 4;
		(!DRIVER1_OPERATION_MASK & driver1) | layer
	}

	pub fn extract_addressing(driver1: u8) -> u8 {
		(DRIVER1_ADDRESSING_MASK & driver1) >> 2
	}

	/// Only the first 2 bits of the addressing is used.
	pub fn set_addressing(driver1: u8, addressing: u8) -> u8 {
		let layer = (0b_000000_11 & addressing) << 2;
		(!DRIVER1_ADDRESSING_MASK & driver1) | layer
	}

	pub fn extract_immediate_exponent(driver1: u8) -> u8 {
		DRIVER1_ADDRESSING_PARAMETER_MASK & driver1
	}

	/// Only the first 2 bits of the addressing is used.
	pub fn set_immediate_exponent(driver1: u8, immediate_exponent: u8) -> u8 {
		let layer = 0b000000_11 & immediate_exponent;
		(!DRIVER1_ADDRESSING_PARAMETER_MASK & driver1) | layer
	}

	pub fn from_encoded(bytes: [u8; 2]) -> Self {
		let driver0 = bytes[0];
		let driver1 = bytes[1];

		Driver {
			extension: Driver::extract_extension(driver0),
			operation: Driver::extract_operation(driver1),
			synchronise: Driver::extract_synchronise(driver0),
			dynamic_destination: Driver::extract_dynamic_destination(driver0),
			addressing: Driver::extract_addressing(driver1),
			immediate_exponent: Driver::extract_immediate_exponent(driver1),
		}
	}

	pub fn encode(&self) -> [u8; 2] {
		let mut driver0 = Driver::set_extension(0, self.extension);
		driver0 = Driver::set_synchronise(driver0, self.synchronise);
		driver0 = Driver::set_dynamic_destination(driver0, self.dynamic_destination);

		let mut driver1 = Driver::set_operation(0, self.operation);
		driver1 = Driver::set_addressing(driver1, self.addressing);
		driver1 = Driver::set_immediate_exponent(driver1, self.immediate_exponent);

		[driver0, driver1]
	}
}

/// The operand to dereference store the operation result in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
	Static,
	Dynamic
}

/// Data byte structure.
pub struct RawData {
	pub width: u8,
	pub x_static: u8,
	pub x_dynamic: u8
}

impl RawData {
	pub fn extract_width(data: u8) -> u8 {
		(DATA_WIDTH_MASK & data) >> 6
	}

	/// Only first 2 bits are used.
	pub fn set_width(data: u8, width: u8) -> u8 {
		let layer = (0b000000_11 & width) << 6;
		(!DATA_WIDTH_MASK & data) | layer
	}

	pub fn extract_static(data: u8) -> u8 {
		(DATA_STATIC_OPERAND_MASK & data) >> 3
	}

	/// Only first 3 bits are used.
	pub fn set_static(data: u8, x_static: u8) -> u8 {
		let layer = (0b00000_111 & x_static) << 3;
		(!DATA_STATIC_OPERAND_MASK & data) | layer
	}

	pub fn extract_dynamic(data: u8) -> u8 {
		DATA_DYNAMIC_OPERAND_MASK & data
	}

	/// Only first 3 bits are used.
	pub fn set_dynamic(data: u8, dynamic: u8) -> u8 {
		let layer = 0b00000_111 & dynamic;
		(!DATA_DYNAMIC_OPERAND_MASK & data) | layer
	}

	pub fn from_encoded(encoded: u8) -> Self {
		Self {
			width: RawData::extract_width(encoded),
			x_static: RawData::extract_static(encoded),
			x_dynamic: RawData::extract_static(encoded)
		}
	}

	pub fn encode(&self) -> u8 {
		let mut encoded = RawData::set_width(0, self.width);
		encoded = RawData::set_static(encoded, self.x_static);
		RawData::set_dynamic(encoded, self.x_dynamic)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data {
	pub width: absolute::Type,
	pub destination: Destination,
	pub operands: Operands
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
	pub operation: Extension,
	/// Width of operands when dereferenced and for storing result.
	pub width: absolute::Type,
	pub synchronise: bool,
	pub data: Option<Data>
}

#[derive(Debug)]
pub enum DecodeError {
	/// Stream failed to read.
	StreamRead(io::Error),
	/// Not enough bytes.
	Length,
	/// The extension and or operation are invalid.
	InvalidCode(ExtensionFromCodeInvalid),
	/// Error caused from interpreting the dynamic operand
	Dynamic(FromCodesError)
}

/// Caused by using a destination which corresponds to an operand that is not provided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationError {
	/// No data included.
	Data,
	/// The static operand wasn't present.
	Static,
	/// The dynamic operand wasn't present.
	Dynamic
}

impl Instruction {
	// Decode an encoded binary stream into an instruction.
	pub fn from_encoded(stream: &mut impl Read) -> Result<Self, DecodeError> {
		// Decode driver bytes.
		let mut encoded_driver = [0u8; 2];

		match stream.read(&mut encoded_driver) {
			Ok(length) => if length != encoded_driver.len() { return Err(DecodeError::Length) },
			Err(error) => return Err(DecodeError::StreamRead(error))
		};

		let driver = Driver::from_encoded(encoded_driver);

		let mut extension =  match Extension::from_codes(driver.extension, driver.operation) {
			Ok(operation) => operation,
			Err(error) => return Err(DecodeError::InvalidCode(error))
		};

		// Decode data bytes.
		let mut data: Option<Data> = None;
		let operation = extension.operation();

		if operation.expects_operand() {
			// Decode data byte.
			let mut data_encoded = [0u8; 1];
			match stream.read(&mut data_encoded) {
				Ok(length) => if length != data_encoded.len() { return Err(DecodeError::Length); },
				Err(error) => return Err(DecodeError::StreamRead(error))
			};

			let data_raw = RawData::from_encoded(data_encoded[0]);

			// Construct operand field.
			let operands = if operation.expects_all() {
				let x_dynamic = match Dynamic::from_codes(data_raw.x_dynamic, driver.addressing, driver
					.immediate_exponent, stream) {
					Ok(operand) => operand,
					Err(error) => return Err(DecodeError::Dynamic(error))
				};

				Operands::AllPresent(AllPresent {
					x_static: data_raw.x_static,
					x_dynamic
				})
			} else if operation.expects_static() {
				Operands::Static(todo!())
			} else {
				// Runs if there is a dynamic operand
				Operands::Dynamic(todo!())
			};

			// Store data.
			data = Some(Data {
				width: absolute::Type::from_exponent(data_raw.width).unwrap(),
				destination: if driver.dynamic_destination { Destination::Dynamic } else {
					Destination::Static },
				operands
			})
		}

		// Assemble
		Ok(Self {
			operation: extension,
			width: absolute::Type::Byte,
			synchronise: driver.synchronise,
			data
		})
	}

	/// Get the operand that the destination property corresponds to.
	pub fn destination(&self) -> Result<Operand, DestinationError> {
		let data = match &self.data {
			Some(data) => data,
			None => return Err(DestinationError::Data)
		};

		Ok(match data.destination {
			Destination::Static => match data.operands.x_static() {
				Some(x_static) => Operand::Static(x_static),
				None => return Err(DestinationError::Static)
			},
			Destination::Dynamic => match data.operands.x_dynamic() {
				Some(x_dynamic) => Operand::Dynamic(x_dynamic.clone()),
				None => return Err(DestinationError::Dynamic)
			}
		})
	}
}

#[cfg(test)]
mod driver_test {
	use crate::Driver;

	#[test]
	fn extract_extension() {
		assert_eq!(Driver::extract_extension(0b001101_0_0), 0b00_001101);
		assert_eq!(Driver::extract_extension(0b101010_0_1), 0b00_101010);
	}

	#[test]
	fn set_extension() {
		assert_eq!(Driver::set_extension(0b000000_0_1, 10), 0b001010_0_1);
		assert_eq!(Driver::set_extension(0b101100_0_0, 0b101100), 0b101100_0_0);
		assert_eq!(Driver::set_extension(0b101100_1_0, 0b101100), 0b101100_1_0);

		// Truncating extension
		assert_eq!(Driver::set_extension(0b00000000_0_0, 0b11_111111), 0b111111_0_0);
		assert_eq!(Driver::set_extension(0b00000000_0_1, 0b11_111110), 0b111110_0_1);
	}

	#[test]
	fn extract_synchronise() {
		assert!(Driver::extract_synchronise(0b000000_1_0));
		assert!(!Driver::extract_synchronise(0b000000_0_0));
		assert!(Driver::extract_synchronise(0b001010_1_1));
		assert!(!Driver::extract_synchronise(0b001010_0_1));
	}

	#[test]
	fn set_synchronise() {
		assert_eq!(Driver::set_synchronise(0b000000_0_0, true), 0b000000_1_0);
		assert_eq!(Driver::set_synchronise(0b000000_1_0, false), 0b000000_0_0);
		assert_eq!(Driver::set_synchronise(0b000000_0_1, true), 0b000000_1_1);
		assert_eq!(Driver::set_synchronise(0b111111_0_0, false), 0b111111_0_0);
	}

	#[test]
	fn extract_dynamic_destination() {
		assert!(Driver::extract_dynamic_destination(0b000000_0_1));
		assert!(!Driver::extract_dynamic_destination(0b000000_0_0));
		assert!(Driver::extract_dynamic_destination(0b000000_1_1));
		assert!(!Driver::extract_dynamic_destination(0b000000_1_0));
	}

	#[test]
	fn set_dynamic_destination() {
		assert_eq!(Driver::set_dynamic_destination(0b000000_0_0, true), 0b000000_0_1);
		assert_eq!(Driver::set_dynamic_destination(0b000000_1_0, true), 0b000000_1_1);
		assert_eq!(Driver::set_dynamic_destination(0b000000_0_1, false), 0b000000_0_0);
		assert_eq!(Driver::set_dynamic_destination(0b000000_1_1, false), 0b000000_1_0);
	}

	#[test]
	fn extract_operation() {
		assert_eq!(Driver::extract_operation(0b1101_00_00), 0b0000_1101);
		assert_eq!(Driver::extract_operation(0b1010_01_10), 0b0000_1010);
	}

	#[test]
	fn set_operation() {
		assert_eq!(Driver::set_operation(0b0001_00_11, 0b0000_1111), 0b1111_00_11);
		assert_eq!(Driver::set_operation(0b1111_00_10, 0b0000_1001), 0b1001_00_10);
		assert_eq!(Driver::set_operation(0b1010_00_10, 0b0000_1010), 0b1010_00_10);

		// Truncating extension
		assert_eq!(Driver::set_operation(0b0000_00_00, 0b1111_1111), 0b1111_00_00);
		assert_eq!(Driver::set_operation(0b0000_10_01, 0b1111_1111), 0b1111_10_01);
	}

	#[test]
	fn extract_addressing() {
		assert_eq!(Driver::extract_addressing(0b0011_10_00), 0b000000_10);
		assert_eq!(Driver::extract_addressing(0b1011_11_00), 0b000000_11);
		assert_eq!(Driver::extract_addressing(0b0000_00_00), 0b000000_00);
	}

	#[test]
	fn set_addressing() {
		assert_eq!(Driver::set_addressing(0b0000_11_00, 0b000000_00), 0b0000_00_00);
		assert_eq!(Driver::set_addressing(0b0011_00_00, 0b000000_01), 0b0011_01_00);
		assert_eq!(Driver::set_addressing(0b1011_00_00, 0b000000_00), 0b1011_00_00);

		// Truncating extension
		assert_eq!(Driver::set_addressing(0b0000_00_00, 0b111111_11), 0b0000_11_00);
		assert_eq!(Driver::set_addressing(0b1010_00_01, 0b111111_11), 0b1010_11_01);
	}

	#[test]
	fn extract_immediate_exponent() {
		assert_eq!(Driver::extract_immediate_exponent(0b0000_00_11), 0b000000_11);
		assert_eq!(Driver::extract_immediate_exponent(0b1010_11_01), 0b000000_01);
	}

	#[test]
	fn set_immediate_exponent() {
		assert_eq!(Driver::set_immediate_exponent(0b0011_00_00, 0b000000_11), 0b0011_00_11);
		assert_eq!(Driver::set_immediate_exponent(0b0000_11_00, 0b000000_10), 0b0000_11_10);
		assert_eq!(Driver::set_immediate_exponent(0b1011_01_00, 0b000000_00), 0b1011_01_00);

		// Truncating extension
		assert_eq!(Driver::set_immediate_exponent(0b0000_00_00, 0b111111_11), 0b0000_00_11);
		assert_eq!(Driver::set_immediate_exponent(0b1011_01_00, 0b111111_10), 0b1011_01_10);
	}

	#[test]
	fn from_encoded() {
		let driver = Driver::from_encoded([0b001010_0_1, 0b1111_10_01]);

		// Driver 0
		assert_eq!(driver.extension, 0b001010);
		assert!(!driver.synchronise);
		assert!(driver.dynamic_destination);

		// Driver 1
		assert_eq!(driver.operation, 0b1111);
		assert_eq!(driver.addressing, 0b10);
		assert_eq!(driver.immediate_exponent, 0b1);
	}

	#[test]
	fn encode() {
		let driver = Driver {
			operation: 0b1110,
			extension: 0b1010,
			synchronise: true,
			dynamic_destination: false,
			addressing: 0b11,
			immediate_exponent: 0b10
		};

		let encoded = driver.encode();

		assert_eq!(encoded[0], 0b001010_1_0);
		assert_eq!(encoded[1], 0b1110_11_10);
	}
}

#[cfg(test)]
mod raw_data_test {
	// TODO: Cover these
}

#[cfg(test)]
mod instruction_test {
	use std::io::Cursor;
	use crate::{absolute, Data, Destination, Driver, Instruction, RawData};
	use crate::operand::{AllPresent, Dynamic, IMMEDIATE_EXPONENT_BYTE, Operand, Operands};
	use crate::operation::arithmetic::Arithmetic;
	use crate::operation::Extension;

	#[test]
	fn decode() {
		let driver = Driver {
			extension: 0,
			operation: 0,
			synchronise: true,
			dynamic_destination: false,
			addressing: 0,
			immediate_exponent: 0
		}.encode();
		
		let data = RawData {
			width: 0,
			x_static: 10,
			x_dynamic: 20
		}.encode();

		let mut cursor = Cursor::new([driver[0], driver[1], data]);
		let instruction = Instruction::from_encoded(&mut cursor).unwrap();

		assert!(matches!(instruction.operation, Extension::Arithmetic(_)));
	}

	// TODO: FIX
	#[test]
	fn destination() {
	    let x_static = Instruction {
	        operation: Extension::Arithmetic(Arithmetic::Add),
	        width: absolute::Type::Byte,
			synchronise: false,
			data: Some(Data {
				width: absolute::Type::Byte,
				destination: Destination::Static,
				operands: Operands::AllPresent(AllPresent {
					x_static: 0,
					x_dynamic: Dynamic::Register(1)
				})
			})
	    };

		let mut x_dynamic = Instruction {
			operation: Extension::Arithmetic(Arithmetic::Add),
			width: absolute::Type::Byte,
			synchronise: false,
			data: Some(Data {
				width: absolute::Type::Byte,
				destination: Destination::Dynamic,
				operands: Operands::AllPresent(AllPresent {
					x_static: 0,
					x_dynamic: Dynamic::Register(1)
				})
			})
		};
	
	    assert!(matches!(x_static.destination().unwrap(), Operand::Static(_)));
	    assert!(!matches!(x_dynamic.destination().unwrap(), Operand::Static(_)));
	}
}