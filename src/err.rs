#[derive(Debug)]
pub enum InkErr {
    // lexer errors
    InvalidNumber(String),
    // parser errors
    UnexpectedEOF,
    UnexpectedToken,
    ExpectedCompositeValue,
    ExpectedMatchCaseArrow,
    UnexpectedArgument,
    // UnexpectedStartOfAtom,
    // compiler errors
    InvalidAssignment,
    // runtime errors
    Unimplemented,
}
