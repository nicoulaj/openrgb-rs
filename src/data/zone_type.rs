use std::mem::size_of;

use async_trait::async_trait;
use num_traits::FromPrimitive;

use crate::data::{OpenRGBReadable, OpenRGBWritable};
use crate::OpenRGBError;
use crate::OpenRGBError::ProtocolError;
use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};

/// RGB controller [Zone](crate::data::Zone) type.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#zone-data) for more information.
#[derive(Primitive, Eq, PartialEq, Debug, Copy, Clone)]
pub enum ZoneType {

    /// Single zone.
    Single = 0,

    /// Linear zone.
    Linear = 1,

    /// Matrix zone.
    Matrix = 2,
}

#[async_trait]
impl OpenRGBWritable for ZoneType {
    fn size(&self, _protocol: u32) -> usize {
        size_of::<u32>()
    }

    async fn write(self, stream: &mut impl OpenRGBWritableStream, protocol: u32) -> Result<(), OpenRGBError> {
        stream.write_value(self as u32, protocol).await
    }
}

#[async_trait]
impl OpenRGBReadable for ZoneType {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError> {
        stream.read_value(protocol)
            .await
            .and_then(|id| ZoneType::from_u32(id).ok_or_else(|| ProtocolError(format!("unknown zone type \"{}\"", id))))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::data::ZoneType;
    use crate::DEFAULT_PROTOCOL;
    use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};
    use crate::tests::setup;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&1_u32.to_le_bytes())
            .build();

        assert_eq!(stream.read_value::<ZoneType>(DEFAULT_PROTOCOL).await?, ZoneType::Linear);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&1_u32.to_le_bytes())
            .build();

        stream.write_value(ZoneType::Linear, DEFAULT_PROTOCOL).await?;

        Ok(())
    }
}
