#[derive(Debug)]
pub enum InkErr {
    // lexer errors
    InvalidNumber(String),
    // parser errors
    UnexpectedEOF,
    UnexpectedToken,
    // UnexpectedStartOfAtom,
    // runtime errors
    Unimplemented,
}
