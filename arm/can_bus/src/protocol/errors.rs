#[derive(Debug, PartialEq)]
pub enum SendError {
    DidntReceiveACK,
    SendFailed,
}

#[derive(Debug, PartialEq)]
pub enum ProtocolError {
    InvalidId(usize),
    MessageTooLong,
    ParametersTooLong,
    SrcAndDestCanNotBeEqual,
    ACKCanNotContainData,
    SendFailed(SendError),
}
