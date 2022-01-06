use std::mem::size_of;

use async_trait::async_trait;
use num_traits::FromPrimitive;

use crate::data::{OpenRGBReadable, OpenRGBWritable};
use crate::OpenRGBError::{self, ProtocolError};
use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};

/// OpenRGB protocol packet ID.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#packet-ids) for more information.
#[derive(Primitive, PartialEq, Debug, Copy, Clone)]
pub enum PacketId {
    /// Request RGBController device count from server.
    RequestControllerCount = 0,

    /// Request RGBController data block.
    RequestControllerData = 1,

    /// Request OpenRGB SDK protocol version from server.
    RequestProtocolVersion = 40,

    /// Send client name string to server.
    SetClientName = 50,

    /// Indicate to clients that device list has updated.
    DeviceListUpdated = 100,

    /// Request profile list.
    RequestProfileList = 150,

    /// Save current configuration in a new profile.
    RequestSaveProfile = 151,

    /// Load a given profile.
    RequestLoadProfile = 152,

    /// Delete a given profile.
    RequestDeleteProfile = 153,

    /// RGBController::ResizeZone().
    RGBControllerResizeZone = 1000,

    /// RGBController::UpdateLEDs().
    RGBControllerUpdateLeds = 1050,

    /// RGBController::UpdateZoneLEDs().
    RGBControllerUpdateZoneLeds = 1051,

    /// RGBController::UpdateSingleLED().
    RGBControllerUpdateSingleLed = 1052,

    /// RGBController::SetCustomMode().
    RGBControllerSetCustomMode = 1100,

    /// RGBController::UpdateMode().
    RGBControllerUpdateMode = 1101,

    /// RGBController::SaveMode().
    RGBControllerSaveMode = 1102,
}

#[async_trait]
impl OpenRGBWritable for PacketId {
    fn size(&self, _protocol: u32) -> usize {
        size_of::<u32>()
    }

    async fn write(self, stream: &mut impl OpenRGBWritableStream, protocol: u32) -> Result<(), OpenRGBError> {
        stream.write_value(self as u32, protocol).await
    }
}

#[async_trait]
impl OpenRGBReadable for PacketId {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError> {
        stream.read_value::<u32>(protocol)
            .await
            .and_then(|id| PacketId::from_u32(id).ok_or_else(|| ProtocolError(format!("unknown packed ID \"{}\"", id))))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use num_traits::{FromPrimitive, ToPrimitive};
    use tokio_test::io::Builder;

    use crate::data::packet::PacketId;
    use crate::DEFAULT_PROTOCOL;
    use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};
    use crate::tests::setup;

    #[test]
    fn test_convert_to_u32() {
        assert_eq!(PacketId::DeviceListUpdated.to_u32(), Some(100));
    }

    #[test]
    fn test_convert_from_u32() {
        assert_eq!(PacketId::from_u32(100), Some(PacketId::DeviceListUpdated))
    }


    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&1101_u32.to_le_bytes())
            .build();

        assert_eq!(stream.read_value::<PacketId>(DEFAULT_PROTOCOL).await?, PacketId::RGBControllerUpdateMode);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&1101_u32.to_le_bytes())
            .build();

        stream.write_value(PacketId::RGBControllerUpdateMode, DEFAULT_PROTOCOL).await?;

        Ok(())
    }
}
