use drs_0x01;

pub struct Servo {
    id : u8,
}

/// Use the broadcast ID by default
impl Default for Servo {
    fn default() -> Self {
        Servo { id: 0xFE }
    }
}

impl Servo {
    /// Create a new Servo with the given ID
    pub fn new(id : u8) -> Servo {
        Servo { id }
    }

    /// Change the ID of the servo
    pub fn set_id(&mut self, id: u8) {

    }

    /// Get current ID
    pub fn get_id(self) {

    }

    /// Get the number of servos using this id
    pub fn get_nb_servo_with_id(self){

    }

    pub fn reboot(self) {

    }

    /// Functions :
    ///     - init
    ///     - reboot
    ///     - clear errors
    ///     - enable torque
    ///     - disable torque
    ///     - set_position
    ///     - set_speed
    ///     - stat
    ///     - set_id_eeprom
    pub fn err() {

    }

}