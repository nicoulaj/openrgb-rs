use async_trait::async_trait;

use crate::data::OpenRGBReadable;
use crate::OpenRGBError;
use crate::protocol::OpenRGBReadableStream;

/// A single LED.
#[derive(Debug, Eq, PartialEq)]
pub struct LED {
    /// LED name.
    pub name: String,

    /// LED value.
    pub value: u32,
}

#[async_trait]
impl OpenRGBReadable for LED {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError> {
        Ok(LED {
            name: stream.read_value(protocol).await?,
            value: stream.read_value(protocol).await?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::data::LED;
    use crate::DEFAULT_PROTOCOL;
    use crate::protocol::OpenRGBReadableStream;
    use crate::tests::setup;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&5_u16.to_le_bytes())
            .read(b"test\0")
            .read(&45_u32.to_le_bytes())
            .build();

        assert_eq!(stream.read_value::<LED>(DEFAULT_PROTOCOL).await?, LED { name: "test".to_string(), value: 45 });

        Ok(())
    }
}
