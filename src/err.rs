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
    // runtime errors
    Unimplemented,
}
