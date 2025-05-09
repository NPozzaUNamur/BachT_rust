#[derive(Debug)]
pub enum CLIError {
    ParseError(String),
    UnknownPrimitive(String),
    CommuncationError(String),
}