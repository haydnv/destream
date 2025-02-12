//! Stream deserialization framework with an interface based on `serde::de`.
//!
//! The two most important traits in this module are [`FromStream`] and [`Decoder`].
//!
//!  - **A type that implements [`FromStream`] is a data structure** that can be decoded from any
//!  stream encoding supported by `destream`, and conversely
//!  - **A type that implements `Decoder` is a data format** that can decode any supported stream.
//!
//! # The FromStream trait
//!
//! `destream` implements [`FromStream`] for many Rust primitive and standard library types.
//! The complete list is below.
//!
//! # Implementations of FromStream provided by destream
//!
//!  - **Primitive types**:
//!    - ()
//!    - bool
//!    - i8, i16, i32, i64
//!    - u8, u16, u32, u64, usize
//!    - f32, f64
//!  - **Compound types**:
//!    - \[T; 0\] through \[T; 32\]
//!    - tuples up to size 16
//!  - **Common standard library types**:
//!    - String
//!    - Option\<T\>
//!    - PhantomData\<T\>
//!  - **Other common types**:
//!    - Bytes
//!    - Uuid
//!  - **Collection types**:
//!    - BTreeMap\<K, V\>
//!    - BTreeSet\<T\>
//!    - BinaryHeap\<T\>
//!    - HashMap\<K, V, H\>
//!    - HashSet\<T, H\>
//!    - LinkedList\<T\>
//!    - VecDeque\<T\>
//!    - Vec\<T\>
//!
//! Enable support for `SmallVec` using the `smallvec` feature flag.

use std::fmt;

mod impls;

mod size_hint {
    use std::cmp;

    #[inline]
    pub fn cautious(hint: Option<usize>) -> usize {
        cmp::min(hint.unwrap_or(0), 4096)
    }
}

/// The `Error` trait allows [`FromStream`] implementations to create descriptive
/// error messages belonging to their [`Decoder`] context.
///
/// Most implementors should only need to provide the `Error::custom` method
/// and inherit the default behavior for the other methods.
///
/// Based on `serde::de::Error`.
pub trait Error: Send + Sized + std::error::Error {
    /// Raised when there is general error when decoding a type.
    /// The message should not be capitalized and should not end with a period.
    fn custom<T: fmt::Display>(msg: T) -> Self;

    /// Raised when [`FromStream`] receives a type different from what it was expecting.
    fn invalid_type<U: fmt::Display, E: fmt::Display>(unexp: U, exp: E) -> Self {
        Error::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
    }

    /// Raised when [`FromStream`] receives a value of the right type but that
    /// is wrong for some other reason.
    fn invalid_value<U: fmt::Display, E: fmt::Display>(unexp: U, exp: E) -> Self {
        Error::custom(format_args!("invalid value: {}, expected {}", unexp, exp))
    }

    /// Raised when decoding a sequence or map and the input data contains too many
    /// or too few elements.
    fn invalid_length<E: fmt::Display>(len: usize, exp: E) -> Self {
        Error::custom(format_args!("invalid length: {}, expected {}", len, exp))
    }
}

/// A data format that can decode a given well-formatted stream using one or more [`Visitor`]s.
///
/// Based on `serde::de::Deserializer`.
#[trait_variant::make(Send)]
pub trait Decoder: Send {
    /// Type to return in case of a decoding error.
    type Error: Error;

