#[cfg(feature = "bevy")]
pub mod bevy;
pub mod standalone;

pub mod prelude {
    #[cfg(feature = "bevy")]
    pub use crate::bevy::*;
    pub use crate::standalone::*;
}
