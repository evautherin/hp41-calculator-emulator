#![deny(clippy::unwrap_used)]
//! hp41-core — HP-41 calculator behavioral emulation library.
//!
//! Zero UI/CLI dependencies. All state is in [`state::CalcState`].

pub mod cardreader;
pub mod error;
pub mod format;
pub mod num;
pub mod ops;
pub mod stack;
pub mod state;

// Convenience re-exports for consumers
pub use error::HpError;
pub use format::{format_alpha, format_hpnum, format_hpnum_lcd};
pub use num::{HpNum, HpValue};
pub use ops::program::{resume_program, resume_program_with_key, run_program};
pub use ops::{StoArithKind, TestKind};
pub use stack::LiftEffect;
pub use state::{AngleMode, CalcState, DisplayMode, Stack};

#[cfg(test)]
mod tests;
