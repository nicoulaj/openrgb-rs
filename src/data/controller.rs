use async_trait::async_trait;

use crate::data::{Color, DeviceType, LED, Mode, OpenRGBReadable, Zone};
use crate::OpenRGBError;
use crate::protocol::OpenRGBReadableStream;

/// RGB controller.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_controller_data) for more information.
#[derive(Debug, Eq, PartialEq)]
pub struct Controller {
    /// Controller type.
    pub r#type: DeviceType,

    /// Controller name.
    pub name: String,

    /// Controller vendor.
    pub vendor: String,

    /// Controller description.
    pub description: String,

    /// Controller version.
    pub version: String,

    /// Controller serial.
    pub serial: String,

    /// Controller location.
    pub location: String,

    /// Controller active mode index.
    pub active_mode: i32,

    /// Controller modes.
    pub modes: Vec<Mode>,

    /// Controller zones.
    pub zones: Vec<Zone>,

    /// Controller LEDs.
    pub leds: Vec<LED>,

    /// Controller colors.
    pub colors: Vec<Color>,
}

#[async_trait]
impl OpenRGBReadable for Controller {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError> {
        let _data_size = stream.read_value::<u32>(protocol).await?;
        let r#type = stream.read_value(protocol).await?;
        let name = stream.read_value(protocol).await?;
        let vendor = stream.read_value(protocol).await?;
        let description = stream.read_value(protocol).await?;
        let version = stream.read_value(protocol).await?;
        let serial = stream.read_value(protocol).await?;
        let location = stream.read_value(protocol).await?;
        let num_modes = stream.read_value::<u16>(protocol).await?;
        let active_mode = stream.read_value(protocol).await?;
        let mut modes = Vec::with_capacity(num_modes as usize);
        for _ in 0..num_modes {
            modes.push(stream.read_value(protocol).await?);
        }
        let zones = stream.read_value(protocol).await?;
        let leds = stream.read_value(protocol).await?;
        let colors = stream.read_value(protocol).await?;

