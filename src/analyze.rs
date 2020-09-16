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

// Scoping, decl annotation, and variable->register renaming: Work backwards from code generator to
// derive just the info needed in analysis.
//  - For every variable reference (ident), we need to be able to refer back to its declared
//  register allocation. This means each unique variable needs a unique ID or we need a global map
//  from parsed ident node -> Reg.

pub fn analyze(nodes: &Vec<Node>) -> Result<(), InkErr> {
    return Ok(());
}
