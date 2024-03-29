/*
 * Filename: data.rs
 * Description: modules for holding data from the sensor. This module also
 * impliments the cyclic redundancy check methods.
 */


#[allow(dead_code)]
const INITAL_CRC_VAL: u8 = 0xFF;
pub const CRC_INDEX: usize = 6;

const AHT20_DIVISOR: f32 = 1048576.0; 

/*
 * CRC8-MAXIM
 * Lookup table for the CRC8 values. This vastly improves the speed of the 
 * checksum process at the expense of taking up memory on the controller.
 *  0x131 = (1<<8)+(1<<5)+(1<<4)+(1<<0) = 0b0000_0001_0001_1001 =aprox= 0x31
 *
 *  POLYNOMIAL: 0x31
 *  INIT VALUE: 0xFF
 *  FINAL XOR: 0x00
 *  REFIN: False 
 *  REFOUT: False 
 */
const CRC8_MAXIM_LUT: [u8; 256] = [
0x00, 0x31, 0x62, 0x53, 0xC4, 0xF5, 0xA6, 0x97, 0xB9, 0x88, 0xDB, 0xEA, 0x7D, 0x4C, 0x1F, 0x2E,
0x43, 0x72, 0x21, 0x10, 0x87, 0xB6, 0xE5, 0xD4, 0xFA, 0xCB, 0x98, 0xA9, 0x3E, 0x0F, 0x5C, 0x6D,
0x86, 0xB7, 0xE4, 0xD5, 0x42, 0x73, 0x20, 0x11, 0x3F, 0x0E, 0x5D, 0x6C, 0xFB, 0xCA, 0x99, 0xA8,
0xC5, 0xF4, 0xA7, 0x96, 0x01, 0x30, 0x63, 0x52, 0x7C, 0x4D, 0x1E, 0x2F, 0xB8, 0x89, 0xDA, 0xEB,
0x3D, 0x0C, 0x5F, 0x6E, 0xF9, 0xC8, 0x9B, 0xAA, 0x84, 0xB5, 0xE6, 0xD7, 0x40, 0x71, 0x22, 0x13,
0x7E, 0x4F, 0x1C, 0x2D, 0xBA, 0x8B, 0xD8, 0xE9, 0xC7, 0xF6, 0xA5, 0x94, 0x03, 0x32, 0x61, 0x50,
0xBB, 0x8A, 0xD9, 0xE8, 0x7F, 0x4E, 0x1D, 0x2C, 0x02, 0x33, 0x60, 0x51, 0xC6, 0xF7, 0xA4, 0x95,
0xF8, 0xC9, 0x9A, 0xAB, 0x3C, 0x0D, 0x5E, 0x6F, 0x41, 0x70, 0x23, 0x12, 0x85, 0xB4, 0xE7, 0xD6,
0x7A, 0x4B, 0x18, 0x29, 0xBE, 0x8F, 0xDC, 0xED, 0xC3, 0xF2, 0xA1, 0x90, 0x07, 0x36, 0x65, 0x54,
0x39, 0x08, 0x5B, 0x6A, 0xFD, 0xCC, 0x9F, 0xAE, 0x80, 0xB1, 0xE2, 0xD3, 0x44, 0x75, 0x26, 0x17,
0xFC, 0xCD, 0x9E, 0xAF, 0x38, 0x09, 0x5A, 0x6B, 0x45, 0x74, 0x27, 0x16, 0x81, 0xB0, 0xE3, 0xD2,
0xBF, 0x8E, 0xDD, 0xEC, 0x7B, 0x4A, 0x19, 0x28, 0x06, 0x37, 0x64, 0x55, 0xC2, 0xF3, 0xA0, 0x91,
0x47, 0x76, 0x25, 0x14, 0x83, 0xB2, 0xE1, 0xD0, 0xFE, 0xCF, 0x9C, 0xAD, 0x3A, 0x0B, 0x58, 0x69,
0x04, 0x35, 0x66, 0x57, 0xC0, 0xF1, 0xA2, 0x93, 0xBD, 0x8C, 0xDF, 0xEE, 0x79, 0x48, 0x1B, 0x2A,
0xC1, 0xF0, 0xA3, 0x92, 0x05, 0x34, 0x67, 0x56, 0x78, 0x49, 0x1A, 0x2B, 0xBC, 0x8D, 0xDE, 0xEF,
0x82, 0xB3, 0xE0, 0xD1, 0x46, 0x77, 0x24, 0x15, 0x3B, 0x0A, 0x59, 0x68, 0xFF, 0xCE, 0x9D, 0xAC,
];


///Impliments the CRC checks, as well as sensor bitwise operations.
#[allow(dead_code)]
pub struct SensorData {
    pub bytes: [u8; 7],
    pub crc: u8,
}

#[allow(dead_code)]
impl SensorData {
    pub fn new() ->SensorData {
        let s = SensorData {
            bytes: [0u8; 7],
            crc: 0x00,
        };

        return s;
    }

    ///Uses the crc_8_maxim on the CRC byte and returns true if the calculated
    ///and received CRC bytes match.
    pub fn is_crc_good(&mut self) -> bool{
        self.crc_8_maxim();
        self.crc == self.bytes[CRC_INDEX] 
    }

