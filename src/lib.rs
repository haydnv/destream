//! Provides streaming/async analogues of [`serde::de::Deserialize`], [`serde::de::Deserializer`],
//! [`serde::ser::Serialize`], and [`serde::ser::Serializer`] called [`FromStream`], [`Decoder`],
//! [`ToStream`] and [`Encoder`].
//!
//! The structure of this crate and its trait and macro definitions are based on `serde` but not
//! compatible with it (primarily because `serde` doesn't support `async`). Many traits and macros
//! are copied directly from `serde` with minimal modification.
//!
//! [`serde`] is dual-licensed under the MIT and Apache-2.0 licenses, which are available at
//! [https://github.com/serde-rs/serde/blob/master/LICENSE-MIT](https://github.com/serde-rs/serde/blob/master/LICENSE-MIT)
//! and [https://github.com/serde-rs/serde/blob/master/LICENSE-APACHE](https://github.com/serde-rs/serde/blob/master/LICENSE-APACHE)
//! respectively.

pub mod de;
pub mod en;

pub use de::*;
pub use en::*;
