use async_trait::async_trait;

use crate::data::{OpenRGBReadable, OpenRGBWritable};
use crate::OpenRGBError;
use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};

#[async_trait]
impl<A: OpenRGBWritable, B: OpenRGBWritable> OpenRGBWritable for (A, B) {
    fn size(&self, protocol: u32) -> usize {
        self.0.size(protocol) + self.1.size(protocol)
    }

    async fn write(self, stream: &mut impl OpenRGBWritableStream, protocol: u32) -> Result<(), OpenRGBError> {
        stream.write_value(self.0, protocol).await?;
        stream.write_value(self.1, protocol).await?;
        Ok(())
    }
}

#[async_trait]
impl<A: OpenRGBReadable, B: OpenRGBReadable> OpenRGBReadable for (A, B) {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError> {
        Ok((
            stream.read_value::<A>(protocol).await?,
            stream.read_value::<B>(protocol).await?,
        ))
    }
}

#[async_trait]
impl<A: OpenRGBWritable, B: OpenRGBWritable, C: OpenRGBWritable> OpenRGBWritable for (A, B, C) {
    fn size(&self, protocol: u32) -> usize {
        self.0.size(protocol) + self.1.size(protocol) + self.2.size(protocol)
    }

    async fn write(self, stream: &mut impl OpenRGBWritableStream, protocol: u32) -> Result<(), OpenRGBError> {
        stream.write_value(self.0, protocol).await?;
        stream.write_value(self.1, protocol).await?;
        stream.write_value(self.2, protocol).await?;
        Ok(())
    }
}

#[async_trait]
impl<A: OpenRGBReadable, B: OpenRGBReadable, C: OpenRGBReadable> OpenRGBReadable for (A, B, C) {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError> {
        Ok((
            stream.read_value::<A>(protocol).await?,
            stream.read_value::<B>(protocol).await?,
            stream.read_value::<C>(protocol).await?,
        ))
    }
}

#[async_trait]
impl<A: OpenRGBWritable, B: OpenRGBWritable, C: OpenRGBWritable, D: OpenRGBWritable> OpenRGBWritable for (A, B, C, D) {
    fn size(&self, protocol: u32) -> usize {
        self.0.size(protocol) + self.1.size(protocol) + self.2.size(protocol) + self.3.size(protocol)
    }

    async fn write(self, stream: &mut impl OpenRGBWritableStream, protocol: u32) -> Result<(), OpenRGBError> {
        stream.write_value(self.0, protocol).await?;
        stream.write_value(self.1, protocol).await?;
        stream.write_value(self.2, protocol).await?;
        stream.write_value(self.3, protocol).await?;
        Ok(())
    }
}

#[async_trait]
impl<A: OpenRGBReadable, B: OpenRGBReadable, C: OpenRGBReadable, D: OpenRGBReadable> OpenRGBReadable for (A, B, C, D) {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError> {
        Ok((
            stream.read_value::<A>(protocol).await?,
            stream.read_value::<B>(protocol).await?,
            stream.read_value::<C>(protocol).await?,
            stream.read_value::<D>(protocol).await?,
        ))
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
            .read(&37_u8.to_le_bytes())
            .read(&1337_u32.to_le_bytes())
            .read(&(-1337_i32).to_le_bytes())
            .read(&4_u32.to_le_bytes())
            .build();

        assert_eq!(stream.read_value::<(u8, u32, i32, DeviceType)>(DEFAULT_PROTOCOL).await?, (37, 1337, -1337, DeviceType::LEDStrip));

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&37_u8.to_le_bytes())
            .write(&1337_u32.to_le_bytes())
            .write(&(-1337_i32).to_le_bytes())
            .write(&4_u32.to_le_bytes())
            .build();

        stream.write_value((37_u8, 1337_u32, (-1337_i32), DeviceType::LEDStrip), DEFAULT_PROTOCOL).await?;

        Ok(())
    }
}
