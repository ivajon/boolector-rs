mod btor;
pub use btor::Btor;
pub use btor::{Bitwuzla, SolverResult};
mod array;
mod bool;
mod bv;
pub mod fp;
mod rounding_mode;
mod util;
pub use array::Array;
pub use bool::Bool;
pub use bv::{BVSolution, BV};
use fp::FPError;
pub use fp::FP;
pub use rounding_mode::{RoundingMode, RoundingModeNode};
pub mod option;
pub mod options;
pub use options::BitwuzlaOptions;
mod sort;

/// Enumerates the errors that may occur when using these bindings.
///
///
/// ## NOTE
///
/// These do not include internal bitwuzla errors.
#[derive(Clone, Debug)]
pub enum BitWuzlaError {
    FPError(FPError),
}
