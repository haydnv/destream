//! Provides traits [`FromStream`], [`Decoder`], [`ToStream`] and [`Encoder`], which are
//! streaming/async analogues of [`serde`]'s [`Deserialize`], [`Deserializer`], [`Serialize`],
//! and [`Serializer`].
//!
//! [`Deserialize`]:[`serde::de::Deserialize`]
//! [`Deserializer`]:[`serde::de::Deserializer`]
//! [`Serialize`]:[`serde::ser::Serialize`]
//! [`Serializer`]:[`serde::ser::Serializer`]
//!
//! The structure and contents of this crate are based on `serde` but not compatible with it
//! (primarily because `serde` doesn't support `async`). Most of the code which makes up `destream`
//! is copied directly from `serde` with minimal modifications.
//!
//! [`serde`] is dual-licensed under the MIT and Apache-2.0 licenses, which are available at
//! [https://github.com/serde-rs/serde/blob/master/LICENSE-MIT](https://github.com/serde-rs/serde/blob/master/LICENSE-MIT)
//! and [https://github.com/serde-rs/serde/blob/master/LICENSE-APACHE](https://github.com/serde-rs/serde/blob/master/LICENSE-APACHE)
//! respectively.
//!
//! Important differences between `destream` and `serde`:
//!  - `destream` supports decoding from and encoding to a `futures::Stream` (obviously).
//!  - `destream` does not (yet) support the `derive` macro, so you can't derive `FromStream` or
//!     `ToStream`, and there is no built-in functionality for decoding/encoding a given `struct`.
//!  - `Decoder` assumes the static lifetime and only supports owned types, but `Encoder` uses a
//!     specific lifetime `'en`. This is the opposite of `serde`.
//!
//! `destream` itself does not implement support for any specific serialization format.
//! [`destream_json`] provides support for streaming JSON.
//!
//! [`destream_json`]: http://docs.rs/destream_json/

pub mod de;
pub mod en;

pub use de::*;
pub use en::*;
