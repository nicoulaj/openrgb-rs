use std::mem::size_of;

use async_trait::async_trait;
use num_traits::FromPrimitive;

use crate::data::{OpenRGBReadable, OpenRGBWritable};
use crate::OpenRGBError;
use crate::OpenRGBError::ProtocolError;
use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};

/// Direction for [Mode](crate::data::Mode).
#[derive(Primitive, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Direction {

    /// Left direction.
    Left = 0,

    /// Right direction.
    Right = 1,

    /// Up direction.
    Up = 2,

    /// Down direction.
    Down = 3,

    /// Horizontal direction.
    Horizontal = 4,

    /// Vertical direction.
    Vertical = 5,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Left
    }
}

#[async_trait]
impl OpenRGBWritable for Direction {
    fn size(&self, _protocol: u32) -> usize {
        size_of::<u32>()
    }

    async fn write(self, stream: &mut impl OpenRGBWritableStream, protocol: u32) -> Result<(), OpenRGBError> {
        stream.write_value(self as u32, protocol).await
    }
}

#[async_trait]
impl OpenRGBReadable for Direction {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError> {
        stream.read_value(protocol)
            .await
            .and_then(|id| Direction::from_u32(id).ok_or_else(|| ProtocolError(format!("unknown direction \"{}\"", id))))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;
    use crate::data::Direction;

    use crate::DEFAULT_PROTOCOL;
    use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};
    use crate::tests::setup;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&4_u32.to_le_bytes())
            .build();

        assert_eq!(stream.read_value::<Direction>(DEFAULT_PROTOCOL).await?, Direction::Horizontal);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&4_u32.to_le_bytes())
            .build();

        stream.write_value(Direction::Horizontal, DEFAULT_PROTOCOL).await?;

        Ok(())
    }
}
