#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

//use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::blocking::i2c;

/// AHT20 Sensor Address
pub const SENSOR_ADDR: u8 = 0b0011_1000; // = 0x38

//Seventh bit indicates read/write                                        
pub const READ_MSK: u8 = 1<<0;        
pub const WRITE_MSK: u8 = 0<<0;

//Minimum startup wait time.
pub const STARTUP_DELAY_MS: u8 = 40;



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
pub enum StatusBitMasks {
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
        assert_eq!(StatusBitMasks::Busy as u8, 128);
    }
    
    #[test]
    fn check_modes() {
        assert_eq!(StatusBitMasks::NorMode as u8, 0);
        assert_eq!(StatusBitMasks::CycMode as u8, 32);
        assert_eq!(StatusBitMasks::CmdMode as u8, 64);

    }

    #[test]
    fn check_combined() {

        assert_eq!(
                StatusBitMasks::CmdMode as u8 |
                StatusBitMasks::Busy as u8,
                128 + 64
            );
    }
}

//Impliment Error type for our driver.
#[derive(Debug, PartialEq)]
pub enum Error<E> {
    I2C(E),
    InvalidChecksum,
    UnexpectedBusy,
    Internal,
}


//We have sepreate consts and enums for the puporse of being used during
//testing(consts) or as parameters(enum).
const CMD_READ_STATUS: u8 = 0x71;
const CMD_INIT_SENSOR: u8 = 0xBE;
const CMD_CALIBRATE: u8 = 0xE1;
const CMD_TRIG_MESSURE: u8 = 0xAC;
const CMD_SOFT_RESET: u8 = 0xBA;

#[repr(u8)]
pub enum Command {
    ReadStatus = CMD_READ_STATUS,
    InitSensor = CMD_INIT_SENSOR,
    Calibrate = CMD_CALIBRATE,
    TrigMessure = CMD_TRIG_MESSURE,
    SoftReset = CMD_SOFT_RESET,
}


#[allow(dead_code)]
pub struct Sensor<I2C>
where I2C: i2c::Read + i2c::Write,
{
    i2c: I2C,
    address: u8,
    buffer: [u8; 4],
}

//This stucture encapsulates the Sensor structure after the sensor
//has been initialized; enforcing correct method availbility.
#[allow(dead_code)]
pub struct InitializedSensor<'a, I2C>
where I2C: i2c::Read + i2c::Write,
{
    sensor: &'a mut Sensor<I2C>,
}


//Impliment functions for the sensor that require the embedded-hal
//I2C.
impl<E, I2C> Sensor<I2C>
where I2C: i2c::Read<Error = E> + i2c::Write<Error = E>,
{
    //We're implimenting a new function to return an instance of the sensor
    pub fn new(i2c: I2C, address: u8) -> Self {
        let buf = [0, 0, 0, 0];
        Sensor{i2c, address, buffer: buf}
    }


    pub fn init(&mut self) -> Result<InitializedSensor<I2C>, Error<E>>
    {
        let tmp_buf = [Command::InitSensor as u8];
        let _result = self.i2c.write(SENSOR_ADDR, &tmp_buf);

        Ok(InitializedSensor {sensor: self})
    }
}



impl <'a, E, I2C> InitializedSensor<'a, I2C>
where I2C: i2c::Read<Error = E> + i2c::Write<Error = E>,
{

    pub fn check_status(&mut self) -> Result< u8, Error<E> >{ 

        let write_buf = vec![Command::ReadStatus as u8];
        
        let _write_result = self.sensor.i2c 
            .write(SENSOR_ADDR, &write_buf)
            .map_err(Error::I2C)?;

        
        let mut read_buf  = [0 as u8];
        
        let _read_result = self.sensor.i2c 
            .read(SENSOR_ADDR, &mut read_buf)
            .map_err(Error::I2C)?;

        let status = read_buf[0];
        Ok(status)
    }
 
}


#[cfg(test)]
mod sensor_test {
    use embedded_hal;
    use embedded_hal_mock;

    //use embedded_hal::prelude::*;
    use embedded_hal::blocking::i2c::{Read, Write};
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use super::*;

//    fn initialized_sensor_setup(expected: &[I2cTransaction]) -> InitializedSensor<'a, I2C> {
//        //Skip doing the INIT of the sensor.
//        let i2c = I2cMock::new(expected);
//        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);
//        InitializedSensor {sensor: &mut sensor_instance}
//    }

    #[test]
    fn self_test()
    {
        assert!(true);
    }

    #[test]
    fn mocking_i2c()
    {
        let expectations = [
            I2cTransaction::write(SENSOR_ADDR, vec![1, 2]),
            I2cTransaction::read(SENSOR_ADDR, vec![3, 4]),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut buf = vec![0u8; 2];

        i2c.write(SENSOR_ADDR, &vec![1, 2]).unwrap();
        i2c.read(SENSOR_ADDR, &mut buf).unwrap();

        assert_eq!(buf, vec![3, 4]);

        i2c.done();
    }

    #[test]
    fn correct_init()
    {
        let expectations = [
            I2cTransaction::write(SENSOR_ADDR, vec![CMD_INIT_SENSOR]),
        ];
        
        let i2c = I2cMock::new(&expectations);

        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);
        let initialized_sensor_instance = sensor_instance.init();

        initialized_sensor_instance.unwrap().sensor.i2c.done();
    }

    #[test]
    fn check_status()
    {
        let sensor_status= vec![
            StatusBitMasks::CmdMode as u8 | 
            StatusBitMasks::CalEnabled as u8
            ];
        
        let expected = [
            I2cTransaction::write(SENSOR_ADDR, vec!(0x71)),
            I2cTransaction::read(SENSOR_ADDR, sensor_status),
        ];

        //Skip doing the INIT of the sensor.
        let i2c = I2cMock::new(&expected);
        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);
        let mut inited_sensor = InitializedSensor {sensor: &mut sensor_instance}; 
       
        let _r = inited_sensor.check_status();

        inited_sensor.sensor.i2c.done();
    }

    #[test]
    fn read_sensor()
    {
        
        let sensor_reading = vec![0x0, 0x0];
        
        let expected = [
            I2cTransaction::write(SENSOR_ADDR, vec!(0x71)),
            I2cTransaction::read(SENSOR_ADDR, sensor_reading),
        ];

        //Skip doing the INIT of the sensor.
        let i2c = I2cMock::new(&expected);
        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);
        let mut inited_sensor = InitializedSensor {sensor: &mut sensor_instance}; 
       
        let _r = inited_sensor.check_status();

        inited_sensor.sensor.i2c.done();
    }
}



pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
