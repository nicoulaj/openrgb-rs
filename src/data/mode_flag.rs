use std::mem::size_of;

use async_trait::async_trait;
use flagset::{flags, FlagSet};

use crate::data::{OpenRGBReadable, OpenRGBWritable};
use crate::OpenRGBError;
use crate::OpenRGBError::ProtocolError;
use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};

flags! {
    /// RGB controller mode flags.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
    pub enum ModeFlag: u32 {
        /// Mode has speed parameter.
        HasSpeed = 1 << 0,

        /// Mode has left/right parameter.
        HasDirectionLR = 1 << 1,

        /// Mode has up/down parameter.
        HasDirectionUD = 1 << 2,

        /// Mode has horiz/vert parameter.
        HasDirectionHV = 1 << 3,

        /// Mode has direction parameter.
        HasDirection = (ModeFlag::HasDirectionLR | ModeFlag::HasDirectionUD | ModeFlag::HasDirectionHV).bits(),

        /// Mode has brightness parameter.
        HasBrightness = 1 << 4,

        /// Mode has per-LED colors.
        HasPerLEDColor = 1 << 5,

        /// Mode has mode specific colors.
        HasModeSpecificColor = 1 << 6,

        /// Mode has random color option.
        HasRandomColor = 1 << 7,

        /// Mode can manually be saved.
        ManualSave = 1 << 8,

        /// Mode automatically saves.
        AutomaticSave = 1 << 9,
    }
}

#[async_trait]
impl OpenRGBWritable for FlagSet<ModeFlag> {
    fn size(&self, _protocol: u32) -> usize {
        size_of::<u32>()
    }

    async fn write(self, stream: &mut impl OpenRGBWritableStream, protocol: u32) -> Result<(), OpenRGBError> {
        stream.write_value(self.bits(), protocol).await
    }
}

#[async_trait]
impl OpenRGBReadable for FlagSet<ModeFlag> {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError> {
        let value = stream.read_value(protocol).await?;
        FlagSet::<ModeFlag>::new(value).map_err(|e| ProtocolError(format!("Received invalid mode flag set: {} ({})", value, e)))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use flagset::FlagSet;
    use tokio_test::io::Builder;

    use ModeFlag::*;
    use crate::data::ModeFlag;

    use crate::DEFAULT_PROTOCOL;
    use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};
    use crate::tests::setup;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&154_u32.to_le_bytes())
            .build();

        assert_eq!(stream.read_value::<FlagSet<ModeFlag>>(DEFAULT_PROTOCOL).await?, HasDirectionLR | HasDirectionHV | HasBrightness | HasRandomColor);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&31_u32.to_le_bytes())
            .build();

        stream.write_value(HasDirection | HasSpeed | HasBrightness, DEFAULT_PROTOCOL).await?;

        Ok(())
    }
}
