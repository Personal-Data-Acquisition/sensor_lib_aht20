#![cfg_attr(not(test), no_std)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;


use embedded_hal::blocking::{
    i2c,
    delay::DelayMs,
};

//Import the module with the Sensor status functions/struct
mod sensor_status;
#[allow(unused_imports)]
use crate::sensor_status::SensorStatus;

//Import the sensor's availble i2c commands and variables
mod commands;
use crate::commands::Command;

mod data;
#[allow(unused_imports)]
use data::SensorData;


/// AHT20 Sensor Address
pub const SENSOR_ADDR: u8 = 0b0011_1000; // = 0x38

pub const STARTUP_DELAY_MS: u16 = 40;
pub const BUSY_DELAY_MS: u16 = 20;
pub const MEASURE_DELAY_MS: u16 = 80;
pub const CALIBRATE_DELAY_MS: u16 = 10;

pub const MAX_ATTEMPTS: usize = 3;

// Described by the datasheet as parameters.
pub const TRIG_MEASURE_PARAM0: u8 = 0x33;
pub const TRIG_MEASURE_PARAM1: u8 = 0x00;


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
        delay: &mut impl DelayMs<u16>,
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
        where D:  DelayMs<u16>,
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
where I2C: i2c::Read + i2c::Write,
{
    sensor: &'a mut Sensor<I2C>,
}



impl <'a, E, I2C> InitializedSensor<'a, I2C>
where I2C: i2c::Read<Error = E> + i2c::Write<Error = E>,
{
    pub fn get_status(&mut self) -> Result<SensorStatus, Error<E> >{ 
        let s = self.sensor.read_status()?;
        Ok(s)
    }
    
    pub fn trigger_measurement(&mut self) -> Result<(), Error<E>> 
    {
        let wbuf = vec![Command::TrigMessure as u8,
            TRIG_MEASURE_PARAM0,
            TRIG_MEASURE_PARAM1];
        self.sensor.i2c
            .write(self.sensor.address, &wbuf)
            .map_err(Error::I2C)?;
        
        Ok(())
    }


    pub fn read_sensor(
        &mut self,
        delay: &mut impl DelayMs<u16>,
        ) -> Result<SensorData, Error<E>> {
        
        self.trigger_measurement()?;
        
        delay.delay_ms(MEASURE_DELAY_MS);

        let mut sd = SensorData::new();

        //Limits the number of times it tries to get status
        for attempt in 0..MAX_ATTEMPTS{
            
            self.sensor.i2c.read(self.sensor.address, &mut sd.bytes)
                .map_err(Error::I2C)?;

            if sensor_status::BUSY_BM as u8 & sd.bytes[0] == 0 {
                break;
            }
            else if attempt == MAX_ATTEMPTS {
                return Err(Error::DeviceTimeOut);
            }
            delay.delay_ms(BUSY_DELAY_MS);
        }

        //check against the CRC?
        Ok(sd)
    }

    pub fn soft_reset(&mut self, _delay: &mut impl DelayMs<u16>) ->
        Result<SensorStatus, Error<E>>
    {
        
        let mut status =  self.get_status()?;
        if status.is_busy() {
            return Err(Error::UnexpectedBusy);
        }

        let wbuf = vec![Command::SoftReset as u8];
        self.sensor.i2c.write(self.sensor.address, &wbuf)
            .map_err(Error::I2C)?;

        status =  self.get_status()?;
        return Ok(status);
    }

}


#[cfg(test)]
mod sensor_test {
    use embedded_hal::prelude::*;
    use embedded_hal_mock::i2c::{
        Mock as I2cMock,
        Transaction as I2cTransaction,
    };
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
            I2cTransaction::read(SENSOR_ADDR, vec![sensor_status::BUSY_BM as u8]),
            I2cTransaction::write(SENSOR_ADDR, vec![Command::Calibrate as u8, 0x08, 0x00]),
            I2cTransaction::write(SENSOR_ADDR, vec![Command::ReadStatus as u8]),
            I2cTransaction::read(SENSOR_ADDR, vec![sensor_status::CALENABLED_BM as u8]),
        ]; 

        let i2c = I2cMock::new(&expectations);
        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);


        let mut mock_delay = embedded_hal_mock::delay::MockNoop;
        let mut results = sensor_instance.calibrate(&mut mock_delay);
        assert!(results.is_err());

        results = sensor_instance.calibrate(&mut mock_delay);
        assert!(results.is_ok());

        sensor_instance.i2c.done();
    }

    #[test]
    fn get_status_busy()
    {
        let busy_status: u8 = sensor_status::BUSY_BM as u8;

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
           (sensor_status::CALENABLED_BM as u8)
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

        let mut mock_delay = embedded_hal_mock::delay::MockNoop;
        let initialized_sensor_instance = sensor_instance.init(&mut mock_delay);
        
        assert!(initialized_sensor_instance.is_ok());

        initialized_sensor_instance.unwrap().sensor.i2c.done();
    }


    #[test]
    fn get_initialized_status()
    {
        let wbuf = vec![Command::ReadStatus as u8];
        let sensor_status= vec![
            sensor_status::CMDMODE_BM as u8 | 
            sensor_status::CALENABLED_BM as u8
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

        assert!(r.is_ok());
        assert_eq!(r.unwrap().status, sensor_status[0]);
        inited_sensor.sensor.i2c.done();
    }

}


