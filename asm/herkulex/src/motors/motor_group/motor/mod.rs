use drs_0x01::builder::HerkulexMessage;
use embedded_hal::serial::{Read, Write};
use nb::block;
use unwrap_infallible::UnwrapInfallible;


//@TODO Ajouter Tx et Rx
//@TODO Implementer send_message et read_message
pub struct Motor<'a, Tx, Rx> {
    tx : &'a mut Tx,
    rx : &'a mut Rx,
    id : u8
}

impl <Tx: Write<u8>, Rx : Read<u8>> Motor<'_, Tx, Rx> {
    pub fn new<'a>( id : u8, tx : &'a mut Tx, rx : &'a mut Rx) -> Motor<'a, Tx, Rx>{
        Motor { id, tx, rx}
    }


}