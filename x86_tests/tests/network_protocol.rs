
#[cfg(test)]
mod Network_protocol_tests {
    use network_protocol::{MessageSender, Read, SendError, Write};

    struct Tx {}

    impl Write for Tx {
        type Error = SendError;

        fn write(&mut self, word: u8) -> Result<(), Self::Error> {
            Ok(())
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    struct Rx {}

    impl Read for Rx {
        type Error = ();

        fn read(&mut self) -> Result<u8, Self::Error> {
            return Ok(42);
        }
    }

    #[test]
    fn message_sender_instantiation_correctly() {
        let tx = Tx {};
        let rx = Rx {};
        let sender = MessageSender::new(12, tx, rx).unwrap();
        assert_eq!(sender.get_host_id(), 12);

    }

    #[should_panic]
    #[test]
    fn message_sender_instantiation_with_overflowing_id() {
        let tx = Tx {};
        let rx = Rx {};
        let _sender = MessageSender::new(16, tx, rx).unwrap();
    }
}
