extern crate embedded_hal;
extern crate bit_field;

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
    WrongId,
    TooMuchData
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
        ret.hold.set_high().map_err(Error::PinError)?;
        ret.wp.set_high().map_err(Error::PinError)?;

        let id = ret.release_from_deep_sleep_and_get_manufacturer_id()?;
        if id != 0x29 {
            Err(Error::WrongId)
        } else {
            // ret.write_enable()?;
            // ret.enable_write_to_status()?;
            // ret.write_enable()?;
            // ret.disable_write_to_status()?;
            #[cfg(any(
                feature = "25lc512",
                feature = "25lc1024"
            ))]
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
        
        #[cfg(feature = "25lc1024")]
        {
            // <Instruction byte><Dummy address 3 bytes><Manufacturer ID byte>
            let mut buf = [Instruction::ReleasePowerDown as u8, 0, 0, 0, 0];
            self.transfer(&mut buf)?;
            Ok(buf[4])
        }
        #[cfg(not(feature = "25lc1024"))]
        {
            // <Instruction byte><Dummy address 2 bytes><Manufacturer ID byte>
            let mut buf = [Instruction::ReleasePowerDown as u8, 0, 0, 0];
            self.transfer(&mut buf)?;
            Ok(buf[3])
        }
    }

    #[cfg(any(
        feature = "25lc512",
        feature = "25lc1024"
    ))]
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

    pub fn status_read(&mut self) -> Result<u8, Error<SpiError, PinError>>{
        let mut buf = [Instruction::ReadStatus as u8, 0];
        self.cs.set_low().map_err(Error::PinError)?;
        self.spi.transfer(&mut buf).map_err(Error::SpiError)?;
        self.cs.set_high().map_err(Error::PinError)?;
        Ok(buf[1])
    }
    /// Transfer over the SPI
    fn transfer(&mut self, bytes: &mut [u8]) -> Result<(), Error<SpiError, PinError>> {
        self.cs.set_low().map_err(Error::PinError)?;
        self.spi.transfer(bytes).map_err(Error::SpiError)?;
        self.cs.set_high().map_err(Error::PinError)?;
        Ok(())
    }

    pub fn read(&mut self, address: u32, bytes: &mut [u8]) -> Result<(), Error<SpiError, PinError>>
    {
        let read_reg = e25x_read_from_address_command(address);
        let read_reg: [u8; 4] = read_reg.to_be_bytes();
        self.cs.set_low().map_err(Error::PinError)?;
        self.spi.write(&read_reg).map_err(Error::SpiError)?;
        self.spi.transfer(bytes).map_err(Error::SpiError)?;
        self.cs.set_high().map_err(Error::PinError)?;
        Ok(())
    }

    pub fn write(&mut self, address: u32, bytes: &[u8]) -> Result<(), Error<SpiError, PinError>>
    {
        let read_reg = e25x_write_from_address_command(address);
        let read_reg: [u8; 4] = read_reg.to_be_bytes();
        self.cs.set_low().map_err(Error::PinError)?;
        self.spi.write(&read_reg).map_err(Error::SpiError)?;
        self.spi.write(bytes).map_err(Error::SpiError)?;
        self.cs.set_high().map_err(Error::PinError)?;
        Ok(())
    }
}

/// Get a u32 command integer from a 24 bit address
fn e25x_read_from_address_command(address: u32) -> u32 {
    let mut ret = address;
    ret.set_bits(24..31, Instruction::Read as u32);
    ret
}

/// Get a u32 command integer from a 24 bit address
fn e25x_write_from_address_command(address: u32) -> u32 {
    let mut ret = address;
    ret.set_bits(24..31, Instruction::Write as u32);
    ret
}
