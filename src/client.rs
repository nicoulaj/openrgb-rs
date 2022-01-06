use std::fmt::Debug;
use std::net::Ipv4Addr;
use std::sync::Arc;

use log::debug;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::Mutex;

use OpenRGBError::*;
use PacketId::*;

use crate::data::{Color, Controller, Mode, OpenRGBWritable, PacketId, RawString};
use crate::OpenRGBError;
use crate::protocol::OpenRGBStream;

/// Default protocol version used by [OpenRGB] client.
pub static DEFAULT_PROTOCOL: u32 = 3;

/// Default address used by [OpenRGB::connect].
pub static DEFAULT_ADDR: (Ipv4Addr, u16) = (Ipv4Addr::LOCALHOST, 6742);

/// OpenRGB client.
pub struct OpenRGB<S: OpenRGBStream> {
    protocol: u32,
    stream: Arc<Mutex<S>>,
}

impl OpenRGB<TcpStream> {
    /// Connect to default OpenRGB server.
    ///
    /// Use [OpenRGB::connect_to] to connect to a specific server.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use openrgb::OpenRGB;
    /// # use std::error::Error;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let client = OpenRGB::connect().await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect() -> Result<Self, OpenRGBError> {
        Self::connect_to(DEFAULT_ADDR).await
    }

    /// Connect to OpenRGB server at given coordinates.
    ///
    /// Use [OpenRGB::connect] to connect to default server.
    ///
    /// # Arguments
    /// * `addr` - A socket address (eg: a `(host, port)` tuple)
    ///
    /// # Example
    /// ```no_run
    /// # use openrgb::OpenRGB;
    /// # use std::error::Error;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let client = OpenRGB::connect_to(("localhost", 6742)).await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_to(addr: impl ToSocketAddrs + Debug + Copy) -> Result<Self, OpenRGBError> {
        debug!("Connecting to OpenRGB server at {:?}...", addr);
        Self::new(
            TcpStream::connect(addr)
                .await
                .map_err(|source| ConnectionError { addr: format!("{:?}", addr), source })?
        ).await
    }
}

impl<S: OpenRGBStream> OpenRGB<S> {
    /// Build a new client from given stream.
    ///
    /// This constructor expects a connected, ready to use stream.
    pub async fn new(mut stream: S) -> Result<Self, OpenRGBError> {
        let protocol = DEFAULT_PROTOCOL.min(stream.request(
            DEFAULT_PROTOCOL,
            0,
            RequestProtocolVersion,
            DEFAULT_PROTOCOL,
        ).await?);

        debug!("Connected to OpenRGB server using protocol version {:?}", protocol);

        Ok(Self { protocol, stream: Arc::new(Mutex::new(stream)) })
    }

    /// Get protocol version negotiated with server.
    ///
    /// This is the lowest between this client maximum supported version ([DEFAULT_PROTOCOL]) and server version.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#protocol-versions) for more information.
    pub fn get_protocol_version(&self) -> u32 {
        self.protocol
    }

    /// Set client name.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_set_client_name) for more information.
    pub async fn set_name(&self, name: impl Into<String>) -> Result<(), OpenRGBError> {
        self.stream.lock().await.write_packet(
            self.protocol,
            0,
            SetClientName,
            RawString(name.into()),
        ).await
    }

    /// Get number of controllers.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_controller_count) for more information.
    pub async fn get_controller_count(&self) -> Result<u32, OpenRGBError> {
        self.stream.lock().await.request(
            self.protocol,
            0,
            RequestControllerCount,
            (),
        ).await
    }

    /// Get controller data.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_controller_data) for more information.
    pub async fn get_controller(&self, controller_id: u32) -> Result<Controller, OpenRGBError> {
        self.stream.lock().await.request(
            self.protocol,
            controller_id,
            RequestControllerData,
            self.protocol,
        ).await
    }

    /// Resize a controller zone.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_resizezone) for more information.
    pub async fn resize_zone(&self, zone_id: i32, new_size: i32) -> Result<(), OpenRGBError> {
        self.stream.lock().await.write_packet(
            self.protocol,
            0,
            RGBControllerResizeZone,
            (zone_id, new_size),
        ).await
    }

    /// Update a single LED.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_updatesingleled) for more information.
    pub async fn update_led(&self, controller_id: u32, led_id: i32, color: Color) -> Result<(), OpenRGBError> {
        self.stream.lock().await.write_packet(
            self.protocol,
            controller_id,
            RGBControllerUpdateSingleLed,
            (led_id, color),
        ).await
    }

    /// Update LEDs.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_updateleds) for more information.
    pub async fn update_leds(&self, controller_id: u32, colors: Vec<Color>) -> Result<(), OpenRGBError> {
        self.stream.lock().await.write_packet(
            self.protocol,
            controller_id,
            RGBControllerUpdateLeds,
            (colors.size(self.protocol), colors),
        ).await
    }

