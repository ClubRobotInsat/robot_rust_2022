#[cfg(test)]
use crate::model::header::Header;

#[test]
pub fn new_from_binary_array_test() {
    #[allow(clippy::all)]
    let sample_data = [0xFD,0b001_0010_1];
    let header = Header::new_from_binary_array(&sample_data);
    assert_eq!(header.get_id_dest(), 0x0F);
    assert_eq!(header.get_id_src(), 0x0D);
    assert_eq!(header.get_id_message(), 0b001);
    assert_eq!(header.get_seq_number(), 0b0010);
    assert!(header.get_is_ack());
}