pub type GraphemeIndex = usize;
pub type LineIndex = usize;
pub type ByteIndex = usize;
pub type ColIndex = usize;
pub type RowIndex = usize;

mod location;
mod position;
mod size;

pub use location::Location;
pub use position::Position;
pub use size::Size;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");