    /// Update a zone LEDs.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_updatezoneleds) for more information.
    pub async fn update_zone_leds(&self, controller_id: u32, zone_id: u32, colors: Vec<Color>) -> Result<(), OpenRGBError> {
        self.stream.lock().await.write_packet(
            self.protocol,
            controller_id,
            RGBControllerUpdateZoneLeds,
            (zone_id.size(self.protocol) + colors.size(self.protocol), zone_id, colors),
        ).await
    }

    /// Get profiles.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_profile_list) for more information.
    pub async fn get_profiles(&self) -> Result<Vec<String>, OpenRGBError> {
        self.check_protocol_version_profile_control()?;
        self.stream.lock().await
            .request::<_, (u32, Vec<String>)>(
                self.protocol,
                0,
                RequestProfileList,
                (),
            )
            .await
            .map(|(_size, profiles)| profiles)
    }

    /// Load a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_load_profile) for more information.
    pub async fn load_profile(&self, name: impl Into<String>) -> Result<(), OpenRGBError> {
        self.check_protocol_version_profile_control()?;
        self.stream.lock().await.write_packet(
            self.protocol,
            0,
            RequestLoadProfile,
            name.into(),
        ).await
    }

    /// Save a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_save_profile) for more information.
    pub async fn save_profile(&self, name: impl Into<String>) -> Result<(), OpenRGBError> {
        self.check_protocol_version_profile_control()?;
        self.stream.lock().await.write_packet(
            self.protocol,
            0,
            RequestSaveProfile,
            name.into(),
        ).await
    }

    /// Delete a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_delete_profile) for more information.
    pub async fn delete_profile(&self, name: impl Into<String>) -> Result<(), OpenRGBError> {
        self.check_protocol_version_profile_control()?;
        self.stream.lock().await.write_packet(
            self.protocol,
            0,
            RequestDeleteProfile,
            name.into(),
        ).await
    }

    /// Set custom mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_setcustommode) for more information.
    pub async fn set_custom_mode(&self, controller_id: u32) -> Result<(), OpenRGBError> {
        self.stream.lock().await.write_packet(
            self.protocol,
            controller_id,
            RGBControllerSetCustomMode,
            (),
        ).await
    }

    /// Update a mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_updatemode) for more information.
    pub async fn update_mode(&self, controller_id: u32, mode_id: i32, mode: Mode) -> Result<(), OpenRGBError> {
        self.stream.lock().await.write_packet(
            self.protocol,
            controller_id,
            RGBControllerUpdateMode,
            (mode_id.size(self.protocol) + mode.size(self.protocol), mode_id, mode),
        ).await
    }

    /// Save a mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_savemode) for more information.
    pub async fn save_mode(&self, controller_id: u32, mode: Mode) -> Result<(), OpenRGBError> {
        self.check_protocol_version_saving_modes()?;
        self.stream.lock().await.write_packet(
            self.protocol,
            controller_id,
            RGBControllerSaveMode,
            mode,
        ).await
    }

    fn check_protocol_version_profile_control(&self) -> Result<(), OpenRGBError> {
        if self.protocol < 2 {
            return Err(UnsupportedOperation {
                operation: "Profile control".to_owned(),
                current_protocol_version: self.protocol,
                min_protocol_version: 2,
            });
        }
        Ok(())
    }

    fn check_protocol_version_saving_modes(&self) -> Result<(), OpenRGBError> {
        if self.protocol < 3 {
            return Err(UnsupportedOperation {
                operation: "Saving modes".to_owned(),
                current_protocol_version: self.protocol,
                min_protocol_version: 3,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;

    use crate::tests::{OpenRGBMockBuilder, setup};

    #[tokio::test]
    async fn test_negotiate_protocol_version_3() -> Result<(), Box<dyn Error>> {
        setup()?;

        let client = Builder::new()
            .negotiate_protocol(3)
            .to_client().await?;

        assert_eq!(client.get_protocol_version(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_negotiate_protocol_version_2() -> Result<(), Box<dyn Error>> {
        setup()?;

        let client = Builder::new()
            .negotiate_protocol(2)
            .to_client().await?;

        assert_eq!(client.get_protocol_version(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_set_name() -> Result<(), Box<dyn Error>> {
        setup()?;

        let client = Builder::new()
            .negotiate_default_protocol()
            .write(b"ORGB") // magic
            .write(&0_u32.to_le_bytes()) // device id
            .write(&50_u32.to_le_bytes()) // packet id
            .write(&5_u32.to_le_bytes()) // data size
            .write(b"test\0") // name
            .to_client().await?;

        client.set_name("test").await?;

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_controller_count() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client().await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_controller() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client().await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_zone_leds() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client().await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_resize_zone() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client().await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_save_profile() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client().await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_leds() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client().await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_profile() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client().await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_load_profile() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client().await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_led() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client().await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_profiles() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client().await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_mode() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client().await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_set_custom_mode() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client().await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_save_mode() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client().await?;

        todo!("test not implemented")
    }
}
