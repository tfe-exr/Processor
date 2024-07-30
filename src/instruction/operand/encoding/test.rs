use crate::cursor_test;
use crate::instruction::operand::{AddressingMode, ImmediateAddressing, Operand};
use crate::math::dynamic_number::{Signed, Size, Unsigned};

#[test]
fn decode_relative_addressing() {
    // +1 int_1 offset with a qword value.
    assert_eq!(cursor_test([ 0b10_110000, 0b00000001 ], Operand::decode).unwrap(), Operand {
        size: Size::U64,
        mode: AddressingMode::Immediate { mode: ImmediateAddressing::Relative {
            offset: Signed::I8(1)
        }}
    });

    // +0 int_2 offset with a qword value.
    assert_eq!(cursor_test([ 0b10_110100, 0b00000000, 0b00000000 ], Operand::decode).unwrap(), Operand {
        size: Size::U64,
        mode: AddressingMode::Immediate { mode: ImmediateAddressing::Relative {
            offset: Signed::I16(0)
        }}
    });
}

#[test]
fn decode_immediate_value() {
    // 10 uint_1 as a qword value.
    assert_eq!(cursor_test([ 0b01_110000, 0b00001010 ], Operand::decode).unwrap(), Operand {
        size: Size::U64,
        mode: AddressingMode::Immediate { mode: ImmediateAddressing::Immediate {
            immediate: Unsigned::U8(10)
        }}
    });

    // 10 uint_8 as a word value.
    assert_eq!(cursor_test([ 0b01_001100, 0b00001010, 0, 0, 0, 0, 0, 0, 0 ], Operand::decode).unwrap(), Operand {
        size: Size::U8,
        mode: AddressingMode::Immediate { mode: ImmediateAddressing::Immediate {
            immediate: Unsigned::U64(10)
        }}
    });
}