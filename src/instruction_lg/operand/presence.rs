use crate::instruction_lg::operand::Operand;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DualInput {
    pub input_a: Operand,
    pub input_b: Operand
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct All {
    pub destination: Operand,
    pub input_a: Operand,
    pub input_b: Operand
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Combination {
    Destination(Operand),
    SingleInput(Operand),
    DualInput(DualInput),
    DestinationAndInput(DualInput),
    All(All)
}