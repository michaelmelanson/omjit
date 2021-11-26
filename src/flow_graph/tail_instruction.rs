use super::BasicBlockId;

#[derive(Debug, Clone)]
pub enum TailInstruction {
    Jump(BasicBlockId),
    ConditionalJump(BasicBlockId)
}
