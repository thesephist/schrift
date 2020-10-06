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
    // analyzer errors
    UndefinedVariable,
    // compiler errors
    InvalidAssignment,
    // runtime errors
    InvalidOperand,
    InvalidFunctionCall,
    Unimplemented,
    InvalidArguments,
    NotEnoughArguments,
    IOError,
    ExpectedIntegerIndex,
    IndexOutOfBounds,
    ExpectedString,
}