    /// Require the `Decoder` to figure out how to drive the visitor based
    /// on what data type is in the input.
    ///
    /// When implementing [`FromStream`], you should avoid relying on
    /// `Decoder::decode_any` unless you need to be told by the
    /// Decoder what type is in the input. Know that relying on
    /// `Decoder::decode_any` means your data type will be able to
    /// decode self-describing formats only.
    async fn decode_any<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a `bool` value.
    async fn decode_bool<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a binary value.
    async fn decode_bytes<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an `i8` value.
    async fn decode_i8<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an `i16` value.
    async fn decode_i16<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an `i32` value.
    async fn decode_i32<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an `i64` value.
    async fn decode_i64<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a `u8` value.
    async fn decode_u8<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a `u16` value.
    async fn decode_u16<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a `u32` value.
    async fn decode_u32<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a `u64` value.
    async fn decode_u64<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a `f32` value.
    async fn decode_f32<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a `f64` value.
    async fn decode_f64<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an array of `bool`s.
    async fn decode_array_bool<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an array of `i8`s.
    async fn decode_array_i8<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an array of `i16`s.
    async fn decode_array_i16<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an array of `i32`s.
    async fn decode_array_i32<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an array of `i64`s.
    async fn decode_array_i64<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an array of `u8`s.
    async fn decode_array_u8<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an array of `u16`s.
    async fn decode_array_u16<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an array of `u32`s.
    async fn decode_array_u32<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an array of `u64`s.
    async fn decode_array_u64<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an array of `f32`s.
    async fn decode_array_f32<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an array of `f64`s.
    async fn decode_array_f64<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a map of key-value pairs.
    async fn decode_map<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting an optional value.
    ///
    /// This allows decoders that encode an optional value as a nullable
    /// value to convert the null value into `None` and a regular value into
    /// `Some(value)`.
    async fn decode_option<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a sequence of values.
    async fn decode_seq<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a string value.
    async fn decode_string<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a sequence of values and
    /// knows how many values there are without looking at the encoded data.
    async fn decode_tuple<V: Visitor>(
        &mut self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a unit value (i.e. `()`).
    async fn decode_unit<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type is expecting a [`uuid::Uuid`].
    async fn decode_uuid<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the [`FromStream`] type needs to decode a value whose type
    /// doesn't matter because it is ignored.
    ///
    /// Decoders for non-self-describing formats may not support this mode.
    async fn decode_ignored_any<V: Visitor>(&mut self, visitor: V)
        -> Result<V::Value, Self::Error>;
}

/// This trait describes a value which can be decoded from a stream.
///
/// Based on `serde::de::Deserialize`.
#[trait_variant::make(Send)]
pub trait FromStream: Send + Sized {
    /// The decoding context of this type, useful in situations where the stream to be decoded
    /// may be too large to hold in main memory.
    ///
    /// Types intended to be stored entirely in main memory should use the unit context `()`.
    type Context: Send;

    /// Parse this value using the given `Decoder`.
    async fn from_stream<D: Decoder>(
        context: Self::Context,
        decoder: &mut D,
    ) -> Result<Self, D::Error>;
}

/// Provides a [`Visitor`] with access to an array of type `T`.
///
/// This is a trait that a [`Decoder`] passes to a `Visitor` implementation.
#[trait_variant::make(Send)]
pub trait ArrayAccess<T>: Send {
    type Error: Error;

    /// Write array values from the stream being decoded into the given `buffer`.
    ///
    /// Returns the number of values written (this will be in the range `0..buffer.len()`).
    async fn buffer(&mut self, buffer: &mut [T]) -> Result<usize, Self::Error>;
}

/// Provides a [`Visitor`] with access to each entry of a map in the input.
///
/// This is a trait that a [`Decoder`] passes to a `Visitor` implementation.
///
/// Based on `serde::de::MapAccess`.
#[trait_variant::make(Send)]
pub trait MapAccess: Send {
    /// Type to return in case of a decoding error.
    type Error: Error;

    /// This returns `Ok(Some(key))` for the next key in the map, or `Ok(None)`
    /// if there are no more remaining entries.
    ///
    /// `context` is the decoder context used by `K`'s [`FromStream`] impl.
    /// If `K` is small enough to fit in main memory, pass the unit context `()`.
    async fn next_key<K: FromStream>(
        &mut self,
        context: K::Context,
    ) -> Result<Option<K>, Self::Error>;

    /// This returns `Ok(value)` for the next value in the map.
    ///
    /// `context` is the decoder context used by `V`'s [`FromStream`] impl.
    /// If `V` is small enough to fit in main memory, pass the unit context `()`.
    ///
    /// # Panics
    ///
    /// Calling `next_value` before `next_key` is incorrect and is allowed to
    /// panic or return bogus results.
    async fn next_value<V: FromStream>(&mut self, context: V::Context) -> Result<V, Self::Error>;

    /// Returns the number of entries remaining in the map, if known.
    #[inline]
    fn size_hint(&self) -> Option<usize> {
        None
    }
}

/// Provides a [`Visitor`] access to each element of a sequence in the input.
///
/// This is a trait that a [`Decoder`] passes to a `Visitor` implementation,
/// which decodes each item in a sequence.
///
/// Based on `serde::de::SeqAccess`.
#[trait_variant::make(Send)]
pub trait SeqAccess: Send {
    /// The type to return if decoding encounters an error.
    type Error: Error;