#[cfg(test)]
mod initialized_sensor_tests {
    use embedded_hal_mock;

    use embedded_hal_mock::i2c::{
        Mock as I2cMock, 
        Transaction as I2cTransaction
    };
    
    use super::*;
    
    #[test]
    fn trigger_messurement() 
    {
        let expected = [
            I2cTransaction::write(SENSOR_ADDR, vec![
                                  commands::TRIG_MESSURE,
                                  TRIG_MEASURE_PARAM0,
                                  TRIG_MEASURE_PARAM1,
            ]),
        ];
        
        //Skip doing the INIT of the sensor.
        let i2c = I2cMock::new(&expected);
        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);
        let mut inited_sensor = InitializedSensor {
            sensor: &mut sensor_instance
        }; 
        
        let res = inited_sensor.trigger_measurement();
        assert!(res.is_ok());

        inited_sensor.sensor.i2c.done();

    }

    #[test]
    fn read_sensor()
    {

        let busy_status = sensor_status::CALENABLED_BM as u8 | 
            sensor_status::BUSY_BM as u8 |
            0x10;

        let not_busy_status = sensor_status::CALENABLED_BM as u8 | 0x10;

        let fake_sensor_data = vec![
            busy_status,
            0x7E, 0x51, //Humid values
            0x65,   //split byte 
            0xD4, 0xA0, //Temp values
            0xDA,   //CRC8-MAXIM, calulated by sensor 
        ];


        let ready_fake_sensor_data = vec![
            not_busy_status,
            0x7E, 0x51, //Humid values
            0x65,   //split byte 
            0xD4, 0xA0, //Temp values
            0xDA,   //CRC8-MAXIM, calulated by sensor 
        ];
        

        let expected = [
            I2cTransaction::write(SENSOR_ADDR, vec![commands::TRIG_MESSURE, TRIG_MEASURE_PARAM0, TRIG_MEASURE_PARAM1]),
            I2cTransaction::read(SENSOR_ADDR, fake_sensor_data),
            I2cTransaction::read(SENSOR_ADDR, ready_fake_sensor_data),
        ];

        //Skip doing the INIT of the sensor.
        let i2c = I2cMock::new(&expected);
        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);
        let mut inited_sensor = InitializedSensor {
            sensor: &mut sensor_instance
        }; 
        
        let mut mock_delay = embedded_hal_mock::delay::MockNoop;
        let data = inited_sensor.read_sensor(&mut mock_delay);

        assert!(data.is_ok());

        let mut sd = data.unwrap();
       
        assert_eq!(sd.bytes[0], 0x18);
        assert_eq!(sd.bytes[6], 0xDA);
        assert!(sd.is_crc_good());
        assert_eq!(sd.crc, 0xDA);       
        assert_eq!(sd.bytes[6], sd.crc);
 

        inited_sensor.sensor.i2c.done();
    }

    #[test]
    fn soft_reset()
    {
        
        let not_busy_status = vec![0x00];

        let expected = [
            I2cTransaction::write(SENSOR_ADDR, vec![commands::READ_STATUS]),
            I2cTransaction::read(SENSOR_ADDR, not_busy_status.clone()),
            I2cTransaction::write(SENSOR_ADDR, vec![commands::SOFT_RESET]),
            I2cTransaction::write(SENSOR_ADDR, vec![commands::READ_STATUS]),
            I2cTransaction::read(SENSOR_ADDR, not_busy_status.clone()),
        ];


        //Skip doing the INIT of the sensor.
        let i2c = I2cMock::new(&expected);
        let mut sensor_instance = Sensor::new(i2c, SENSOR_ADDR);
        let mut inited_sensor = InitializedSensor {
            sensor: &mut sensor_instance
        }; 
        
        let mut mock_delay = embedded_hal_mock::delay::MockNoop;
        
        let sr = inited_sensor.soft_reset(&mut mock_delay);
        assert!(sr.is_ok());

        sensor_instance.i2c.done();
    }
}
