#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;


use embedded_hal::i2c;
use embedded_hal::delay::DelayNs;


//Import the module with the Sensor status functions/struct
mod sensor_status;
#[allow(unused_imports)]
use crate::sensor_status::{
    SensorStatus, 
    BitMasks
};

//Import the sensor's availble i2c commands and variables
mod commands;
use crate::commands::Command;

mod data;
#[allow(unused_imports)]
use data::SensorData;


/// AHT20 Sensor Address
pub const SENSOR_ADDR: u8 = 0b0011_1000; // = 0x38

pub const STARTUP_DELAY_MS: u32 = 40;
pub const BUSY_DELAY_MS: u32 = 20;
pub const CALIBRATE_DELAY_MS: u32 = 10;
pub const MAX_STATUS_CHECK_ATTEMPTS: u32 = 3;


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
where I2C: i2c::I2c
{
    i2c: I2C,
    address: u8,
    buffer: [u8; 4],
}

//Impliment functions for the sensor that require the embedded-hal
//I2C.
impl<E, I2C> Sensor<I2C>
where I2C: i2c::I2c<Error = E>
//where I2C: i2c::Read<Error = E> + i2c::Write<Error = E>,
{

    //We're implimenting a new function to return an instance of the sensor
    pub fn new(i2c: I2C, address: u8) -> Self {
        let buf = [0, 0, 0, 0];
        Sensor{i2c, address, buffer: buf}
    }


    pub fn init(
        &mut self,
        delay: &mut impl DelayNs,
        ) -> Result<InitializedSensor<I2C>, Error<E>>
    {
        //we need a startup delay according to the datasheet.
        delay.delay_ms(STARTUP_DELAY_MS); 

       let tmp_buf = [Command::InitSensor as u8,];
        self.i2c.write(self.address, &tmp_buf).map_err(Error::I2C)?;

        let status = self.read_status()?;
        if !status.is_calibration_enabled() {
            self.calibrate(delay)?;
        }
        
        return Ok(InitializedSensor {sensor: self}); 
    }


    pub fn calibrate<D>(&mut self, delay: &mut D) -> Result<SensorStatus, Error<E>>
        where D:  DelayNs,
    {
        let wbuf = vec![Command::Calibrate as u8, 0x08, 0x00];
        self.i2c.write(self.address, &wbuf)
            .map_err(Error::I2C)?;
        
        //we wait 10ms because the data sheet say to.
        delay.delay_ms(CALIBRATE_DELAY_MS);

        let status = self.read_status()?;
        
        if status.is_calibration_enabled() {
            return Ok(status);
        }
        return Err(Error::Internal);
    }


    pub fn read_status(&mut self) -> Result<SensorStatus, Error<E>>
    {
        self.i2c 
            .write(self.address, &[Command::ReadStatus as u8])
            .map_err(Error::I2C)?;
        

        let mut buf = [0];
        //now try to read it.
        self.i2c
            .read(self.address, &mut buf)
            .map_err(Error::I2C)?;

        Ok(SensorStatus{ status: buf[0]})
    }
}


//This stucture encapsulates the Sensor structure after the sensor
//has been initialized; enforcing correct method availbility.
#[allow(dead_code)]
pub struct InitializedSensor<'a, I2C>
where I2C: i2c::I2c
{
    sensor: &'a mut Sensor<I2C>,
}



impl <'a, E, I2C> InitializedSensor<'a, I2C>
where I2C: i2c::I2c<Error = E>
//where I2C: i2c::Read<Error = E> + i2c::Write<Error = E>,
{
    pub fn get_status(&mut self) -> Result<SensorStatus, Error<E> >{ 
        let s = self.sensor.read_status()?;
        Ok(s)
    }

    pub fn read_sensor(
        &mut self,
        delay: &mut impl DelayNs,
        ) -> Result< [u8; 7], Error<E>> {
        //check to make sure the sensor isn't busy.
        self.get_status()?;


        //The datasheet calls the 0x33 & 0x00 bytes parameters of it.
        //Doesn't really say or explain anything else about it.
        let wbuf = vec![Command::TrigMessure as u8, 0x33, 0x00];
        self.sensor.i2c.write(self.sensor.address, &wbuf)
            .map_err(Error::I2C)?;

        //wait for the messurement.
        delay.delay_ms(BUSY_DELAY_MS);

        //read sensor
        let rbuf = [0u8; 7];

        Ok(rbuf)
    }

}


#[cfg(test)]
mod sensor_test {
    use embedded_hal;
    use embedded_hal_mock;

    //use embedded_hal::prelude::*;
    use embedded_hal::i2c::I2c;
   
    use embedded_hal_mock::eh1::i2c::{
        Mock as I2cMock,
        Transaction as I2cTransaction,
    };
    
