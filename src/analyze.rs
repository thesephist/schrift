use crate::err::InkErr;
use crate::parse::Node;

// Stack: Vec<Frame>
//  - A function body is always normalized during analysis to an expression group,
//      so we only need logic to codegen for exprlists with optional arguments
//
// In order to produce this bytecode stream, the analyzer must ensure
// these invariants:
//  - Every function body must be an ExprList
//  - Every match clause body must be an ExprList

pub fn analyze(nodes: &Vec<Node>) -> Result<(), InkErr> {
    return Ok(());
}
