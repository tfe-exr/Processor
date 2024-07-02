use emulator::memory::Memory;
use super::processor::instruction::{Instruction, operation::Operation};

pub mod array;
pub mod instruction;

/// Ports list for input and output.
pub type Ports = [u8; 8];

/// Registers array.
pub type Registers = [u64; 8];

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Core {
    pub context: Context,
}

/// Context objects for units outside of a processor core.
#[derive(Debug, Clone, Default)]
pub struct ExternalContext {
    pub memory: Memory,
    pub ports: Ports
}

/// The execution context of an individual core.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Context {
    pub registers: Registers,
    /// Whether virtual memory address translation is enabled.
    pub virtual_mode: bool,
    /// Points to the start of the current instruction that should be decoded.
    pub instruction_pointer: u64
}

impl Core {
    /// Execute an instruction and see if the processor must halt. Doing this could modify the execution context.
    pub fn execute(&mut self, instruction: &Instruction, external_context: &mut ExternalContext) -> bool {
        instruction.extension().operation().execute(instruction.data().as_ref(), &mut self.context, external_context).expect("TODO: panic message");
        false
    }
}
