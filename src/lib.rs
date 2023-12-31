#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

use embedded_hal::blocking::{
    i2c,
    delay::DelayMs,
    delay::DelayUs,
};

//Import the module with the Sensor status functions/struct
mod sensor_status;

//Import the sensor's availble i2c commands and variables
mod commands;
use crate::commands::Command;


/// AHT20 Sensor Address
pub const SENSOR_ADDR: u8 = 0b0011_1000; // = 0x38

pub const STARTUP_DELAY_MS: u16 = 40;
pub const BUSY_DELAY_MS: u16 = 20;
pub const MAX_STATUS_CHECK_ATTEMPTS: u16 = 3;


//Impliment Error type for our driver.
#[derive(Debug, PartialEq)]
pub enum Error<E> {
    I2C(E),
    InvalidChecksum,
    UnexpectedBusy,
    Internal,
    DeviceTimeOut
}


#[allow(dead_code)]
pub struct Sensor<I2C>
where I2C: i2c::Read + i2c::Write,
{
    i2c: I2C,
    address: u8,
    buffer: [u8; 4],
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


    pub fn init(
        &mut self,
        delay: &mut (impl DelayUs<u16> + DelayMs<u16>),
        ) -> Result<InitializedSensor<I2C>, Error<E>>
    {
        //we need a startup delay according to the datasheet.
        delay.delay_ms(STARTUP_DELAY_MS);
             
        let tmp_buf = [Command::InitSensor as u8,];
        self.i2c.write(self.address, &tmp_buf).map_err(Error::I2C)?;

        //check if the status is good.
        let mut status = self.read_status()?;
        for _i in 0..MAX_STATUS_CHECK_ATTEMPTS {
            if (status & (sensor_status::BitMasks::Busy as u8)) != 0 {
                delay.delay_ms(BUSY_DELAY_MS); 
                status = self.read_status()?;
            }
            else {
                return Ok(InitializedSensor {sensor: self});
            }
        }
        return Err(Error::DeviceTimeOut);
    }

    pub fn read_status(&mut self) -> Result<u8, Error<E>>
    {
        self.i2c 
            .write(self.address, &[Command::ReadStatus as u8])
            .map_err(Error::I2C)?;
        

        let mut buf = [0];
        //now try to read it.
        self.i2c
            .read(self.address, &mut buf)
            .map_err(Error::I2C)?;

        Ok(buf[0])
    }
}


//This stucture encapsulates the Sensor structure after the sensor
//has been initialized; enforcing correct method availbility.
#[allow(dead_code)]
pub struct InitializedSensor<'a, I2C>
where I2C: i2c::Read + i2c::Write,
{
    sensor: &'a mut Sensor<I2C>,
}



impl <'a, E, I2C> InitializedSensor<'a, I2C>
where I2C: i2c::Read<Error = E> + i2c::Write<Error = E>,
{
    pub fn get_status(&mut self) -> Result< u8, Error<E> >{ 
        let s = self.sensor.read_status()?;

        Ok(s)
    }

    pub fn read_sensor(&mut self) -> Result< [u8; 2], Error<E>> {
        Ok([0, 0])
    }

}


#[cfg(test)]
mod sensor_test {
    use embedded_hal;
    use embedded_hal_mock;

    //use embedded_hal::prelude::*;
    use embedded_hal::blocking::i2c::{Read, Write};
    
    use embedded_hal_mock::i2c::{
        Mock as I2cMock, 
        Transaction as I2cTransaction
    };
    
    //use embedded_hal_mock::timer;
    use embedded_hal_mock::delay;

    use super::*;

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
    fn get_status()
    {
        let not_busy_status: u8 = 0x00;

        let expectations = [
            I2cTransaction::write(
                SENSOR_ADDR, 
                vec![Command::ReadStatus as u8]
                ),
            I2cTransaction::read(
                SENSOR_ADDR,
                vec![not_busy_status]),
        ];


        let i2c = I2cMock::new(&expectations);
        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);

        let results = sensor_instance.read_status();
        
