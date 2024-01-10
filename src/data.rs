/*
 * Filename: data.rs
 * Description: modules for holding data from the sensor.
 */

const INITAL_CRC_VAL: u8 = 0xFF;

pub struct SensorData {
    bytes: [u8; 6],
    crc: u16,
}

impl SensorData {
    pub fn is_crc_good(&mut self) {
        
    }

    pub fn crc8(&mut self) -> u8 {
        //according to the datasheet, after 6bytes the next(7th) byte will
        //be the crc check data.
        //let mut inital_val: u8 = INITAL_CRC_VAL; 
        return 0x00;
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
    fn crc8() 
    {
        //polynomial: 0b1000_0111 = 0x87
        let polynomial: u16 = 0x00;
        polynomial |= 0x87;

        let mut s = SensorData { bytes: [0u8; 6], crc: 0 };
        let checksum: u8 = s.crc8();
        assert_eq!(0x00, checksum);
    }

    #[test]
    fn is_crc_good() {
        //function should return a result type, along with errors,
        //otherwise unit type wrapped in ok()
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

