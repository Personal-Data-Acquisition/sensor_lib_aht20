//! Register Mapping

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]

//TODO: add registers to it.
pub enum Registers {
    /// Control Register 1.
    CTRL_REG1 = 0x20,
}


//TODO: setup methods for it.
impl Registers {
    pub fn addr(self) -> u8 {
        self as u8
    }
}


//TODO: Add bitmasks
#[allow(non_camel_case_types)]
pub (crate) struct Bitmaks;

#[allow(dead_code)]
impl Bitmaks {
    // === CTRL_REG1 (0x20) ===
    pub (crate) const BDU: u8 = 0b000_0100;
}
