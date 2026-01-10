pub mod standalone;
#[cfg(feature = "bevy")]
pub mod bevy;

pub mod prelude {
    pub use crate::standalone::*;
    #[cfg(feature = "bevy")]
    pub use crate::bevy::*;
}