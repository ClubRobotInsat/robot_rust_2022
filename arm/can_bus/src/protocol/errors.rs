#[derive(Debug, PartialEq)]
pub enum SendError {
    DidntReceiveACK,
    SendFailed,
}

#[derive(Debug, PartialEq)]
pub enum ProtocolError {
    InvalidId(usize),
    MessageTooShort(usize),
    MessageTooLong,
    ParametersTooLong,
    SrcAndDestCanNotBeEqual,
    ACKCanNotContainData,
    SendFailed(SendError),
}
