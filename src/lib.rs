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
    pub fn new(spi: SPI, cs: CS, wp: WP, hold: HOLD) -> Result<Self, Error<SpiError, PinError>>{
        let mut ret = Eeprom25x {
            spi, cs, wp, hold
        };
        ret.cs.set_high().map_err(Error::PinError)?;
        ret.hold.set_low().map_err(Error::PinError)?;
        ret.wp.set_low().map_err(Error::PinError)?;

        let id = ret.release_from_deep_sleep_and_get_manufacturer_id()?;
        if id != 0x29 {
            Err(Error::WrongId)
        } else {
            ret.write_enable()?;
            ret.enable_write_to_status()?;
            ret.write_enable()?;
            ret.disable_write_to_status()?;
            Ok(ret)
        }
    }

    pub fn status(&mut self) -> Result<Status, Error<SpiError, PinError>> {
        let mut buf = [Instruction::ReadStatus as u8, 0];
        self.transfer(&mut buf)?;
        Ok(Status { value: buf[1] })
    }

    pub fn set_array_write_protection(&mut self, level: WriteProtection, enabled: bool) -> Result<(), Error<SpiError, PinError>> {
        let mut status = self.status()?;
        status.set_write_protection_level(level);
        status.set_write_protection_enabled(enabled);
        self.write_enable()?;
        self.enable_write_to_status()?;
        self.write_enable()?;
        let mut buf = [Instruction::WriteStatus as u8, status.value];
        self.transfer(&mut buf)
    }

    pub fn error_on_writing(&mut self) -> Result<Status, Error<SpiError, PinError>> {
        let status = self.status()?;
        if status.write_in_progress() {
            Err(Error::BusyWriting)
        } else {
            Ok(status)
        }
    }

    pub fn erase(&mut self, mut address: u32, erase: Erase) -> Result<(), Error<SpiError, PinError>> {
        self.error_on_writing()?;
        self.write_enable()?;
        address.set_bits(24..31, erase as u32);
        let mut buf: [u8; 4] = address.to_be_bytes();
        self.transfer(&mut buf)
    }

    pub fn hold_transfer(&mut self, enabled: bool) -> Result<(), Error<SpiError, PinError>> {
        if enabled {
            self.hold.set_high().map_err(Error::PinError)
        } else {
            self.hold.set_low().map_err(Error::PinError)
        }
    }

    pub fn release_from_deep_sleep_and_get_manufacturer_id(&mut self) -> Result<u8, Error<SpiError, PinError>> {
        // <Instruction byte><Dummy address 3 bytes><Manufacturer ID byte>
        let mut buf = [Instruction::ReleasePowerDown as u8, 0, 0, 0, 0];
        self.transfer(&mut buf)?;
        Ok(buf[3])
    }

    pub fn deep_sleep(&mut self) -> Result<(), Error<SpiError, PinError>>{
        let mut buf = [Instruction::DeepSleepPowerMode as u8];
        self.transfer(&mut buf)
    }

    pub fn unprotected_blocks_writable(&mut self) -> Result<bool, Error<SpiError, PinError>> {
        let status = self.status()?;
        Ok(status.write_latch_enabled())
    }

    pub fn disable_write_to_status(&mut self) -> Result<(), Error<SpiError, PinError>> {
        self.wp.set_high().map_err(Error::PinError)?;
        let mut status = self.status()?;
        status.set_write_protection_enabled(true);
        let mut buf = [Instruction::WriteStatus as u8, status.value];
        self.transfer(&mut buf)?;
        self.wp.set_low().map_err(Error::PinError)
    }

    pub fn enable_write_to_status(&mut self) -> Result<(), Error<SpiError, PinError>> {
        self.wp.set_high().map_err(Error::PinError)?;
        let mut status = self.status()?;
        status.set_write_protection_enabled(false);
        let mut buf = [Instruction::WriteStatus as u8, status.value];
        self.transfer(&mut buf)
    }

    pub fn write_enable(&mut self) -> Result<(), Error<SpiError, PinError>> {
        let mut buf = [Instruction::WriteEnable as u8];
        self.transfer(&mut buf)
    }

    pub fn write_disable(&mut self) -> Result<(), Error<SpiError, PinError>> {
        let mut buf = [Instruction::WriteDisable as u8];
        self.transfer(&mut buf)
    }

    pub fn read_from_address_command(address: u32) -> u32 {
        let mut ret = address;
        ret.set_bits(24..31, Instruction::Read as u32);
        ret
    }

    pub fn write_from_address_command(address: u32) -> u32 {
        let mut ret = address;
        ret.set_bits(24..31, Instruction::Write as u32);
        ret
    }

    pub fn transfer(&mut self, bytes: &mut [u8]) -> Result<(), Error<SpiError, PinError>> {
        self.cs.set_low().map_err(Error::PinError)?;
        self.spi.transfer(bytes).map_err(Error::SpiError)?;
        self.cs.set_high().map_err(Error::PinError)?;
        Ok(())
    }
}
