use std::mem::size_of;

use async_trait::async_trait;
use num_traits::FromPrimitive;

use crate::data::{OpenRGBReadable, OpenRGBWritable};
use crate::OpenRGBError;
use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};

/// RGB controller device type.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
#[derive(Primitive, Eq, PartialEq, Debug, Copy, Clone)]
pub enum DeviceType {
    /// Motherboard.
    Motherboard = 0,

    /// DRAM.
    DRAM = 1,

    /// GPU.
    GPU = 2,

    /// Cooler.
    Cooler = 3,

    /// LED strip.
    LEDStrip = 4,

    /// Keyboard.
    Keyboard = 5,

    /// Mouse.
    Mouse = 6,

    /// Mouse mat.
    MouseMat = 7,

    /// Headset.
    Headset = 8,

    /// Headset stand.
    HeadsetStand = 9,

    /// Gamepad.
    Gamepad = 10,

    /// Light.
    Light = 11,

    /// Speaker.
    Speaker = 12,

    /// Virtual.
    Virtual = 13,

    /// Unknown.
    Unknown = 14,
}

#[async_trait]
impl OpenRGBWritable for DeviceType {
    fn size(&self, _protocol: u32) -> usize {
        size_of::<u32>()
    }

    async fn write(self, stream: &mut impl OpenRGBWritableStream, protocol: u32) -> Result<(), OpenRGBError> {
        stream.write_value(self as u32, protocol).await
    }
}

#[async_trait]
impl OpenRGBReadable for DeviceType {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError> {
        Ok(DeviceType::from_u32(stream.read_value(protocol).await?).unwrap_or(DeviceType::Unknown))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::data::DeviceType;
    use crate::DEFAULT_PROTOCOL;
    use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};
    use crate::tests::setup;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&8_u32.to_le_bytes())
            .build();

        assert_eq!(stream.read_value::<DeviceType>(DEFAULT_PROTOCOL).await?, DeviceType::Headset);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&8_u32.to_le_bytes())
            .build();

        stream.write_value(DeviceType::Headset, DEFAULT_PROTOCOL).await?;

        Ok(())
    }
}
