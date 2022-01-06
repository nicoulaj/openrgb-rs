use std::mem::size_of;

use async_trait::async_trait;

use crate::data::{OpenRGBReadable, OpenRGBWritable};
use crate::OpenRGBError;
use crate::OpenRGBError::ProtocolError;
use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};

// FIXME buggy for non ASCII strings

#[async_trait]
impl OpenRGBWritable for String {
    fn size(&self, _protocol: u32) -> usize {
        self.len() + 1 + size_of::<u16>()
    }

    async fn write(self, stream: &mut impl OpenRGBWritableStream, protocol: u32) -> Result<(), OpenRGBError> {
        stream.write_value((self.len() + 1) as u16, protocol).await?;
        stream.write_value(RawString(self), protocol).await
    }
}

#[async_trait]
impl OpenRGBReadable for String {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError> {
        let mut buf = vec![Default::default(); stream.read_value::<u16>(protocol).await? as usize];
        stream.read_exact(&mut buf).await?;
        buf.pop();
        String::from_utf8(buf).map_err(|e| ProtocolError(format!("Failed decoding string as UTF-8: {}", e)))
    }
}

#[doc(hidden)]
pub struct RawString(pub String);

#[async_trait]
impl OpenRGBWritable for RawString {
    fn size(&self, _protocol: u32) -> usize {
        self.0.len() + 1
    }

    async fn write(self, stream: &mut impl OpenRGBWritableStream, _protocol: u32) -> Result<(), OpenRGBError> {
        stream.write_all(format!("{}\0", self.0).as_bytes()).await.map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::data::RawString;
    use crate::DEFAULT_PROTOCOL;
    use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};
    use crate::tests::setup;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&5_u16.to_le_bytes())
            .read(b"test\0")
            .build();

        assert_eq!(stream.read_value::<String>(DEFAULT_PROTOCOL).await?, "test".to_string());

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&5_u16.to_le_bytes())
            .write(b"test\0")
            .build();

        stream.write_value("test".to_string(), DEFAULT_PROTOCOL).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_write_raw_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(b"test\0")
            .build();

        stream.write_value(RawString("test".to_string()), DEFAULT_PROTOCOL).await?;

        Ok(())
    }
}
