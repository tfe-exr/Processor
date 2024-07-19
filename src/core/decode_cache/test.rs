use std::io::Cursor;
use crate::dynamic_number::Size;
use crate::instruction::Instruction;
use crate::instruction::operand::{Name, Operands};
use crate::instruction::operand::dynamic::Dynamic;
use crate::instruction::operand::register::{GeneralPurpose, Register};
use crate::instruction::operation::{Arithmetic, Operation};
use super::DecodeCache;

const ADD: Instruction = Instruction {
    branch_likely_taken: None,
    execution: None,
    operation: Operation::Arithmetic(Arithmetic::Add),
    operands: Operands {
        result: Name::Register,
        register: Register::Accumulator,
        dynamic: Dynamic::Register(Register::GeneralPurpose(GeneralPurpose::G0)),
        size: Size::QuadWord,
        custom_data: false
    }
};

#[test]
fn lifetime_aging() {
    let mut cache = DecodeCache {
        decoded: Vec::new(),
        initial_lifetime: 4,
        chunk_size: 0
    };
    
    // The base address is not meant to be accurately reflective of the instruction for this test.
    cache.append(0, ADD);
    cache.append(2, ADD);
    
    assert_eq!(cache.decoded[0].lifetime, 3);
    assert_eq!(cache.decoded[1].lifetime, 4);
    
    // Kill the first instruction.
    {
        let mut cache = cache.clone();
        for _ in 0..3 { cache.age(); }
        assert!(cache.find(0).is_none());
    }
    
    // Reset the first lifetime.
    {
        let mut cache = cache.clone();
        let taken = cache.take(0).unwrap();
        assert_eq!(taken, ADD);
        assert_eq!(cache.decoded[0].lifetime, 4);
    }
}

#[test]
fn populating() {
    let mut encoded_add = Cursor::new(vec![0u8; 0]);
    ADD.encode(&mut encoded_add).unwrap();
    let mut data = encoded_add.get_ref().clone();
    data.extend(encoded_add.get_ref());
    
    let mut memory = Cursor::new(data);
    
    let mut cache = DecodeCache {
        decoded: Vec::new(),
        initial_lifetime: 4,
        chunk_size: 2
    };
    
    assert_eq!(cache.populate(&mut memory).unwrap(), 2);
}