    /// Returns `Ok(Some(value))` for the next value in the sequence,
    /// or `Ok(None)` if there is no next item of type `T`.
    ///
    /// `context` is the decoder context used by `T`'s [`FromStream`] impl.
    /// If `T` is small enough to fit in main memory, pass the unit context `()`.
    async fn next_element<T: FromStream>(
        &mut self,
        context: T::Context,
    ) -> Result<Option<T>, Self::Error>;

    /// Returns `Ok(Some(value))` for the next value in the sequence,
    /// or an error if there is no next item or it's not the required type.
    ///
    /// `context` is the decoder context used by `T`'s [`FromStream`] impl.
    /// If `T` is small enough to fit in main memory, pass the unit context `()`.
    async fn expect_next<T: FromStream>(&mut self, context: T::Context) -> Result<T, Self::Error> {
        async {
            if let Some(element) = self.next_element(context).await? {
                Ok(element)
            } else {
                Err(Error::custom("expected sequence element is missing"))
            }
        }
    }

    /// Returns the number of elements remaining in the sequence, if known.
    #[inline]
    fn size_hint(&self) -> Option<usize> {
        None
    }
}

/// This trait describes a visitor responsible for decoding a stream.
///
/// Based on `serde::de::Visitor`.
#[trait_variant::make(Send)]
pub trait Visitor: Send + Sized {
    /// The type which this [`Visitor`] is responsible for decoding.
    type Value;

    /// Format a message stating what data this [`Visitor`] expects to receive.
    ///
    /// This is used in error messages. The message should complete the sentence
    /// "This Visitor expects to receive ...", for example the message could be
    /// "an integer between 0 and 64". The message should not be capitalized and
    /// should not end with a period.
    fn expecting() -> &'static str;

    /// The input contains a boolean.
    ///
    /// The default implementation fails with a type error.
    fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
        Err(Error::invalid_type(v, Self::expecting()))
    }

    /// The input contains an `i8`.
    ///
    /// The default implementation forwards to [`visit_i64`].
    ///
    /// [`visit_i64`]: #method.visit_i64
    #[inline]
    fn visit_i8<E: Error>(self, v: i8) -> Result<Self::Value, E> {
        self.visit_i64(v as i64)
    }

    /// The input contains an `i16`.
    ///
    /// The default implementation forwards to [`visit_i64`].
    ///
    /// [`visit_i64`]: #method.visit_i64
    #[inline]
    fn visit_i16<E: Error>(self, v: i16) -> Result<Self::Value, E> {
        self.visit_i64(v as i64)
    }

    /// The input contains an `i32`.
    ///
    /// The default implementation forwards to [`visit_i64`].
    ///
    /// [`visit_i64`]: #method.visit_i64
    #[inline]
    fn visit_i32<E: Error>(self, v: i32) -> Result<Self::Value, E> {
        self.visit_i64(v as i64)
    }

