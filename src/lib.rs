#![no_std]
#![recursion_limit = "1024"]

extern crate embedded_hal;
extern crate bit_field;

pub mod status;
pub mod register;

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use crate::status::{Status, WriteProtection};
use crate::register::{Instruction, Erase};
use bit_field::BitField;

pub struct Eeprom25x<SPI, CS, WP, HOLD> {
    spi: SPI,
    cs: CS,
    wp: WP,
    hold: HOLD
}

#[derive(Debug)]
pub enum Error<SpiError, PinError> {
    SpiError(SpiError),
    PinError(PinError),
    BusyWriting,
    WrongId
}

impl<SPI, CS, WP, HOLD, SpiError, PinError> Eeprom25x<SPI, CS, WP, HOLD>
where
    SPI: Transfer<u8, Error=SpiError> + Write<u8, Error=SpiError>,
    CS: OutputPin<Error = PinError>,
    WP: OutputPin<Error = PinError>,
    HOLD: OutputPin<Error = PinError>
{
    /// Initializes the EEPROM device
    ///
    /// Checks if the manufacturer ID is correct otherwise returns an error.
    /// Makes sure that you can't write to the status register and also that the device is in deep
    /// sleep mode. Also the chip hold is removed
    pub fn new(spi: SPI, cs: CS, wp: WP, hold: HOLD) -> Result<Self, Error<SpiError, PinError>>{
        let mut ret = Eeprom25x {
            spi, cs, wp, hold
        };
        ret.cs.set_high().map_err(Error::PinError)?;
        #[cfg(feature = "hold_line")]
        ret.hold.set_low().map_err(Error::PinError)?;
        ret.wp.set_high().map_err(Error::PinError)?;

        let id = ret.release_from_deep_sleep_and_get_manufacturer_id()?;
        if id != 0x29 {
            Err(Error::WrongId)
        } else {
            ret.write_enable()?;
            ret.enable_write_to_status()?;
            ret.write_enable()?;
            ret.disable_write_to_status()?;
            ret.deep_sleep()?;
            ret.hold_transfer(false)?;
            Ok(ret)
        }
    }

    /// Returns the status of the chip
    pub fn status(&mut self) -> Result<Status, Error<SpiError, PinError>> {
        let mut buf = [Instruction::ReadStatus as u8, 0];
        self.transfer(&mut buf)?;
        Ok(Status { value: buf[1] })
    }

    /// Set the array write protection level
    /// It can be the first 1/4, 1/2 or whole chip length
    /// Will return the chip in writing to status register disabled
    pub fn set_array_write_protection(&mut self, level: WriteProtection) -> Result<(), Error<SpiError, PinError>> {
        let mut status = self.status()?;
        status.set_write_protection_level(level);
        self.write_enable()?;
        self.enable_write_to_status()?;
        self.write_enable()?;
        let mut buf = [Instruction::WriteStatus as u8, status.value];
        self.transfer(&mut buf)?;
        self.disable_write_to_status()
    }

    /// Returns the status of the chip or an error if it is busy writing
    pub fn error_on_writing(&mut self) -> Result<Status, Error<SpiError, PinError>> {
        let status = self.status()?;
        if status.write_in_progress() {
            Err(Error::BusyWriting)
        } else {
            Ok(status)
        }
    }

    /// Erase parts of the chip. Can be a page, a sector or the whole chip
    pub fn erase(&mut self, mut address: u32, erase: Erase) -> Result<(), Error<SpiError, PinError>> {
        self.error_on_writing()?;
        self.write_enable()?;
        address.set_bits(24..31, erase as u32);
        let mut buf: [u8; 4] = address.to_be_bytes();
        self.transfer(&mut buf)
    }

    /// Keep the device from clocking out data, or enable it to do so
    pub fn hold_transfer(&mut self, enabled: bool) -> Result<(), Error<SpiError, PinError>> {
        if enabled {
            self.hold.set_high().map_err(Error::PinError)
        } else {
            self.hold.set_low().map_err(Error::PinError)
        }
    }

    /// Wake up the chip and also return the manufacturer ID
    pub fn release_from_deep_sleep_and_get_manufacturer_id(&mut self) -> Result<u8, Error<SpiError, PinError>> {
        // <Instruction byte><Dummy address 3 bytes><Manufacturer ID byte>
        let mut buf = [Instruction::ReleasePowerDown as u8, 0, 0, 0, 0];
        self.transfer(&mut buf)?;
        Ok(buf[3])
    }

    /// Put the device in deep sleep mode
    pub fn deep_sleep(&mut self) -> Result<(), Error<SpiError, PinError>>{
        let mut buf = [Instruction::DeepSleepPowerMode as u8];
        self.transfer(&mut buf)
    }

    /// Disable writing to the status register
    pub fn disable_write_to_status(&mut self) -> Result<(), Error<SpiError, PinError>> {
        self.wp.set_high().map_err(Error::PinError)?;
        let mut status = self.status()?;
        status.set_write_protection_enabled(true);
        let mut buf = [Instruction::WriteStatus as u8, status.value];
        self.transfer(&mut buf)?;
        self.wp.set_low().map_err(Error::PinError)
    }

    /// Enable writing to the status register
    pub fn enable_write_to_status(&mut self) -> Result<(), Error<SpiError, PinError>> {
        self.wp.set_high().map_err(Error::PinError)?;
        let mut status = self.status()?;
        status.set_write_protection_enabled(false);
        let mut buf = [Instruction::WriteStatus as u8, status.value];
        self.transfer(&mut buf)
    }

    /// Put the write protection down
    pub fn write_enable(&mut self) -> Result<(), Error<SpiError, PinError>> {
        let mut buf = [Instruction::WriteEnable as u8];
        self.transfer(&mut buf)
    }

    /// Enable write protection
    pub fn write_disable(&mut self) -> Result<(), Error<SpiError, PinError>> {
        let mut buf = [Instruction::WriteDisable as u8];
        self.transfer(&mut buf)
    }

    /// Transfer over the SPI
    pub fn transfer(&mut self, bytes: &mut [u8]) -> Result<(), Error<SpiError, PinError>> {
        self.cs.set_low().map_err(Error::PinError)?;
        self.spi.transfer(bytes).map_err(Error::SpiError)?;
        self.cs.set_high().map_err(Error::PinError)?;
        Ok(())
    }
}

/// Get a u32 command integer from a 24 bit address
pub fn e25x_read_from_address_command(address: u32) -> u32 {
    let mut ret = address;
    ret.set_bits(24..31, Instruction::Read as u32);
    ret
}

/// Get a u32 command integer from a 24 bit address
pub fn e25x_write_from_address_command(address: u32) -> u32 {
    let mut ret = address;
    ret.set_bits(24..31, Instruction::Write as u32);
    ret
}
