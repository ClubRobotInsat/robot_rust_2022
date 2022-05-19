
use crate::model::header::Header;
use crate::model::packet::Packet;

#[test]
fn test_packet_new() {
    let header = Header::new(1, 2, false, 4, 8).unwrap();
    let packet = Packet::new(header, [3;6]);
    let mess:[u8;8] = [
        0b0001_0010,
        0b1001_0000,
        3,3,3,3,3,3
    ];
    println!("Packet : {:?}", packet.get_packet_as_binary_array());
    for (p,m) in packet.get_packet_as_binary_array().iter().zip(mess.iter()) {
        assert_eq!(p,m);
    }
}