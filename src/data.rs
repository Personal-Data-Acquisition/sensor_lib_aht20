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
        for v in self.bytes.iter_mut() {
            *v = 0x00 as u8;
        }
    }

}

#[cfg(test)]
mod sensor_data_tests {
    use super::*;

    #[test]
    fn selftest() {
        assert!(true);
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
        let mut s = SensorData { bytes: [0u8; 6], crc: 0 };
        s.bytes[0] = 0xFF;
        
        s.clear_bytes();

        for v in s.bytes.iter() {
            assert_eq!(*v, 0x00 as u8);
        }
    }
}