        Ok(Controller {
            r#type,
            name,
            vendor,
            description,
            version,
            serial,
            location,
            active_mode,
            modes,
            zones,
            leds,
            colors,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use ModeFlag::*;

    use crate::data::{Color, ColorMode, Controller, DeviceType, Mode, ModeFlag, Zone, ZoneType};
    use crate::DEFAULT_PROTOCOL;
    use crate::protocol::OpenRGBReadableStream;
    use crate::tests::setup;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        setup()?;

        let mut stream = Builder::new()
            .read(&760_u32.to_le_bytes())
            .read(&[3, 0, 0, 0, 18, 0, 84, 104, 101, 114, 109, 97, 108, 116, 97, 107, 101, 32,
                82, 105, 105, 110, 103, 0, 12, 0, 84, 104, 101, 114, 109, 97, 108, 116, 97, 107,
                101, 0, 25, 0, 84, 104, 101, 114, 109, 97, 108, 116, 97, 107, 101, 32, 82, 105, 105,
                110, 103, 32, 68, 101, 118, 105, 99, 101, 0, 1, 0, 0, 1, 0, 0, 19, 0, 72, 73, 68,
                58, 32, 47, 100, 101, 118, 47, 104, 105, 100, 114, 97, 119, 49, 48, 0, 8, 0, 0, 0,
                0, 0, 7, 0, 68, 105, 114, 101, 99, 116, 0, 24, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1, 0, 0, 0, 0, 0, 7, 0, 83, 116, 97, 116, 105, 99, 0, 25, 0, 0, 0, 64, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 5, 0, 70, 108, 111, 119,
                0, 0, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 83, 112,
                101, 99, 116, 114, 117, 109, 0, 4, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 7, 0, 82, 105, 112, 112, 108, 101, 0, 8, 0, 0, 0, 33, 0, 0, 0, 3, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 6, 0, 66, 108, 105, 110, 107, 0, 12, 0, 0, 0,
                33, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 6, 0, 80, 117, 108, 115, 101,
                0, 16, 0, 0, 0, 33, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 5, 0, 87,
                97, 118, 101, 0, 20, 0, 0, 0, 33, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
                5, 0, 16, 0, 82, 105, 105, 110, 103, 32, 67, 104, 97, 110, 110, 101, 108, 32, 49, 0,
                1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 82, 105, 105, 110,
                103, 32, 67, 104, 97, 110, 110, 101, 108, 32, 50, 0, 1, 0, 0, 0, 0, 0, 0, 0, 20, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 82, 105, 105, 110, 103, 32, 67, 104, 97, 110, 110,
                101, 108, 32, 51, 0, 1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0,
                82, 105, 105, 110, 103, 32, 67, 104, 97, 110, 110, 101, 108, 32, 52, 0, 1, 0, 0, 0,
                0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 82, 105, 105, 110, 103, 32, 67,
                104, 97, 110, 110, 101, 108, 32, 53, 0, 1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0])
            .build();

        assert_eq!(stream.read_value::<Controller>(DEFAULT_PROTOCOL).await?, Controller {
            r#type: DeviceType::Cooler,
            name: "Thermaltake Riing".to_string(),
            vendor: "Thermaltake".to_string(),
            description: "Thermaltake Riing Device".to_string(),
            version: "".to_string(),
            serial: "".to_string(),
            location: "HID: /dev/hidraw10".to_string(),
            active_mode: 0,
            modes: vec![
                Mode {
                    name: "Direct".to_string(),
                    value: 24,
                    flags: HasPerLEDColor.into(),
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
                },
                Mode {
                    name: "Static".to_string(),
                    value: 25,
                    flags: HasModeSpecificColor.into(),
                    speed_min: None,
                    speed_max: None,
                    brightness_min: None,
                    brightness_max: None,
                    colors_min: Some(1),
                    colors_max: Some(1),
                    speed: None,
                    brightness: None,
                    direction: None,
                    color_mode: Some(ColorMode::ModeSpecific),
                    colors: vec![Color { r: 0, g: 0, b: 0 }],
                },
                Mode {
                    name: "Flow".to_string(),
                    value: 0,
                    flags: HasSpeed.into(),
                    speed_min: Some(3),
                    speed_max: Some(0),
                    brightness_min: None,
                    brightness_max: None,
                    colors_min: None,
                    colors_max: None,
                    speed: Some(2),
                    brightness: None,
                    direction: None,
                    color_mode: Some(ColorMode::None),
                    colors: vec![],
                },
                Mode {
                    name: "Spectrum".to_string(),
                    value: 4,
                    flags: HasSpeed.into(),
                    speed_min: Some(3),
                    speed_max: Some(0),
                    brightness_min: None,
                    brightness_max: None,
                    colors_min: None,
                    colors_max: None,
                    speed: Some(2),
                    brightness: None,
                    direction: None,
                    color_mode: Some(ColorMode::None),
                    colors: vec![],
                },
                Mode {
                    name: "Ripple".to_string(),
                    value: 8,
                    flags: HasSpeed | HasPerLEDColor,
                    speed_min: Some(3),
                    speed_max: Some(0),
                    brightness_min: None,
                    brightness_max: None,
                    colors_min: None,
                    colors_max: None,
                    speed: Some(2),
                    brightness: None,
                    direction: None,
                    color_mode: Some(ColorMode::PerLED),
                    colors: vec![],
                },
                Mode {
                    name: "Blink".to_string(),
                    value: 12,
                    flags: HasSpeed | HasPerLEDColor,
                    speed_min: Some(3),
                    speed_max: Some(0),
                    brightness_min: None,
                    brightness_max: None,
                    colors_min: None,
                    colors_max: None,
                    speed: Some(2),
                    brightness: None,
                    direction: None,
                    color_mode: Some(ColorMode::PerLED),
                    colors: vec![],
                },
                Mode {
                    name: "Pulse".to_string(),
                    value: 16,
                    flags: HasSpeed | HasPerLEDColor,
                    speed_min: Some(3),
                    speed_max: Some(0),
                    brightness_min: None,
                    brightness_max: None,
                    colors_min: None,
                    colors_max: None,
                    speed: Some(2),
                    brightness: None,
                    direction: None,
                    color_mode: Some(ColorMode::PerLED),
                    colors: vec![],
                },
                Mode {
                    name: "Wave".to_string(),
                    value: 20,
                    flags: HasSpeed | HasPerLEDColor,
                    speed_min: Some(3),
                    speed_max: Some(0),
                    brightness_min: None,
                    brightness_max: None,
                    colors_min: None,
                    colors_max: None,
                    speed: Some(2),
                    brightness: None,
                    direction: None,
                    color_mode: Some(ColorMode::PerLED),
                    colors: vec![],
                },
            ],
            zones: vec![
                Zone {
                    name: "Riing Channel 1".to_string(),
                    r#type: ZoneType::Linear,
                    leds_min: 0,
                    leds_max: 20,
                    leds_count: 0,
                    matrix: None,
                },
                Zone {
                    name: "Riing Channel 2".to_string(),
                    r#type: ZoneType::Linear,
                    leds_min: 0,
                    leds_max: 20,
                    leds_count: 0,
                    matrix: None,
                },
                Zone {
                    name: "Riing Channel 3".to_string(),
                    r#type: ZoneType::Linear,
                    leds_min: 0,
                    leds_max: 20,
                    leds_count: 0,
                    matrix: None,
                },
                Zone {
                    name: "Riing Channel 4".to_string(),
                    r#type: ZoneType::Linear,
                    leds_min: 0,
                    leds_max: 20,
                    leds_count: 0,
                    matrix: None,
                },
                Zone {
                    name: "Riing Channel 5".to_string(),
                    r#type: ZoneType::Linear,
                    leds_min: 0,
                    leds_max: 20,
                    leds_count: 0,
                    matrix: None,
                },
            ],
            leds: vec![],
            colors: vec![],
        });

        Ok(())
    }
}
