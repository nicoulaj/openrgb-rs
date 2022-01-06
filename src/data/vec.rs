use std::mem::size_of;

use async_trait::async_trait;

use crate::data::{OpenRGBReadable, OpenRGBWritable};
use crate::OpenRGBError;
use crate::OpenRGBError::ProtocolError;
use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};

#[async_trait]
impl<T: OpenRGBWritable> OpenRGBWritable for Vec<T> {
    fn size(&self, protocol: u32) -> usize {
        size_of::<u16>() + self.iter().map(|e| e.size(protocol)).sum::<usize>()
    }

    async fn write(self, stream: &mut impl OpenRGBWritableStream, protocol: u32) -> Result<(), OpenRGBError> {
        stream.write_value(u16::try_from(self.len()).map_err(|e| ProtocolError(format!("Vec is too large to encode: {}", e)))?, protocol).await?;
        for elem in self {
            stream.write_value(elem, protocol).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<T: OpenRGBReadable> OpenRGBReadable for Vec<T> {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError> {
        let len = stream.read_value::<u16>(protocol).await? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(stream.read_value(protocol).await?);
        }
        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::{DEFAULT_PROTOCOL};
    use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};
    use crate::tests::setup;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&3_u16.to_le_bytes())
            .read(&[37_u8, 54_u8, 126_u8])
            .build();

        assert_eq!(stream.read_value::<Vec<u8>>(DEFAULT_PROTOCOL).await?, vec![37_u8, 54_u8, 126_u8]);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&3_u16.to_le_bytes())
            .write(&[37_u8, 54_u8, 126_u8])
            .build();

        stream.write_value(vec![37_u8, 54_u8, 126_u8], DEFAULT_PROTOCOL).await?;

        Ok(())
    }
}
