use crate::gen::Block;

pub fn optimize(prog: Vec<Block>) -> Vec<Block> {
    return prog
        .iter()
        .map(|block| optimize_block(block.clone()))
        .collect();
}

fn optimize_block(block: Block) -> Block {
    return block;
}
