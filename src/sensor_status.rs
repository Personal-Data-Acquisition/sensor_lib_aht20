//Bits and their meanings Check the datasheet for version 1.1
//URL: www.aosong.com
/*
 * bit[7]: Busy
 * bit[6:5]: 00: NOR mode, 01: CYC mode, 1x: CMD mode 
 * bit[4]: Reserved
 * bit[3]: CAL Enable
 * bit[2:0]: Reserved
*/

//This means it's a primitive enum representation; aka uint8_t
#[repr(u8)]
#[allow(dead_code)]
pub enum BitMasks {
    Busy = (1 << 7),
    NorMode = ((0 << 6) |( 0 << 5)),
    CycMode = ((0 << 6) |( 1 << 5)),
    CmdMode = (1 << 6),
    CalEnabled = (1 << 3),
}

#[cfg(test)]
mod test_bitmaks {
    use super::*;

    #[test]
    fn check_busy() {
        assert_eq!(BitMasks::Busy as u8, 128);
    }
    
    #[test]
    fn check_modes() {
        assert_eq!(BitMasks::NorMode as u8, 0);
        assert_eq!(BitMasks::CycMode as u8, 32);
        assert_eq!(BitMasks::CmdMode as u8, 64);

    }

    #[test]
    fn check_combined() {

        assert_eq!(
                BitMasks::CmdMode as u8 |
                BitMasks::Busy as u8,
                128 + 64
            );
    }
}

#[allow(dead_code)]
pub struct SensorStatus {
    status: u8,
}

#[allow(dead_code)]
impl SensorStatus{
    pub fn is_busy(&self) -> bool {
        if self.status & BitMasks::Busy as u8 > 0 {
            return true;
        }
        return false;
    }

    pub fn is_calibration_enabled(&self) -> bool {
        if self.status & BitMasks::CalEnabled as u8 > 0 {
            return true
        }
        return false 
    }
}

#[cfg(test)]
mod sensor_status_tests {
    use super::*;

    #[test]
    fn busy_status() {
        let mut senstat = SensorStatus {status: 0x00};
        assert_eq!(senstat.status, 0x00);

        assert!(!senstat.is_busy());

        //set the busy bit.
        senstat.status |= BitMasks::Busy as u8;
        assert!(senstat.is_busy());

        senstat.status |= BitMasks::CalEnabled as u8;
        assert!(senstat.is_busy());
    }

    #[test]
    fn calibration_status() {
    
        let mut senstat = SensorStatus {status: 0x00};
        assert_eq!(senstat.status, 0x00);

        assert!(!senstat.is_calibration_enabled());

        //set the calibration bit
        senstat.status |= BitMasks::CalEnabled as u8;
        assert!(senstat.is_calibration_enabled());

        senstat.status |= BitMasks::Busy as u8;
        assert!(senstat.is_calibration_enabled());
    }
}

