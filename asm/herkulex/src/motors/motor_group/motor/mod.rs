use drs_0x01::builder::HerkulexMessage;
use embedded_hal::serial::{Read, Write};
use stm32f1xx_hal::serial::{Rx, Tx};
use stm32f1xx_hal::stm32::USART1;


//@TODO Ajouter Tx et Rx
//@TODO Implementer send_message et read_message
pub struct Motor<'a> {
    tx : &'a mut Tx<USART1, u8>,
    rx : &'a mut Rx<USART1, u8>,
    id : u8
}

impl Motor<'_> {
    pub fn new<'a>( id : u8, tx : &'a mut Tx<USART1, u8>, rx : &'a mut Rx<USART1, u8>) -> Motor<'a>{
        Motor { id, tx, rx}
    }


}