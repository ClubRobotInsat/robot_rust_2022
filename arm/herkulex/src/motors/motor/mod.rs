use crate::motors::communication::HerkulexCommunication;
use core::cell::RefCell;
use drs_0x01::{
    ReadableEEPAddr, ReadableRamAddr, Rotation, Servo, WritableEEPAddr, WritableRamAddr,
};
// use drs_0x01::*;

pub struct Motor<'a, Comm: HerkulexCommunication> {
    communication: &'a RefCell<Comm>,
    id: u8,
}

impl<'a, Comm: HerkulexCommunication> Motor<'_, Comm> {
    /// Create a new servo motor, associated with its ID.
    /// To move, enable torque.
    pub fn new(id: u8, communication: &'a RefCell<Comm>) -> Motor<'a, Comm> {
        Motor {
            id,
            communication: communication,
        }
    }

    /// Reboot the motor.
    /// Use a delay of 250ms to let the servo reboot.
    /// During the reboot all changes applied to the EEP memory will take effect.
    pub fn reboot(&self) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).reboot());
    }

    /// Clear the error register of the servo
    pub fn clear_errors(&self) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).clear_errors());
    }

    /// Enable torque to allow the servo moving
    pub fn enable_torque(&self) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).enable_torque());
    }

    /// Disable torque
    pub fn disable_torque(&self) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).disable_torque());
    }

    /// Set a certain speed.
    /// The value should be between 0 and 1023
    pub fn set_speed(&self, speed: u16, rot: Rotation) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).set_speed(speed, rot));
    }

    /// Set a position.
    /// The value should be between 21 and 1002 to avoid errors.
    pub fn set_position(&self, position: u16) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).set_position(position));
    }

    // Check that the return array is the size of the buffer
    /// Get the status of the servo.
    pub fn stat(&self) -> [u8; 20] {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).stat());
        self.communication.borrow_mut().read_message()
    }

    /// Get the ID of the servo in its RAM register.
    pub fn get_id(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::ID));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Get the ID of the servo in its EEP register.
    pub fn get_id_eep(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).eep_request(ReadableEEPAddr::ID));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the ID of the servo in its RAM register.
    /// The broadcast ID (0xFE) cannot be assigned.
    pub fn set_id(&self, id: u8) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::ID(id)));
    }

    /// Set the ID of the servo in its EEP register.
    /// To take effect, the servo should be rebooted.
    /// The broadcast ID (0xFE) cannot be assigned.
    pub fn set_id_eep(&self, id: u8) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).eep_write(WritableEEPAddr::ID(id)));
    }

    /// Get the ACK Policy of the servo set in its RAM register.
    /// The values are the following
    /// 0 : No reply to any Request Packet
    /// 1 : Only reply to Read CMD
    /// 2 : Reply to all Request Packet
    pub fn get_ack_policy(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::AckPolicy));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the ACK Policy of the servo set in its RAM register.
    /// The values are the following
    /// 0 : No reply to any Request Packet
    /// 1 : Only reply to Read CMD
    /// 2 : Reply to all Request Packet
    pub fn set_ack_policy(&self, policy: u8) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::AckPolicy(policy)));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Get the Alarm LED policy of the servo set in its RAM register.
    /// The Alarm LED policy is used when an error is detected.
    /// When LED Policy and Status Error are true, the Alarm LED starts to blink. The blink period can be changed.
    pub fn get_alarm_led_policy(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::AlarmLEDPolicy));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Get the Torque Policy of the servo set in its RAM register.
    /// Controls Torque states :
    /// - 0x40 : Break On
    /// - 0x60 : Torque On
    /// - 0x00 : Torque Free
    pub fn get_torque_policy(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::TorquePolicy));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Get the maximum temperature allowed of the servo set in its RAM register.
    /// When the temperature of the servo is greater than the maximum temperature allowed, the 'Exceed Temperature Limit' is thrown.
    /// Default value is 0xDF ( approximately 85 degrees Celcius)
    pub fn get_max_temp(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::MaxTemperature));
        self.communication.borrow_mut().read_message()[9]
    }
    /// Get the minimum voltage allowed of the servo set in its RAM register.
    /// When servo input voltage is below the minimum voltage "Exceeded Voltage Limit" error is thrown
    /// Default value is 0x5B ( approximately 6.74V).
    pub fn get_min_voltage(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::MinVoltage));
        self.communication.borrow_mut().read_message()[9]
    }
    /// Get max minimum voltage allowed of the servo set in its RAM register.
    /// When servo input voltage exceeds the maximum voltage "Exceeded Voltage Limit" error is thrown.
    /// Default value is 0x89( approximately 10.14V).
    pub fn get_max_voltage(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::MaxTemperature));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Get the current acceleration ratio of the servo set in its RAM register.
    /// Ratio of time to reach goal position to acceleration or decceleration.
    /// Acceleration is same as decceleration ratio.
    /// Maximum value is 50.
    /// Example : operating time is 100ms and acceleration is 20 => Acceleration time = 100*0.2=20ms.
    /// When the acceleration ratio is 0, speed profile is rectangle
    /// When the acceleration ratio is below 50, velocity profile is triangle.
    pub fn get_acceleration_ratio(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::AccelerationRatio));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the current acceleration ratio of the servo set in its RAM register.
    /// Ratio of time to reach goal position to acceleration or decceleration.
    /// Acceleration is same as decceleration ratio.
    /// Maximum value is 50.
    /// Example : operating time is 100ms and acceleration is 20 => Acceleration time = 100*0.2=20ms.
    /// When the acceleration ratio is 0, speed profile is rectangle
    /// When the acceleration ratio is below 50, velocity profile is triangle.
    pub fn set_acceleration_ratio(&self, ratio: u8) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::AccelerationRatio(ratio)));
    }

    /// Get the current acceleration ratio of the servo set in its RAM register.
    /// Acceleration interval is 11.2ms.
    /// When the value is 254, the maximum acceleration time is 2844ms.
    pub fn get_max_acceleration(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::MaxAcceleration));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the current acceleration ratio of the servo set in its RAM register.
    /// Acceleration interval is 11.2ms.
    /// When the value is 254, the maximum acceleration time is 2844ms.
    pub fn set_max_acceleration(&self, acceleration: u8) {
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_write(WritableRamAddr::MaxAcceleration(acceleration)),
        );
    }

    /// Get the current dead zone of the servo set in its RAM register.
    /// The dead zone only works within position control (using set_position)
    pub fn get_dead_zone(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::DeadZone));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the current dead zone of the servo set in its RAM register.
    /// The dead zone only works within position control (using set_position)
    pub fn set_dead_zone(&self, dead_zone: u8) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::DeadZone(dead_zone)));
    }

    /// Set the offset of the saturator.
    /// The saturator effect on PWM is similar to having a spring installed near the goal position.
    /// Refer to the page 36 of the manual for further information.
    pub fn get_saturator_offset(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::SaturatorOffset));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Get the offset of the saturator.
    /// The saturator effect on PWM is similar to having a spring installed near the goal position.
    /// Refer to the page 36 of the manual for further information.
    pub fn set_saturator_offset(&self, offset: u8) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::SaturatorOffset(offset)));
    }

    /// Get the slope of the saturator.
    /// The saturator effect on PWM is similar to having a spring installed near the goal position.
    /// Refer to the page 36 of the manual for further information.
    pub fn get_saturator_slope(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::SaturatorSlope));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    /// Set the slope of the saturator.
    /// The saturator effect on PWM is similar to having a spring installed near the goal position.
    /// Refer to the page 36 of the manual for further information.
    pub fn set_saturator_slope(&self, slope: u16) {
        let var1: u8 = (slope & 0xFF) as u8;
        let var2: u8 = (slope >> 8) as u8;
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_write(WritableRamAddr::SaturatorSlope(var1, var2)),
        );
    }

    /// Get the PWM Offset value.
    /// PWM will increase the output by the amount of the offset.
    /// This output could be used to act as a compensator in a system where load is on one side.
    pub fn get_pwm_offset(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::PWMOffset));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the PWM Offset value.
    /// PWM will increase the output by the amount of the offset.
    /// This output could be used to act as a compensator in a system where load is on one side.
    pub fn set_pwm_offset(&self, offset: u8) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::PWMOffset(offset)));
    }

    /// Get the minimum PWM value.
    /// The minimum PWM corresponds to the minimum torque
    /// The values range is 0x00-0xFE
    pub fn get_min_pwm(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::MinPWM));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the minimum PWM value.
    /// The minimum PWM corresponds to the minimum torque
    /// The values range is 0x00-0xFE
    pub fn set_min_pwm(&self, min_pwm: u8) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::MinPWM(min_pwm)));
    }

    /// Get the minimum PWM value.
    /// The minimum PWM corresponds to the minimum torque
    /// The values range should be 0-1024.
    pub fn get_max_pwm(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::MaxPWM));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    /// Set the minimum PWM value.
    /// The minimum PWM corresponds to the minimum torque
    /// The values range should be 0-1024.
    pub fn set_max_pwm(&self, max_pwm: u16) {
        let var1: u8 = (max_pwm & 0xFF) as u8;
        let var2: u8 = (max_pwm >> 8) as u8;
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::MaxPWM(var1, var2)));
    }

    /// Get the overload PWM Threshold in the RAM register.
    /// Overload PWM Threshold sets overload activation point.
    /// Overload activates when external force is grater than this value.
    pub fn get_overload_pwm_threshold(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::OverloadPWMThreshold));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    /// Set the overload PWM Threshold in the RAM register.
    /// Overload PWM Threshold sets overload activation point.
    /// Overload activates when external force is grater than this value.
    pub fn set_overload_pwm_threshold(&self, threshold: u16) {
        let var1: u8 = (threshold & 0xFF) as u8;
        let var2: u8 = (threshold >> 8) as u8;
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_write(WritableRamAddr::OverloadPWMThreshold(var1, var2)),
        );
    }

    /// Get the minimum operational angle in the RAM register.
    /// When requested position angle is less than the minimum position value, "Exceed Allowed POT Limit" error is thrown.
    /// Default value is 0x15 ( -159.8 degrees).
    pub fn get_min_position(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::MinPosition));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    /// Set the minimum operational angle in the RAM register.
    /// When requested position angle is less than the minimum position value, "Exceed Allowed POT Limit" error is thrown.
    /// Default value is 0x15 ( -159.8 degrees).
    pub fn set_min_position(&self, min_position: u16) {
        let var1: u8 = (min_position & 0xFF) as u8;
        let var2: u8 = (min_position >> 8) as u8;
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::MinPosition(var1, var2)));
    }

    /// Get the maximum operational angle in the RAM register.
    /// When requested position angle is less than the maximum position value, "Exceed Allowed POT Limit" error is thrown.
    /// Default value is 0x3EA ( 159.8 degrees).
    pub fn get_max_position(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::MaxPosition));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    /// Set the maximum operational angle in the RAM register.
    /// When requested position angle is less than the maximum position value, "Exceed Allowed POT Limit" error is thrown.
    /// Default value is 0x3EA ( 159.8 degrees).
    pub fn set_max_position(&self, max_position: u16) {
        let var1: u8 = (max_position & 0xFF) as u8;
        let var2: u8 = (max_position >> 8) as u8;
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::MaxPosition(var1, var2)));
    }

    /// Get the Proportional Gain in the RAM register.
    /// Increasing the proportional gain increases the response time.
    /// If the increase is too large it will result with vibration and overshoot.
    pub fn get_position_kp(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::PositionKp));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    /// Set the Proportional Gain in the RAM register.
    /// Increasing the proportional gain increases the response time.
    /// If the increase is too large it will result with vibration and overshoot.
    pub fn set_position_kp(&self, Kp: u16) {
        let var1: u8 = (Kp & 0xFF) as u8;
        let var2: u8 = (Kp >> 8) as u8;
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::PositionKp(var1, var2)));
    }

    /// Get the Derivative Gain in the RAM register.
    /// Increasing the derivative gain will suppress the over response from the Proportional gain.
    /// Instability may result if the increase is too large.
    pub fn get_position_kd(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::PositionKd));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    /// Set the Derivative Gain in the RAM register.
    /// Increasing the derivative gain will suppress the over response from the Proportional gain.
    /// Instability may result if the increase is too large.
    pub fn set_position_kd(&self, Kd: u16) {
        let var1: u8 = (Kd & 0xFF) as u8;
        let var2: u8 = (Kd >> 8) as u8;
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::PositionKd(var1, var2)));
    }

    /// Get the Integral Gain in the RAM register.
    /// Increasing the integral gain will correct small offset in steady state.
    /// Response lag may result if the increase is too large.
    pub fn get_position_ki(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::PositionKi));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    /// Set the Integral Gain in the RAM register.
    /// Increasing the integral gain will correct small offset in steady state.
    /// Response lag may result if the increase is too large.
    pub fn set_position_ki(&self, Ki: u16) {
        let var1: u8 = (Ki & 0xFF) as u8;
        let var2: u8 = (Ki >> 8) as u8;
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::PositionKi(var1, var2)));
    }

    /// Get the position feedforward first gain in the RAM register.
    /// Position Feedforward first gain is applied to increase servo response time.
    pub fn get_position_feedforward_1st_gain(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::PositionFFFirstGain));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    /// Set the position feedforward first gain in the RAM register.
    /// Position Feedforward first gain is applied to increase servo response time.
    pub fn set_position_feedforward_1st_gain(&self, position: u16) {
        let var1: u8 = (position & 0xFF) as u8;
        let var2: u8 = (position >> 8) as u8;
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_write(WritableRamAddr::PositionFFFirstGain(var1, var2)),
        );
    }

    /// Get the position feedforward second gain in the RAM register.
    /// Position Feedforward second gain is applied to increase servo response time.
    pub fn get_position_feedforward_2nd_gain(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::PositionFFSecondGain));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    /// Set the position feedforward second gain in the RAM register.
    /// Position Feedforward second gain is applied to increase servo response time.
    pub fn set_position_feedforward_2nd_gain(&self, position: u16) {
        let var1: u8 = (position & 0xFF) as u8;
        let var2: u8 = (position >> 8) as u8;
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_write(WritableRamAddr::PositionFFSecondGain(var1, var2)),
        );
    }

    /// Get the LED blink period in the RAM register.
    /// Default value is 0x2D (504ms).
    /// 0x01 is equivalent to 11.2ms.
    pub fn get_led_blink_period(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::LedBlinkPeriod));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the LED blink period in the RAM register.
    /// Default value is 0x2D (504ms).
    /// 0x01 is equivalent to 11.2ms.
    pub fn set_led_blink_period(&self, period: u8) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::LedBlinkPeriod(period)));
    }

    /// Get the ADC Fault Check Period in the RAM register.
    /// Temperature / Input voltage error check interval.
    /// Error is activated if the Temperature/Voltage error lasts longer than the check interval.
    /// Default value is 0x2D (504ms).
    pub fn get_adc_fault_detection_period(&self) -> u8 {
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_request(ReadableRamAddr::ADCFaultDetectionPeriod),
        );
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the ADC Fault Check Period in the RAM register.
    /// Temperature / Input voltage error check interval.
    /// Error is activated if the Temperature/Voltage error lasts longer than the check interval.
    /// Default value is 0x2D (504ms).
    pub fn set_adc_fault_detection_period(&self, period: u8) {
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_write(WritableRamAddr::ADCFaultDetectionPeriod(period)),
        );
    }

    /// Get the Packet Garbage Check Period in the RAM register.
    /// Incomplete Packet is deleted if it remains longer than the check interval.
    /// Default value is 0x21 (201ms).
    pub fn get_packet_garbage_detection_period(&self) -> u8 {
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_request(ReadableRamAddr::PacketGarbageDetectionPeriod),
        );
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the Packet Garbage Check Period in the RAM register.
    /// Incomplete Packet is deleted if it remains longer than the check interval.
    /// Default value is 0x21 (201ms).
    pub fn set_packet_garbage_detection_period(&self, period: u8) {
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_write(WritableRamAddr::PacketGarbageDetectionPeriod(period)),
        );
    }

    /// Get the Stop Detection Period in the RAM register.
    /// This period corresponds to the time limit which the servo stoppage is measured to determine whether it has stopped.
    pub fn get_stop_detection_period(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::StopDetectionPeriod));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the Stop Detection Period in the RAM register.
    /// This period corresponds to the time limit which the servo stoppage is measured to determine whether it has stopped.
    /// Default value is 0x1B (302ms).
    pub fn set_stop_detection_period(&self, period: u8) {
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_write(WritableRamAddr::StopDetectionPeriod(period)),
        );
    }

    /// Get the Overload Detection Period in the RAM register.
    /// This period corresponds to the time limit which the servo overload is measure to determine whether an overload has occured.
    /// Default value is 0x96 (1.68s).
    pub fn get_overload_detection_period(&self) -> u8 {
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_request(ReadableRamAddr::OverloadDetectionPeriod),
        );
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the Overload Detection Period in the RAM register.
    /// This period corresponds to the time limit which the servo overload is measure to determine whether an overload has occured.
    /// Default value is 0x96 (1.68s).
    pub fn set_overload_detection_period(&self, period: u8) {
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_write(WritableRamAddr::OverloadDetectionPeriod(period)),
        );
    }

    /// Get the Stop Threshold in the RAM register.
    /// The servo is seen as stopped when the position movement is less than the Stop Threshold value.
    pub fn get_stop_threshold(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::StopThreshold));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the Stop Threshold in the RAM register.
    /// The servo is seen as stopped when the position movement is less than the Stop Threshold value.
    pub fn set_stop_threshold(&self, threshold: u8) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::StopThreshold(threshold)));
    }

    /// Get the In Position Margin in the RAM register.
    /// Standard value to determine whether the goal position has been reached.
    /// Goal position is judged to have been reached if the deviation is less than the In Position Margin value.
    pub fn get_inposition_margin(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::InpositionMargin));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the In Position Margin in the RAM register.
    /// Standard value to determine whether the goal position has been reached.
    /// Goal position is judged to have been reached if the deviation is less than the In Position Margin value.
    pub fn set_inposition_margin(&self, margin: u8) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::InpositionMargin(margin)))
    }

    /// Get the Calibration Difference in the RAM register.
    /// Used to calibrate neutral point.
    /// It is calculated by the formula : Calibrated Pos = Absolute Pos - Calibration Difference
    pub fn get_calibration_difference(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::CalibrationDifference));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set the Calibration Difference in the RAM register.
    /// Used to calibrate neutral point.
    /// It is calculated by the formula : Calibrated Pos = Absolute Pos - Calibration Difference
    pub fn set_calibration_difference(&self, difference: u8) {
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_write(WritableRamAddr::CalibrationDifference(difference)),
        );
    }

    pub fn get_status_error(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::StatusError));
        self.communication.borrow_mut().read_message()[9]
    }

    pub fn get_status_detail(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::StatusDetail));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Get Torque Control in the RAM register.
    /// Torque states :
    /// 0x40 Break On
    /// 0x60 Torque On
    /// 0x00 Torque Free
    pub fn get_torque_control(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::TorqueControl));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set Torque Control in the RAM register.
    /// Torque states :
    /// 0x40 Break On
    /// 0x60 Torque On
    /// 0x00 Torque Free
    pub fn set_torque_control(&self, control: u8) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::TorqueControl(control)));
    }

    /// Get LED Control in the RAM regsiter.
    /// Controls of the LEDS
    /// 0x00 : Off
    /// 0x01 : Green
    /// 0x02 : Blue
    /// 0x04 : Red
    pub fn get_led_control(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::LEDControl));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Set LED Control in the RAM regsiter.
    /// Controls of the LEDS
    /// 0x00 : Off
    /// 0x01 : Green
    /// 0x02 : Blue
    /// 0x04 : Red
    pub fn set_led_control(&self, control: u8) {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_write(WritableRamAddr::LEDControl(control)));
    }

    /// Get the Voltage raw data in the RAM register.
    /// Voltage = 0.074 * ADC
    pub fn get_voltage(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::Voltage));
        self.communication.borrow_mut().read_message()[9]
    }

    /// Get the temperature raw data in the RAM register.
    pub fn get_temperature(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::Temperature));
        self.communication.borrow_mut().read_message()[9]
    }

    ///
    pub fn get_current_control_mode(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::CurrentControlMode));
        self.communication.borrow_mut().read_message()[9]
    }

    pub fn get_tick(&self) -> u8 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::Tick));
        self.communication.borrow_mut().read_message()[9]
    }

    pub fn get_calibrated_position(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::CalibratedPosition));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    pub fn get_absolute_position(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::AbsolutePosition));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    /// Get the Differential Position in the RAM Register.
    /// Show velocity measurement, velocity is measure in 11.2ms intervals.
    pub fn get_differential_position(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::DifferentialPosition));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    pub fn get_PWM(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::PWM));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    pub fn get_absolute_goal_position(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::AbsoluteGoalPosition));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    pub fn get_absolute_desired_trajectory_position(&self) -> u16 {
        self.communication.borrow_mut().send_message(
            Servo::new(self.id).ram_request(ReadableRamAddr::AbsoluteDesiredTrajectoryPosition),
        );
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }

    pub fn get_desired_velocity(&self) -> u16 {
        self.communication
            .borrow_mut()
            .send_message(Servo::new(self.id).ram_request(ReadableRamAddr::DesiredVelocity));
        let ans = self.communication.borrow_mut().read_message();
        (ans[9] + ans[10] << 8) as u16
    }
}
