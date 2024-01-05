/*
 * Filename: data.rs
 * Description: modules for holding data from the sensor.
 */

pub struct SensorData {
    bytes: [u8; 6],
    crc: u16,
}

impl SensorData {
    pub fn check_crc(&mut self) {
        
    }

    pub fn crc(&mut self) {

    }

    pub fn clear_bytes(&mut self) {

    }

}

#[cfg(test)]
mod sensor_data_tests {
    use super::*;

    #[test]
    fn selftest() {
        assert!(false);
    }

    #[test]
    fn crc() {
        assert!(false);
    }

    #[test]
    fn crc_check() {
        assert!(false);
    }

    #[test]
    fn clear_bytes() {
        assert!(false);
    }
}
