use bit_field::BitField;
use core::cmp::min;
use crate::{eeprom25x::Eeprom25x,eeprom25x::Error};
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use embedded_storage::ReadStorage;


pub struct Storage<SPI, CS, WP, HOLD> 
{
    /// Eeprom driver over which we implement the Storage traits
    pub eeprom: Eeprom25x<SPI, CS, WP, HOLD>
}

impl<SPI, CS, WP, HOLD, SpiError, PinError> Storage<SPI, CS, WP, HOLD>
where
    SPI: Transfer<u8, Error=SpiError> + Write<u8, Error=SpiError>,
    CS: OutputPin<Error = PinError>,
    WP: OutputPin<Error = PinError>,
    HOLD: OutputPin<Error = PinError>
{
    #[cfg(feature = "density_8k")]
    const CAPACITY: usize = 1024 * 8;
    #[cfg(feature = "density_16k")]
    const CAPACITY: usize = 1024 * 16;
    #[cfg(feature = "density_32k")]
    const CAPACITY: usize = 1024 * 32;
    #[cfg(feature = "density_64k")]
    const CAPACITY: usize = 1024 * 64;
    #[cfg(feature = "density_128k")]
    const CAPACITY: usize = 1024 * 128;
    #[cfg(feature = "density_256k")]
    const CAPACITY: usize = 1024 * 256;
    #[cfg(feature = "density_512k")]
    const CAPACITY: usize = 1024 * 512;
    #[cfg(feature = "density_1024k")]
    const CAPACITY: usize = 1024 * 1024;

    #[cfg(feature = "page_size_16")]
    const PAGE_SIZE: usize = 16;
    #[cfg(feature = "page_size_32")]
    const PAGE_SIZE: usize = 32;
    #[cfg(feature = "page_size_64")]
    const PAGE_SIZE: usize = 64;
    #[cfg(feature = "page_size_128")]
    const PAGE_SIZE: usize = 128;
    #[cfg(feature = "page_size_256")]
    const PAGE_SIZE: usize = 256;

    /// Create a new Storage instance wrapping the given Eeprom
    pub fn new(eeprom: Eeprom25x<SPI, CS, WP, HOLD>) -> Self {
        Storage { eeprom }
    }

}

impl<SPI, CS, WP, HOLD, SpiError, PinError> embedded_storage::ReadStorage for Storage<SPI, CS, WP, HOLD>
where
    SPI: Transfer<u8, Error=SpiError> + Write<u8, Error=SpiError>,
    CS: OutputPin<Error = PinError>,
    WP: OutputPin<Error = PinError>,
    HOLD: OutputPin<Error = PinError>
{
    type Error = Error<SpiError, PinError>;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.eeprom.hold_transfer(true)?;
        #[cfg(any(
            feature = "25lc512",
            feature = "25lc1024"
        ))]
        let _ = self.eeprom.release_from_deep_sleep_and_get_manufacturer_id()?;
        self.eeprom.read(offset, bytes)?;
        #[cfg(any(
            feature = "25lc512",
            feature = "25lc1024"
        ))]
        self.eeprom.deep_sleep()?;
        self.eeprom.hold_transfer(false)?;
        Ok(())
    }

    fn capacity(&self) -> usize {
        Self::CAPACITY
    }
}

impl<SPI, CS, WP, HOLD, SpiError, PinError> embedded_storage::Storage for Storage<SPI, CS, WP, HOLD>
where
    SPI: Transfer<u8, Error=SpiError> + Write<u8, Error=SpiError>,
    CS: OutputPin<Error = PinError>,
    WP: OutputPin<Error = PinError>,
    HOLD: OutputPin<Error = PinError>
{
    fn write(&mut self, mut offset: u32, mut bytes: &[u8]) -> Result<(), Self::Error> {
        if offset as usize + bytes.len() > self.capacity() {
            return Err(Error::TooMuchData);
        }
        self.eeprom.hold_transfer(true)?;
        let page_size = Self::PAGE_SIZE;
        #[cfg(any(
            feature = "25lc512",
            feature = "25lc1024"
        ))]
        let _ = self.eeprom.release_from_deep_sleep_and_get_manufacturer_id()?;
        while !bytes.is_empty() {
            self.eeprom.write_enable()?;
            let this_page_offset = offset as usize % page_size;
            let this_page_remaining = page_size - this_page_offset;
            let chunk_size = min(bytes.len(), this_page_remaining);
            self.eeprom.write(offset, &bytes[..chunk_size])?;
            offset += chunk_size as u32;
            bytes = &bytes[chunk_size..];
            while self.eeprom.status_read()?.get_bit(0) {}
            self.eeprom.write_disable()?;
        }
        #[cfg(any(
            feature = "25lc512",
            feature = "25lc1024"
        ))]
        self.eeprom.deep_sleep()?;
        self.eeprom.hold_transfer(false)?;
        Ok(())
    }
}
