//! OpenRGB data types.
//!
//! See [OpenRGB SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
use async_trait::async_trait;

pub use color::*;
pub use color_mode::*;
pub use controller::*;
pub use device_type::*;
pub use direction::*;
pub use led::*;
pub use mode::*;
pub use mode_flag::*;
#[doc(hidden)]
pub use packet::*;
pub use primitive::*;
pub use string::*;
pub use tuple::*;
pub use vec::*;
pub use zone::*;
pub use zone_type::*;

use crate::OpenRGBError;
use crate::protocol::{OpenRGBReadableStream, OpenRGBWritableStream};

mod controller;
mod direction;
mod color_mode;
mod device_type;
mod zone_type;
mod mode_flag;
mod mode;
mod zone;
mod led;
mod color;
mod string;
mod vec;
mod primitive;
mod tuple;
mod packet;

#[async_trait]
#[doc(hidden)]
pub trait OpenRGBReadable: Sized + Send + Sync {
    async fn read(stream: &mut impl OpenRGBReadableStream, protocol: u32) -> Result<Self, OpenRGBError>;
}

#[async_trait]
#[doc(hidden)]
pub trait OpenRGBWritable: Sized + Send + Sync {
    fn size(&self, protocol: u32) -> usize;
    async fn write(self, stream: &mut impl OpenRGBWritableStream, protocol: u32) -> Result<(), OpenRGBError>;
}
