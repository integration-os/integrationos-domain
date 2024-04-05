pub mod algebra;
pub mod domain;
pub mod service;
pub mod util;

pub use crate::domain::*;

pub mod prelude {
    pub use crate::algebra::*;
    pub use crate::domain::*;
    pub use crate::service::*;
    pub use crate::util::*;
}
