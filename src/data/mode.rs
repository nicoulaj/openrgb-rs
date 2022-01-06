use async_trait::async_trait;
use flagset::FlagSet;
use num_traits::FromPrimitive;

use crate::{OpenRGBError::{self, ProtocolError}};
use crate::data::{Color, ColorMode, Direction, ModeFlag::{self, *}, OpenRGBReadable, OpenRGBWritable};
use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};

/// RGB controller mode.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#mode-data) for more information.
#[derive(Debug, Eq, PartialEq)]
pub struct Mode {
    /// Mode name.
    pub name: String,

    /// Mode value.
    pub value: i32,

    /// Mode flags set.
    pub flags: FlagSet<ModeFlag>,

    /// Mode minimum speed (if mode has [ModeFlag::HasSpeed] flag).
    pub speed_min: Option<u32>,

    /// Mode maximum speed (if mode has [ModeFlag::HasSpeed] flag).
    pub speed_max: Option<u32>,

    /// Mode maximum speed (if mode has [ModeFlag::HasSpeed] flag).
    pub speed: Option<u32>,

    /// Mode minimum brightness (if mode has [ModeFlag::HasBrightness] flag).
    pub brightness_min: Option<u32>,

    /// Mode maximum brightness (if mode has [ModeFlag::HasBrightness] flag).
    pub brightness_max: Option<u32>,

    /// Mode brightness (if mode has [ModeFlag::HasBrightness] flag).
    pub brightness: Option<u32>,

    /// Mode color mode.
    pub color_mode: Option<ColorMode>,

    /// Mode colors.
    pub colors: Vec<Color>,

    /// Mode minimum colors (if mode has non empty [Mode::colors] list).
    pub colors_min: Option<u32>,

    /// Mode minimum colors (if mode has non empty [Mode::colors] list).
    pub colors_max: Option<u32>,

    /// Mode direction.
    pub direction: Option<Direction>,
}

#[async_trait]
impl OpenRGBReadable for Mode {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError> {
        let name = stream.read_value(protocol).await?;
        let value = stream.read_value(protocol).await?;
        let flags = stream.read_value(protocol).await?;
        let speed_min = stream.read_value(protocol).await?;
        let speed_max = stream.read_value(protocol).await?;
        let brightness_min = if protocol >= 3 { Some(stream.read_value(protocol).await?) } else { None };
        let brightness_max = if protocol >= 3 { Some(stream.read_value(protocol).await?) } else { None };
        let colors_min = stream.read_value(protocol).await?;
        let colors_max = stream.read_value(protocol).await?;
        let speed = stream.read_value(protocol).await?;
        let brightness = if protocol >= 3 { Some(stream.read_value(protocol).await?) } else { None };
        let direction = stream.read_value(protocol).await?;
        let color_mode = stream.read_value(protocol).await?;
        let colors = stream.read_value::<Vec<Color>>(protocol).await?;

        Ok(Mode {
            name,
            value,
            flags,
            speed_min: if flags.contains(HasSpeed) { Some(speed_min) } else { None },
            speed_max: if flags.contains(HasSpeed) { Some(speed_max) } else { None },
            brightness_min: if flags.contains(HasBrightness) { brightness_min } else { None },
            brightness_max: if flags.contains(HasBrightness) { brightness_max } else { None },
            colors_min: if colors.is_empty() { None } else { Some(colors_min) },
            colors_max: if colors.is_empty() { None } else { Some(colors_max) },
            speed: if flags.contains(HasSpeed) { Some(speed) } else { None },
            brightness: if flags.contains(HasBrightness) { brightness } else { None },
            direction: if flags.contains(HasDirection) { Some(Direction::from_u32(direction).ok_or_else(|| ProtocolError(format!("unknown direction \"{}\"", direction)))?) } else { None },
            color_mode: Some(color_mode),
            colors,
        })
    }
}