    /// The input contains an `i64`.
    ///
    /// The default implementation fails with a type error.
    fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
        Err(Error::invalid_type(v, Self::expecting()))
    }

    /// The input contains a `u8`.
    ///
    /// The default implementation forwards to [`visit_u64`].
    ///
    /// [`visit_u64`]: #method.visit_u64
    #[inline]
    fn visit_u8<E: Error>(self, v: u8) -> Result<Self::Value, E> {
        self.visit_u64(v as u64)
    }

    /// The input contains a `u16`.
    ///
    /// The default implementation forwards to [`visit_u64`].
    ///
    /// [`visit_u64`]: #method.visit_u64
    #[inline]
    fn visit_u16<E: Error>(self, v: u16) -> Result<Self::Value, E> {
        self.visit_u64(v as u64)
    }

    /// The input contains a `u32`.
    ///
    /// The default implementation forwards to [`visit_u64`].
    ///
    /// [`visit_u64`]: #method.visit_u64
    #[inline]
    fn visit_u32<E: Error>(self, v: u32) -> Result<Self::Value, E> {
        self.visit_u64(v as u64)
    }

    /// The input contains a `u64`.
    ///
    /// The default implementation fails with a type error.
    fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
        Err(Error::invalid_type(v, Self::expecting()))
    }

    /// The input contains an `f32`.
    ///
    /// The default implementation forwards to [`visit_f64`].
    ///
    /// [`visit_f64`]: #method.visit_f64
    #[inline]
    fn visit_f32<E: Error>(self, v: f32) -> Result<Self::Value, E> {
        self.visit_f64(v as f64)
    }

    /// The input contains an `f64`.
    ///
    /// The default implementation fails with a type error.
    fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
        Err(Error::invalid_type(v, Self::expecting()))
    }

    /// The input contains an array of `bool`s.
    ///
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_array_bool<A: ArrayAccess<bool>>(
        self,
        array: A,
    ) -> Result<Self::Value, A::Error> {
        async { Err(Error::invalid_type("boolean array", Self::expecting())) }
    }

    /// The input contains an array of `i8`s.
    ///
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_array_i8<A: ArrayAccess<i8>>(self, array: A) -> Result<Self::Value, A::Error> {
        async { Err(Error::invalid_type("i8 array", Self::expecting())) }
    }

    /// The input contains an array of `i16`s.
    ///
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_array_i16<A: ArrayAccess<i16>>(self, array: A) -> Result<Self::Value, A::Error> {
        async { Err(Error::invalid_type("i16 array", Self::expecting())) }
    }

    /// The input contains an array of `i32`s.
    ///
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_array_i32<A: ArrayAccess<i32>>(self, array: A) -> Result<Self::Value, A::Error> {
        async { Err(Error::invalid_type("i32 array", Self::expecting())) }
    }

    /// The input contains an array of `i64`s.
    ///
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_array_i64<A: ArrayAccess<i64>>(self, array: A) -> Result<Self::Value, A::Error> {
        async { Err(Error::invalid_type("i64 array", Self::expecting())) }
    }

    /// The input contains an array of `u8`s.
    ///
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_array_u8<A: ArrayAccess<u8>>(self, array: A) -> Result<Self::Value, A::Error> {
        async { Err(Error::invalid_type("u8 array", Self::expecting())) }
    }

    /// The input contains an array of `u16`s.
    ///
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_array_u16<A: ArrayAccess<u16>>(self, array: A) -> Result<Self::Value, A::Error> {
        async { Err(Error::invalid_type("u16 array", Self::expecting())) }
    }

    /// The input contains an array of `u32`s.
    ///
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_array_u32<A: ArrayAccess<u32>>(self, array: A) -> Result<Self::Value, A::Error> {
        async { Err(Error::invalid_type("u32 array", Self::expecting())) }
    }

    /// The input contains an array of `u64`s.
    ///
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_array_u64<A: ArrayAccess<u64>>(self, array: A) -> Result<Self::Value, A::Error> {
        async { Err(Error::invalid_type("u64 array", Self::expecting())) }
    }

    /// The input contains an array of `f32`s.
    ///
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_array_f32<A: ArrayAccess<f32>>(self, array: A) -> Result<Self::Value, A::Error> {
        async { Err(Error::invalid_type("f32 array", Self::expecting())) }
    }

    /// The input contains an array of `f64`s.
    ///
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_array_f64<A: ArrayAccess<f64>>(self, array: A) -> Result<Self::Value, A::Error> {
        async { Err(Error::invalid_type("f64 array", Self::expecting())) }
    }

    /// The input contains a string and ownership of the string is being given
    /// to the [`Visitor`].
    ///
    /// The default implementation fails with a type error.
    fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
        Err(Error::invalid_type(v, Self::expecting()))
    }

    /// The input contains a unit `()`.
    ///
    /// The default implementation fails with a type error.
    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        Err(Error::invalid_type("unit", Self::expecting()))
    }

    /// The input contains an optional that is absent.
    /// The default implementation fails with a type error.
    fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
        Err(Error::invalid_type("Option::None", Self::expecting()))
    }

    /// The input contains an optional that is present.
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_some<D: Decoder>(self, decoder: &mut D) -> Result<Self::Value, D::Error> {
        async { Err(Error::invalid_type("Option::Some", Self::expecting())) }
    }

    /// The input contains a key-value map.
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_map<A: MapAccess>(self, map: A) -> Result<Self::Value, A::Error> {
        async { Err(Error::invalid_type("map", Self::expecting())) }
    }

    /// The input contains a sequence of elements.
    /// The default implementation fails with a type error.
    #[allow(unused_variables)]
    async fn visit_seq<A: SeqAccess>(self, seq: A) -> Result<Self::Value, A::Error> {
        async { Err(Error::invalid_type("sequence", Self::expecting())) }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct IgnoredAny;