    pub fn crc_8_maxim(&mut self){

        let mut crc: u16 = INITAL_CRC_VAL as u16;
        let mut index: u16;
      
        //we loop thorugh the bytes of data and XOR them to calculate the 
        //index into the lookup table.
        for b in 0..(self.bytes.len() - 1) {
            index = crc ^ (self.bytes[b] as u16);
            crc = ((CRC8_MAXIM_LUT[index as usize] as u16 ^ (crc << 8)) & 0xFF) as u16;
        }
        self.crc = crc as u8; 
    }

    pub fn clear_bytes(&mut self) {
        for v in self.bytes.iter_mut() {
            *v = 0x00 as u8;
        }
    }

    ///Gets the first 20bits of a 3 byte sequence, and typecasts it into
    ///a unsigned 32 bit integer.
    pub fn get_humidity_bits(&self) -> u32 {
        let mut h: u32 = (self.bytes[1] as u32) << 12;
        h |= (self.bytes[2] as u32) << 4;
        h |= (self.bytes[3] as u32) >> 4;
        return h
    }

    ///Gets the last 20bits of a 3 byte sequence, and typecasts it into
    ///a unsigned 32 bit integer.
    pub fn get_temperature_bits(&self) -> u32 {
        let mut t: u32 =  ((self.bytes[3] & 0x0F) as u32) << 16;
        t |= (self.bytes[4] as u32) << 8;
        t |= self.bytes[5] as u32;
        return t;
    }

    ///Uses the sensor's data-sheet formula for relative humidity %.
    pub fn calculate_humidity(&self) -> f32 {
        let mut h: f32 = ((self.get_humidity_bits()) as f32) / AHT20_DIVISOR;
        h *= 100.0;
        return h;
    }


    ///Uses the sensor's data-sheet formula for temperature in C.
    pub fn calculate_temperature(&self) -> f32 {
        let mut t: f32 = ((self.get_temperature_bits() as f32)) / AHT20_DIVISOR;
        t *= 200.0;
        t -= 50.0;
        return t;
    }

}

#[cfg(test)]
mod sensor_data_tests {
    use super::*;

    #[test]
    fn selftest() {
        assert!(true);
    }

    fn setup() -> SensorData 
    {
        let bytes_of_data: [u8; 7] = [0x18, 0x7E, 0x51, 0x65, 0xD4, 0xA0, 0xDA];
        let s = SensorData { bytes: bytes_of_data, crc: 0x00 };
        return s;
    }

    #[test]
    fn new_instance()
    {
        let _s = SensorData::new();
        assert!(true);
    }

    //This test is from the actual data saved over my logic analyzer
    #[test]
    fn crc_8_maxim_actual_one()
    {
        let bytes_of_data: [u8; 7] = [0x18, 0x80, 0x2F, 0x25, 0xD2, 0x6A, 0x13];
        let mut s = SensorData { bytes: bytes_of_data, crc: 0x00 };
        s.crc_8_maxim();
        assert_eq!(0x13, s.crc);

        s.bytes[2] = 0xAA;
        s.crc_8_maxim();
        assert_ne!(0x13, s.crc);
    }

    //This test is from the actual data saved over my logic analyzer
    #[test]
    fn crc_8_maxim_actual_two()
    {
        let bytes_of_data: [u8; 7] = [0x18, 0x7E, 0x51, 0x65, 0xD4, 0xA0, 0xDA];
        let mut s = SensorData { bytes: bytes_of_data, crc: 0x00 };
        s.crc_8_maxim();
        assert_eq!(0xDA, s.crc);
    } 

    #[test]
    fn is_crc_good_t() {
        let mut s = setup();
        assert!(s.is_crc_good());

        s.bytes[CRC_INDEX] = 0xD7;
        assert!(!s.is_crc_good());
    }

    #[test]
    fn clear_bytes() {
        let mut s = setup();        
        s.clear_bytes();

        for v in s.bytes.iter() {
            assert_eq!(*v, 0x00 as u8);
        }
    }

    #[test]
    fn split_data() {
        let mut sd = SensorData::new();
        sd.bytes = [0x18, 0x7E, 0x51, 0x65, 0xD4, 0xA0, 0xDA];

        let h = sd.get_humidity_bits();
        //This is the first 20bits after the state byte
        assert_eq!(h, 517398);


        let t = sd.get_temperature_bits();
        assert_eq!(t, 382112);
    }

    #[test]
    fn calulate_humidity() {
        let mut sd = SensorData::new();
        sd.bytes = [0x18, 0x7E, 0x51, 0x65, 0xD4, 0xA0, 0xDA];
        
        let h = sd.calculate_humidity();
        assert!(h < 49.35);
        assert!(h > 49.34);
    }

    #[test]
    fn calculate_temperature() {
        let mut sd = SensorData::new();
        sd.bytes = [0x18, 0x7E, 0x51, 0x65, 0xD4, 0xA0, 0xDA];
        
        let t = sd.calculate_temperature();
        assert!(t < 22.89);
        assert!(t > 22.87);
    }
}