        assert!(results.is_ok());
        assert_eq!(results.unwrap(), not_busy_status);
    }

    #[test]
    fn get_status_busy()
    {
        let busy_status: u8 = sensor_status::BitMasks::Busy as u8;

        let expectations = [
            I2cTransaction::write(
                SENSOR_ADDR, 
                vec![Command::ReadStatus as u8]
                ),
            I2cTransaction::read(
                SENSOR_ADDR,
                vec![busy_status]),
        ];


        let i2c = I2cMock::new(&expectations);
        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);

        let results = sensor_instance.read_status();
        
        assert!(results.is_ok());
        assert_eq!(results.unwrap(), busy_status);
    }

    #[test]
    fn correct_init()
    {

        let busy_status = vec![
            sensor_status::BitMasks::Busy as u8 
        ];

        let not_busy_status = vec![
            !(sensor_status::BitMasks::Busy as u8) 
        ];

        let expectations = [
            I2cTransaction::write(
                SENSOR_ADDR, vec![Command::InitSensor as u8]
                ),

            I2cTransaction::write(
                SENSOR_ADDR, vec![Command::ReadStatus as u8]
                ),
            I2cTransaction::read(
                SENSOR_ADDR, busy_status.clone()
                ),

            I2cTransaction::write(
                SENSOR_ADDR, vec![Command::ReadStatus as u8]
                ),
            I2cTransaction::read(
                SENSOR_ADDR, busy_status.clone()
                ),

            I2cTransaction::write(
                SENSOR_ADDR, vec![Command::ReadStatus as u8]
                ),
            I2cTransaction::read(
                SENSOR_ADDR, not_busy_status.clone()
                ),
        ];
        
        let i2c = I2cMock::new(&expectations);

        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);

        let mut mock_delay = delay::MockNoop;
        let initialized_sensor_instance = sensor_instance.init(&mut mock_delay);

        initialized_sensor_instance.unwrap().sensor.i2c.done();
    }

    #[test]
    fn timed_out_init()
    {

        let busy_status: u8 = sensor_status::BitMasks::Busy as u8;

        let expectations = [
            I2cTransaction::write(
                SENSOR_ADDR,
                vec![Command::InitSensor as u8]
                ),
            I2cTransaction::write(
                SENSOR_ADDR, 
                vec![Command::ReadStatus as u8]
                ),
            I2cTransaction::read(SENSOR_ADDR, vec![busy_status]),

            I2cTransaction::write(
                SENSOR_ADDR, 
                vec![Command::ReadStatus as u8]
                ),
            I2cTransaction::read(SENSOR_ADDR, vec![busy_status]),

            I2cTransaction::write(
                SENSOR_ADDR, 
                vec![Command::ReadStatus as u8]
                ),
            I2cTransaction::read(SENSOR_ADDR, vec![busy_status]),

            I2cTransaction::write(
                SENSOR_ADDR, 
                vec![Command::ReadStatus as u8]
                ),
            I2cTransaction::read(SENSOR_ADDR, vec![busy_status]),
        ];


        let i2c = I2cMock::new(&expectations);
        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);
 
        let mut mock_delay = delay::MockNoop;
        let initialized_sensor_instance = sensor_instance.init(&mut mock_delay);       
       
        assert!(initialized_sensor_instance.is_err());
    }


    #[test]
    fn get_initialized_status()
    {
        let wbuf = vec![Command::ReadStatus as u8];
        let sensor_status= vec![
            sensor_status::BitMasks::CmdMode as u8 | 
            sensor_status::BitMasks::CalEnabled as u8
            ];
        
        let expected = [
            I2cTransaction::write(SENSOR_ADDR, wbuf),
            I2cTransaction::read(SENSOR_ADDR, sensor_status.clone()),
        ];

        //Skip doing the INIT of the sensor.
        let i2c = I2cMock::new(&expected);
        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);
        let mut inited_sensor = InitializedSensor {
            sensor: &mut sensor_instance
        }; 
       
        let r = inited_sensor.get_status();

        inited_sensor.sensor.i2c.done();
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), sensor_status[0])
    }

    #[test]
    fn read_sensor()
    {
        
        //prepare 7-Bytes of data.
        let sensor_reading = vec![0u8; 7];
        
        let busy_status = vec![
            (sensor_status::BitMasks::Busy as u8) & 0x0
        ];
        let not_busy_status = vec![
            !(sensor_status::BitMasks::Busy as u8) & 0x0
        ];

        let expected = [
            I2cTransaction::write(SENSOR_ADDR, vec!(0xAC)),
            I2cTransaction::write(SENSOR_ADDR, vec!(0x71)),
            I2cTransaction::read(SENSOR_ADDR, busy_status),
            I2cTransaction::read(SENSOR_ADDR, not_busy_status),
            I2cTransaction::read(SENSOR_ADDR, sensor_reading),
        ];

        //Skip doing the INIT of the sensor.
        let i2c = I2cMock::new(&expected);
        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);
        let mut inited_sensor = InitializedSensor {
            sensor: &mut sensor_instance
        }; 
       
        let _r = inited_sensor.get_status();

        inited_sensor.sensor.i2c.done();
    }
}