#[async_trait]
impl OpenRGBWritable for Mode {
    fn size(&self, protocol: u32) -> usize {
        let mut size = 0;
        size += self.name.size(protocol);
        size += self.value.size(protocol);
        size += self.flags.size(protocol);
        size += self.speed_min.unwrap_or_default().size(protocol);
        size += self.speed_max.unwrap_or_default().size(protocol);
        if protocol >= 3 {
            size += self.brightness_min.unwrap_or_default().size(protocol);
            size += self.brightness_max.unwrap_or_default().size(protocol);
        }
        size += self.colors_min.unwrap_or_default().size(protocol);
        size += self.colors_max.unwrap_or_default().size(protocol);
        size += self.speed.unwrap_or_default().size(protocol);
        if protocol >= 3 {
            size += self.brightness.unwrap_or_default().size(protocol);
        }
        size += self.direction.unwrap_or_default().size(protocol);
        size += self.color_mode.unwrap_or_default().size(protocol);
        size += self.colors.size(protocol);
        size
    }

    async fn write(self, stream: &mut impl OpenRGBWritableStream, protocol: u32) -> Result<(), OpenRGBError> {
        stream.write_value(self.name, protocol).await?;
        stream.write_value(self.value, protocol).await?;
        stream.write_value(self.flags, protocol).await?;
        stream.write_value(self.speed_min.unwrap_or_default(), protocol).await?;
        stream.write_value(self.speed_max.unwrap_or_default(), protocol).await?;
        if protocol >= 3 {
            stream.write_value(self.brightness_min.unwrap_or_default(), protocol).await?;
            stream.write_value(self.brightness_max.unwrap_or_default(), protocol).await?;
        }
        stream.write_value(self.colors_min.unwrap_or_default(), protocol).await?;
        stream.write_value(self.colors_max.unwrap_or_default(), protocol).await?;
        stream.write_value(self.speed.unwrap_or_default(), protocol).await?;
        if protocol >= 3 {
            stream.write_value(self.brightness.unwrap_or_default(), protocol).await?;
        }
        stream.write_value(self.direction.unwrap_or_default(), protocol).await?;
        stream.write_value(self.color_mode.unwrap_or_default(), protocol).await?;
        stream.write_value(self.colors, protocol).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::data::{Color, ColorMode, Direction, Mode, ModeFlag::*};
    use crate::DEFAULT_PROTOCOL;
    use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};
    use crate::tests::setup;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&5_u16.to_le_bytes()) // name len
            .read(b"test\0") // name
            .read(&46_i32.to_le_bytes()) // value
            .read(&31_u32.to_le_bytes()) // flags
            .read(&10_u32.to_le_bytes()) // speed_min
            .read(&1000_u32.to_le_bytes()) // speed_max
            .read(&1_u32.to_le_bytes()) // brightness_min
            .read(&1024_u32.to_le_bytes()) // brightness_max
            .read(&0_u32.to_le_bytes()) // colors_min
            .read(&256_u32.to_le_bytes()) // colors_max
            .read(&51_u32.to_le_bytes()) // speed
            .read(&512_u32.to_le_bytes()) // brightness
            .read(&4_u32.to_le_bytes()) // direction
            .read(&1_u32.to_le_bytes()) // color_mode
            .read(&2_u16.to_le_bytes()) // colors len
            .read(&[37_u8, 54_u8, 126_u8, 0_u8])// colors[0]
            .read(&[37_u8, 54_u8, 255_u8, 0_u8])// colors[1]
            .build();

        assert_eq!(stream.read_value::<Mode>(DEFAULT_PROTOCOL).await?, Mode {
            name: "test".to_string(),
            value: 46,
            flags: HasDirection | HasSpeed | HasBrightness,
            speed_min: Some(10),
            speed_max: Some(1000),
            brightness_min: Some(1),
            brightness_max: Some(1024),
            colors_min: Some(0),
            colors_max: Some(256),
            speed: Some(51),
            brightness: Some(512),
            direction: Some(Direction::Horizontal),
            color_mode: Some(ColorMode::PerLED),
            colors: vec![
                Color { r: 37, g: 54, b: 126 },
                Color { r: 37, g: 54, b: 255 },
            ],
        });

        Ok(())
    }

    #[tokio::test]
    async fn test_read_002() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&5_u16.to_le_bytes()) // name len
            .read(b"test\0") // name
            .read(&46_i32.to_le_bytes()) // value
            .read(&0_u32.to_le_bytes()) // flags
            .read(&10_u32.to_le_bytes()) // speed_min
            .read(&1000_u32.to_le_bytes()) // speed_max
            .read(&1_u32.to_le_bytes()) // brightness_min
            .read(&1024_u32.to_le_bytes()) // brightness_max
            .read(&0_u32.to_le_bytes()) // colors_min
            .read(&256_u32.to_le_bytes()) // colors_max
            .read(&51_u32.to_le_bytes()) // speed
            .read(&512_u32.to_le_bytes()) // brightness
            .read(&4_u32.to_le_bytes()) // direction
            .read(&1_u32.to_le_bytes()) // color_mode
            .read(&0_u16.to_le_bytes()) // colors len
            .build();

        assert_eq!(stream.read_value::<Mode>(DEFAULT_PROTOCOL).await?, Mode {
            name: "test".to_string(),
            value: 46,
            flags: Default::default(),
            speed_min: None,
            speed_max: None,
            brightness_min: None,
            brightness_max: None,
            colors_min: None,
            colors_max: None,
            speed: None,
            brightness: None,
            direction: None,
            color_mode: Some(ColorMode::PerLED),
            colors: vec![],
        });

        Ok(())
    }

    #[tokio::test]
    async fn test_read_003() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&5_u16.to_le_bytes()) // name len
            .read(b"test\0") // name
            .read(&46_i32.to_le_bytes()) // value
            .read(&31_u32.to_le_bytes()) // flags
            .read(&10_u32.to_le_bytes()) // speed_min
            .read(&1000_u32.to_le_bytes()) // speed_max
            .read(&0_u32.to_le_bytes()) // colors_min
            .read(&256_u32.to_le_bytes()) // colors_max
            .read(&51_u32.to_le_bytes()) // speed
            .read(&4_u32.to_le_bytes()) // direction
            .read(&1_u32.to_le_bytes()) // color_mode
            .read(&2_u16.to_le_bytes()) // colors len
            .read(&[37_u8, 54_u8, 126_u8, 0_u8])// colors[0]
            .read(&[37_u8, 54_u8, 255_u8, 0_u8])// colors[1]
            .build();

        assert_eq!(stream.read_value::<Mode>(2).await?, Mode {
            name: "test".to_string(),
            value: 46,
            flags: HasDirection | HasSpeed | HasBrightness,
            speed_min: Some(10),
            speed_max: Some(1000),
            brightness_min: None,
            brightness_max: None,
            colors_min: Some(0),
            colors_max: Some(256),
            speed: Some(51),
            brightness: None,
            direction: Some(Direction::Horizontal),
            color_mode: Some(ColorMode::PerLED),
            colors: vec![
                Color { r: 37, g: 54, b: 126 },
                Color { r: 37, g: 54, b: 255 },
            ],
        });

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .write(&5_u16.to_le_bytes()) // name len
            .write(b"test\0") // name
            .write(&46_i32.to_le_bytes()) // value
            .write(&31_u32.to_le_bytes()) // flags
            .write(&10_u32.to_le_bytes()) // speed_min
            .write(&1000_u32.to_le_bytes()) // speed_max
            .write(&1_u32.to_le_bytes()) // brightness_min
            .write(&1024_u32.to_le_bytes()) // brightness_max
            .write(&0_u32.to_le_bytes()) // colors_min
            .write(&256_u32.to_le_bytes()) // colors_max
            .write(&51_u32.to_le_bytes()) // speed
            .write(&512_u32.to_le_bytes()) // brightness
            .write(&4_u32.to_le_bytes()) // direction
            .write(&1_u32.to_le_bytes()) // color_mode
            .write(&2_u16.to_le_bytes()) // colors len
            .write(&[37_u8, 54_u8, 126_u8, 0_u8])// colors[0]
            .write(&[37_u8, 54_u8, 255_u8, 0_u8])// colors[1]
            .build();

        stream.write_value(Mode {
            name: "test".to_string(),
            value: 46,
            flags: HasDirection | HasSpeed | HasBrightness,
            speed_min: Some(10),
            speed_max: Some(1000),
            brightness_min: Some(1),
            brightness_max: Some(1024),
            colors_min: Some(0),
            colors_max: Some(256),
            speed: Some(51),
            brightness: Some(512),
            direction: Some(Direction::Horizontal),
            color_mode: Some(ColorMode::PerLED),
            colors: vec![
                Color { r: 37, g: 54, b: 126 },
                Color { r: 37, g: 54, b: 255 },
            ],
        }, DEFAULT_PROTOCOL).await?;

        Ok(())
    }
}
