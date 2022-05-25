#[derive(Debug, PartialEq)]
pub enum SendError {
    DidntReceiveACK,
    SendFailed,
}

#[derive(Debug, PartialEq)]
pub enum MessageCreationError {
    MessageTooLong,
    ParametersTooLong,
    SrcAndDestCanNotBeEqual,
    ACKCanNotContainData,
    SendFailed(SendError),
}
