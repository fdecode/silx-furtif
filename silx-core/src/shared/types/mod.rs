/// Awakening message definition
mod wake; 
pub use wake::WakeSlx;

/// Convert archive into reference or mutable reference
mod arch_tools; 
pub use arch_tools::{ ArchToDeref, ArchToDerefMut, DerefArch, DerefMutArch, };

/// Convert into slx data
mod into_slx; 
pub use into_slx::{ IntoSlx, SlxFrom, };

/// Convert from slx data
mod slx_into; 
pub use slx_into::{ SlxInto, FromSlx, };
