#![no_std]
#![warn(missing_docs)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! ## Feature flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]

use embedded_hal::spi::{Operation, SpiDevice};

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// All possible errors
pub enum Error<SPI> {
    /// SPI bus error
    Spi(SPI),
    /// Parity mismatch
    Parity,
    /// No magnet detected
    Magnet,
}

/// MT6816 Driver
///
/// Requires MODE_3 (CPOL=1, CPHA=1) SPI to exchange data.
pub struct Mt6816<SPI>
where
    SPI: SpiDevice,
{
    spi: SPI,
}

impl<SPI> Mt6816<SPI>
where
    SPI: SpiDevice,
{
    /// Create a new MT6816 driver.
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Read the angle value from the sensor.
    /// Returns the angle in the range 0..=16383, or an error if parity or magnet issues are detected.
    pub fn read_angle(&mut self) -> Result<u16, Error<SPI::Error>> {
        let mut buf = [0u8; 2];

        self.spi
            .transaction(&mut [Operation::Write(&[0x83]), Operation::Read(&mut buf)])
            .map_err(Error::Spi)?;

        let raw = u16::from_be_bytes(buf);

        // Parity: bit 0, Magnet: bit 1, Angle: bits 2..=15
        let parity_bit = (raw & 0x0001) != 0;
        let magnet_bit = (raw & 0x0002) != 0;
        let angle = raw >> 2;

        // Calculate even parity over bits 1..15 (angle bits + magnet bit)
        let calculated_parity = ((raw & 0xFFFE).count_ones() % 2) != 0;

        if calculated_parity != parity_bit {
            return Err(Error::Parity);
        }

        if magnet_bit {
            return Err(Error::Magnet);
        }

        Ok(angle)
    }
}
