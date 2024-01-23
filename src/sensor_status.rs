//!Bits and their meanings Check the datasheet for version 1.1
//!URL: www.aosong.com
//!
//! * bit[7]: Busy
//! * bit[6:5]: 00: NOR mode, 01: CYC mode, 1x: CMD mode 
//! * bit[4]: Reserved
//! * bit[3]: CAL Enable
//! * bit[2:0]: Reserved
//!

const BUSY_VALUE: u8 = 128;
const NORMODE_VALUE: u8 = 0;
const CYCMODE_VALUE: u8 = 32;
const CMDMODE_VALUE: u8 = 64;
const CALENABLED_VAL: u8 = 8;

pub const BUSY_BM: u8 = 1<<7;
pub const NORMODE_BM: u8 = (1<<6)|(1<<5);
pub const CYCMODE_BM: u8 = (1<<6)|(1<<5);
pub const CMDMODE_BM: u8 = 1<<6;
pub const CALENABLED_BM: u8 = 1<<3;

#[allow(dead_code)]
/// The Sensor status struct is a wraper around a u8(unsigned 8 bit integer).
/// It abstracts the needed bitwise operations into methods that can simply 
/// return a boolean.
pub struct SensorStatus {
    pub status: u8,
}

#[allow(dead_code)]
impl SensorStatus{
    pub fn new(status: u8) -> SensorStatus {
        SensorStatus{ status }  
    }

    pub fn is_busy(&self) -> bool {
        (self.status & BUSY_BM) == BUSY_VALUE
    }

    pub fn is_calibration_enabled(&self) -> bool {
        (self.status & CALENABLED_BM) == CALENABLED_VAL 
    }

    pub fn is_normal_mode(&self) -> bool {
        (self.status & NORMODE_BM) == NORMODE_VALUE
    }

    pub fn is_cyc_mode(&self) -> bool {
        (self.status & CYCMODE_BM) == CYCMODE_VALUE 
    }

    pub fn is_cmd_mode(&self) -> bool {
        (self.status & CMDMODE_BM) == CMDMODE_VALUE 
    }
}

#[cfg(test)]
mod sensor_status_tests {
    use super::*;
    
    #[test]
    fn new_status() {
        let s = SensorStatus::new(0x18);

        assert_eq!(s.status, 0x18);
        assert!(!s.is_busy());
    }

    #[test]
    fn busy_status() {
        let mut senstat = SensorStatus {status: 0x00};
        assert_eq!(senstat.status, 0x00);

        assert!(!senstat.is_busy());

        //set the busy bit.
        senstat.status |= BUSY_BM;
        assert!(senstat.is_busy());

        senstat.status |= CALENABLED_BM; 
        assert!(senstat.is_busy());
    }

    #[test]
    fn calibration_status() {
    
        let mut senstat = SensorStatus {status: 0x00};
        assert_eq!(senstat.status, 0x00);

        assert!(!senstat.is_calibration_enabled());

        //set the calibration bit
        senstat.status |= CALENABLED_BM;
        assert!(senstat.is_calibration_enabled());

        senstat.status |= BUSY_BM;
        assert!(senstat.is_calibration_enabled());
    }

    #[test]
    fn normal_mode_status() {
        //0x18 is the status the sensor returns most the time.
        let mut s = SensorStatus::new(0x18);
        assert!(s.is_normal_mode());

        s.status = s.status | (1<<6); //Hex: 0x58, DEC: 88
        assert!(!s.is_normal_mode());
    }

    #[test]
    fn cyc_mode_status() {
        //0x18 is the status the sensor returns most the time.
        let mut s = SensorStatus::new(0x18);
        assert!(!s.is_cyc_mode());

        s.status = s.status | (1<<5); //Hex: 0x38, DEC: 56 
        assert!(s.is_cyc_mode());
    }

    #[test]
    fn cmd_mode_status() {
        let mut s = SensorStatus::new(0x18);
        assert!(!s.is_cmd_mode());

        s.status = s.status | (1<<6); //Hex: 0x58, DEC: 88
        assert!(s.is_cmd_mode());
    }
}
