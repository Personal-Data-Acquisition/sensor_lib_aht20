
//We have sepreate consts and enums for the puporse of being used during
//testing(consts) or as parameters(enum).
pub const READ_STATUS: u8 = 0x71;
pub const INIT_SENSOR: u8 = 0xBE;
pub const CALIBRATE: u8 = 0xE1;
pub const TRIG_MESSURE: u8 = 0xAC;
pub const SOFT_RESET: u8 = 0xBA;

#[repr(u8)]
#[allow(dead_code)]
pub enum Command {
    ReadStatus = READ_STATUS,
    InitSensor = INIT_SENSOR,
    Calibrate = CALIBRATE,
    TrigMessure = TRIG_MESSURE,
    SoftReset = SOFT_RESET,
}

