/*!
# splEngine.rs
*pron. splen-gin*

An engine. You know, for doing things?

---

See module-level documentation for more details.
*/

pub mod ssh;
#[cfg(feature = "ssh")]
pub use ssh::SSH;

mod tcp;
#[cfg(feature = "tcp")]
pub use tcp::tcp;

mod common;
#[cfg(any(feature = "ssh", feature = "tcp"))]
pub use common::*;

pub mod util;
