#![warn(missing_docs)]

//! See `BoxFnOnce` and `SendBoxFnOnce`.

#[macro_use]
mod macros;

mod no_send;
pub use self::no_send::BoxFnOnce;

mod send;
pub use self::send::SendBoxFnOnce;