    //use embedded_hal_mock::timer;
    use embedded_hal_mock::eh1::delay;

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
        assert!(!results.unwrap().is_busy());
        sensor_instance.i2c.done();
    }

    #[test]
    fn calibrate()
    {
        let expectations = [
            I2cTransaction::write(SENSOR_ADDR, vec![Command::Calibrate as u8, 0x08, 0x00]),
            I2cTransaction::write(SENSOR_ADDR, vec![Command::ReadStatus as u8]),
            I2cTransaction::read(SENSOR_ADDR, vec![BitMasks::Busy as u8]),
            I2cTransaction::write(SENSOR_ADDR, vec![Command::Calibrate as u8, 0x08, 0x00]),
            I2cTransaction::write(SENSOR_ADDR, vec![Command::ReadStatus as u8]),
            I2cTransaction::read(SENSOR_ADDR, vec![BitMasks::CalEnabled as u8]),
        ]; 

        let i2c = I2cMock::new(&expectations);
        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);


        let mut mock_delay = delay::NoopDelay;
        let mut results = sensor_instance.calibrate(&mut mock_delay);
        assert!(results.is_err());

        results = sensor_instance.calibrate(&mut mock_delay);
        assert!(results.is_ok());

        sensor_instance.i2c.done();
    }

    #[test]
    fn get_status_busy()
    {
        let busy_status: u8 = BitMasks::Busy as u8;

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
        assert!(results.unwrap().is_busy());
        sensor_instance.i2c.done();
    }

    #[test]
    fn correct_init()
    {

        let calibrated = vec![
           (BitMasks::CalEnabled as u8)
        ];
        assert_eq!(calibrated[0], 0b0000_1000);

        let not_calibrated = vec![0];
        assert_eq!(not_calibrated[0], 0b0000_0000);

        let expectations = [
            I2cTransaction::write(
                SENSOR_ADDR, vec![Command::InitSensor as u8]),
            I2cTransaction::write(
                SENSOR_ADDR, vec![Command::ReadStatus as u8]),
            I2cTransaction::read(
                SENSOR_ADDR, not_calibrated.clone()),
            I2cTransaction::write(
                SENSOR_ADDR, vec![Command::Calibrate as u8, 0x08, 0x00]),
            I2cTransaction::write(
                SENSOR_ADDR, vec![Command::ReadStatus as u8]),
            I2cTransaction::read(
                SENSOR_ADDR, calibrated.clone()),
        ];
        
        let i2c = I2cMock::new(&expectations);

        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);

        let mut mock_delay = delay::NoopDelay;
        let initialized_sensor_instance = sensor_instance.init(&mut mock_delay);
        
        assert!(initialized_sensor_instance.is_ok());

        initialized_sensor_instance.unwrap().sensor.i2c.done();
    }


    #[test]
    fn get_initialized_status()
    {
        let wbuf = vec![Command::ReadStatus as u8];
        let sensor_status= vec![
            BitMasks::CmdMode as u8 | 
            BitMasks::CalEnabled as u8
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
        assert_eq!(r.unwrap().status, sensor_status[0]);
        inited_sensor.sensor.i2c.done();
    }

}


#[cfg(test)]
mod initialized_sensor_tests {
    use embedded_hal_mock;

    use embedded_hal_mock::eh1::i2c::{
        Mock as I2cMock, 
        Transaction as I2cTransaction
    };
    
    use embedded_hal_mock::eh1::delay;

    use super::*;

    #[test]
    fn read_sensor()
    {
        /*
         * Test list:
         * -reads status
         * -ensures not busy
         * -triggers messurement
         * -waits till measurrement done
         * -reads all data.
         * -checks for errors
         * -returns data.
         */
        //prepare 7-Bytes of data.
        //Byte 0: State,
        //Byte 1: Humid 0
        //Byte 2: Humid 1
        //Byte 3: Humid 2
        //Byte 4: Temp 0
        //Byte 5: Temp 1
        let sensor_reading = vec![0u8; 7];
        
        let busy_status = vec![BitMasks::Busy as u8];
        let not_busy_status = vec![0x00];

        /*
         * Transactions:
         * START_MESSUREMENT
         * Tx: 0x70
         * Tx: 0xAC
         * Tx: 0x33
         * Tx: 0x00
         * 
         * CHECK FOR DONE:
         *
         */
        let expected = [
            I2cTransaction::write(SENSOR_ADDR, vec![commands::READ_STATUS]),
            I2cTransaction::read(SENSOR_ADDR, not_busy_status.clone()),
            I2cTransaction::write(SENSOR_ADDR, vec![commands::TRIG_MESSURE, 0x33, 0x00]),
            I2cTransaction::write(SENSOR_ADDR, vec![commands::READ_STATUS]),
            I2cTransaction::read(SENSOR_ADDR, busy_status),
            I2cTransaction::write(SENSOR_ADDR, vec![commands::READ_STATUS]),
            I2cTransaction::read(SENSOR_ADDR, not_busy_status),
            I2cTransaction::read(SENSOR_ADDR, sensor_reading),
        ];

        //Skip doing the INIT of the sensor.
        let i2c = I2cMock::new(&expected);
        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);
        let mut inited_sensor = InitializedSensor {
            sensor: &mut sensor_instance
        }; 
        
        let mut mock_delay = delay::NoopDelay;
        let data = inited_sensor.read_sensor(&mut mock_delay);
        
        assert!(data.is_ok());

        inited_sensor.sensor.i2c.done();
    }
    
}
