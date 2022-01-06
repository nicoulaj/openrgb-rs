use std::error::Error;
use std::sync::Once;

use async_trait::async_trait;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TerminalMode, TermLogger};
use tokio_test::io::{Builder, Mock};

use crate::{DEFAULT_PROTOCOL, OpenRGB, OpenRGBError};
use crate::protocol::{OpenRGBReadableStream, OpenRGBStream, OpenRGBWritableStream};

impl OpenRGBReadableStream for Mock {}

impl OpenRGBWritableStream for Mock {}

impl OpenRGBStream for Mock {}

static INIT_ONCE: Once = Once::new();

pub fn setup() -> Result<(), Box<dyn Error>> {
    INIT_ONCE.call_once(|| CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::default(),
        ColorChoice::Auto,
    )]).expect("failed initializing logger"));

    Ok(())
}

#[async_trait]
pub trait OpenRGBMockBuilder<S: OpenRGBStream> {
    async fn to_client(&mut self) -> Result<OpenRGB<S>, OpenRGBError>;
    fn negotiate_default_protocol(&mut self) -> &mut Self;
    fn negotiate_protocol(&mut self, protocol: u32) -> &mut Self;
}

#[async_trait]
impl OpenRGBMockBuilder<Mock> for Builder {
    async fn to_client(&mut self) -> Result<OpenRGB<Mock>, OpenRGBError> {
        OpenRGB::new(self.build()).await
    }

    fn negotiate_default_protocol(&mut self) -> &mut Self {
        self.negotiate_protocol(DEFAULT_PROTOCOL)
    }

    fn negotiate_protocol(&mut self, protocol: u32) -> &mut Self {
        self

            // request protocol version request
            .write(b"ORGB") // magic
            .write(&0_u32.to_le_bytes()) // device id
            .write(&40_u32.to_le_bytes()) // packet id
            .write(&4_u32.to_le_bytes()) // data size
            .write(&DEFAULT_PROTOCOL.to_le_bytes()) // protocol version

            // request protocol version response
            .read(b"ORGB") // magic
            .read(&0_u32.to_le_bytes()) // device id
            .read(&40_u32.to_le_bytes()) // packet id
            .read(&4_u32.to_le_bytes()) // data size
            .read(&protocol.to_le_bytes()) // protocol version
    }
}
