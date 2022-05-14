use async_trait::async_trait;
use log::debug;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use OpenRGBError::*;

use crate::data::{OpenRGBReadable, OpenRGBWritable, PacketId};
use crate::OpenRGBError;

static MAGIC: [u8; 4] = *b"ORGB";

#[async_trait]
pub trait OpenRGBReadableStream: AsyncReadExt + Sized + Send + Sync + Unpin {
    async fn read_value<T: OpenRGBReadable>(&mut self, protocol: u32) -> Result<T, OpenRGBError> {
        T::read(self, protocol).await
    }

    async fn read_header(&mut self, protocol: u32, expected_device_id: u32, expected_packet_id: PacketId) -> Result<usize, OpenRGBError> {
        debug!("Reading {:?} packet...", expected_packet_id);

        for c in MAGIC {
            if self.read_u8().await? != c {
                return Err(ProtocolError(format!("expected OpenRGB magic value, got \"{}\"", c)));
            }
        }

        let device_id = self.read_value::<u32>(protocol).await?;
        if device_id != expected_device_id {
            return Err(ProtocolError(format!("expected device ID {}, got {}", expected_device_id, device_id)));
        }

        let packet_id = self.read_value::<PacketId>(protocol).await?;
        if packet_id != expected_packet_id {
            return Err(ProtocolError(format!("expected packet ID {:?}, got {:?}", expected_packet_id, packet_id)));
        }

        self.read_value::<u32>(protocol)
            .await?
            .try_into()
            .map_err(|e| ProtocolError(format!("received invalid data length: {}", e)))
    }

    async fn read_packet<O: OpenRGBReadable>(&mut self, protocol: u32, expected_device_id: u32, expected_packet_id: PacketId) -> Result<O, OpenRGBError> {
        self.read_header(protocol, expected_device_id, expected_packet_id).await?;
        // TODO check header length vs actual read length
        self.read_value(protocol).await
    }
}

#[async_trait]
pub trait OpenRGBWritableStream: AsyncWriteExt + Sized + Send + Sync + Unpin {
    async fn write_value<T: OpenRGBWritable>(&mut self, value: T, protocol: u32) -> Result<(), OpenRGBError> {
        T::write(value, self, protocol).await
    }

    async fn write_header(&mut self, protocol: u32, device_id: u32, packet_id: PacketId, data_len: usize) -> Result<(), OpenRGBError> {
        debug!("Sending {:?} packet of {} bytes...", packet_id, data_len);
        self.write_all(&MAGIC).await?;
        self.write_value(device_id, protocol).await?;
        self.write_value(packet_id, protocol).await?;
        self.write_value(data_len, protocol).await?;
        Ok(())
    }

    async fn write_packet<I: OpenRGBWritable>(&mut self, protocol: u32, device_id: u32, packet_id: PacketId, data: I) -> Result<(), OpenRGBError> {
        let size = data.size(protocol);

        // in debug builds, use intermediate buffer to ease debugging with Wireshark (see #3)
        #[cfg(debug_assertions)] {
            let mut buf: Vec<u8> = Vec::with_capacity(4 /* magic */ + 4 /* device id */ + 4 /* packet id */ + 4 /* len */ + size /* payload size*/);
            buf.write_header(protocol, device_id, packet_id, size).await?;
            buf.write_value(data, protocol).await?;
            self.write_all(&buf).await?;
        }

        // in release builds, write directly
        #[cfg(not(debug_assertions))] {
            self.write_header(protocol, device_id, packet_id, size).await?;
            self.write_value(data, protocol).await?;
        }

        Ok(())
    }
}

#[async_trait]
pub trait OpenRGBStream: OpenRGBReadableStream + OpenRGBWritableStream {
    async fn request<I: OpenRGBWritable, O: OpenRGBReadable>(&mut self, protocol: u32, device_id: u32, packet_id: PacketId, data: I) -> Result<O, OpenRGBError> {
        self.write_packet(protocol, device_id, packet_id, data).await?;
        self.read_packet(protocol, device_id, packet_id).await
    }
}

impl OpenRGBReadableStream for TcpStream {}

impl OpenRGBWritableStream for TcpStream {}

impl OpenRGBStream for TcpStream {}

#[cfg(debug_assertions)]
impl OpenRGBWritableStream for Vec<u8> {}
