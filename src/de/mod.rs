use std::fmt;

use async_trait::async_trait;

mod impls;

/// `Expected` represents an explanation of what data a `Visitor` was expecting to receive.
///
/// This is used as an argument to the `invalid_type`, `invalid_value`, and
/// `invalid_length` methods of the `Error` trait to build error messages. The
/// message should be a noun or noun phrase that completes the sentence "This
/// Visitor expects to receive ...", for example the message could be "an
/// integer between 0 and 64". The message should not be capitalized and should
/// not end with a period.
///
/// Within the context of a `Visitor` implementation, the `Visitor` itself
/// (`&self`) is an implementation of this trait.
pub trait Expected {
    /// Format an explanation of what data was being expected. Same signature as
    /// the `Display` and `Debug` traits.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result;
}

impl<V: Visitor> Expected for V {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.expecting(f)
    }
}

impl Expected for str {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self)
    }
}

impl Expected for &'static str {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self)
    }
}

impl Expected for String {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self)
    }
}

impl<'a> fmt::Display for dyn Expected + 'a {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Expected::fmt(self, f)
    }
}

/// The `Error` trait allows `FromStream` implementations to create descriptive
/// error messages belonging to their `Decoder` context.
///
/// Most implementors should only need to provide the `Error::custom` method
/// and inherit the default behavior for the other methods.
///
/// Based on [`serde::de::Error`].
pub trait Error: Sized + std::error::Error {
    /// Raised when there is general error when deserializing a type.
    /// The message should not be capitalized and should not end with a period.
    fn custom<T: fmt::Display>(msg: T) -> Self;

    /// Raised when `FromStream` receives a type different from what it was expecting.
    fn invalid_type<U: fmt::Display>(unexp: U, exp: &dyn Expected) -> Self {
        Error::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
    }

    /// Raised when `FromStream` receives a value of the right type but that
    /// is wrong for some other reason.
    fn invalid_value<U: fmt::Display>(unexp: U, exp: &dyn Expected) -> Self {
        Error::custom(format_args!("invalid value: {}, expected {}", unexp, exp))
    }

    /// Raised when decoding a sequence or map and the input data contains too many
    /// or too few elements.
    fn invalid_length<E>(len: usize, exp: &dyn Expected) -> Self {
        Error::custom(format_args!("invalid length: {}, expected {}", len, exp))
    }

    /// Raised when `FromStream` receives a field with an unrecognized name.
    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        if expected.is_empty() {
            Error::custom(format!("unknown field `{}`, there are no fields", field))
        } else {
            Error::custom(format!(
                "unknown field `{}`, expected one of {}",
                field,
                expected.join(", ")
            ))
        }
    }

    /// Raised when `FromStream` expected to receive a required
    /// field with a particular name but that field was not present in the
    /// input.
    fn missing_field(field: &'static str) -> Self {
        Error::custom(format_args!("missing field `{}`", field))
    }

    /// Raised when `FromStream` receives more than one of the same field.
    fn duplicate_field(field: &'static str) -> Self {
        Error::custom(format_args!("duplicate field `{}`", field))
    }
}

/// A data format that can decode a given well-formatted stream using one or more [`Visitor`]s.
///
/// Based on [`serde::de::Deserializer`].
#[async_trait]
pub trait Decoder: Send {
    /// Type to return in case of a decoding error.
    type Error: Error;

    /// Require the `Decoder` to figure out how to drive the visitor based
    /// on what data type is in the input.
    ///
    /// When implementing `FromStream`, you should avoid relying on
    /// `Decoder::decode_any` unless you need to be told by the
    /// Decoder what type is in the input. Know that relying on
    /// `Decoder::decode_any` means your data type will be able to
    /// deserialize from self-describing formats only, ruling out Bincode and
    /// many others.
    async fn decode_any<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting a `bool` value.
    async fn decode_bool<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting an `i8` value.
    async fn decode_i8<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting an `i16` value.
    async fn decode_i16<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting an `i32` value.
    async fn decode_i32<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting an `i64` value.
    async fn decode_i64<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting a `u8` value.
    async fn decode_u8<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting a `u16` value.
    async fn decode_u16<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting a `u32` value.
    async fn decode_u32<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting a `u64` value.
    async fn decode_u64<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting a `f32` value.
    async fn decode_f32<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting a `f64` value.
    async fn decode_f64<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting a string value.
    async fn decode_string<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting a byte array.
    async fn decode_byte_buf<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting an optional value.
    ///
    /// This allows decoders that encode an optional value as a nullable
    /// value to convert the null value into `None` and a regular value into
    /// `Some(value)`.
    async fn decode_option<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting a sequence of values.
    async fn decode_seq<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting a sequence of values and
    /// knows how many values there are without looking at the serialized data.
    async fn decode_tuple<V: Visitor>(
        &mut self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting a map of key-value pairs.
    async fn decode_map<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type is expecting the name of a struct
    /// field or the discriminant of an enum variant.
    async fn decode_identifier<V: Visitor>(&mut self, visitor: V) -> Result<V::Value, Self::Error>;

    /// Hint that the `FromStream` type needs to decode a value whose type
    /// doesn't matter because it is ignored.
    ///
    /// Decoders for non-self-describing formats may not support this mode.
    async fn decode_ignored_any<V: Visitor>(&mut self, visitor: V)
        -> Result<V::Value, Self::Error>;
}

#[async_trait]
/// This trait describes a value which can be decoded from a stream.
///
/// Based on [`serde::de::Deserialize`].
pub trait FromStream: Send + Sized {
    /// Parse this value using the given `Decoder`.
    async fn from_stream<D: Decoder>(decoder: &mut D) -> Result<Self, D::Error>;
}

/// Provides a `Visitor` access to each entry of a map in the input.
///
/// This is a trait that a `Decoder` passes to a `Visitor` implementation.
#[async_trait]
pub trait MapAccess: Send {
    /// Type to return in case of a decoding error.
    type Error: Error;

    /// This returns `Ok(Some(key))` for the next key in the map, or `Ok(None)`
    /// if there are no more remaining entries.
    async fn next_key<K: FromStream>(&mut self) -> Result<Option<K>, Self::Error>;

    /// This returns a `Ok(value)` for the next value in the map.
    ///
    /// # Panics
    ///
    /// Calling `next_value` before `next_key` is incorrect and is allowed to
    /// panic or return bogus results.
    async fn next_value<V: FromStream>(&mut self) -> Result<V, Self::Error>;

    /// This returns `Ok(Some((key, value)))` for the next (key-value) pair in
    /// the map, or `Ok(None)` if there are no more remaining items.
    ///
    /// This method exists as a convenience for `Deserialize` implementations.
    /// `MapAccess` implementations should not override the default behavior.
    async fn next_entry<K: FromStream, V: FromStream>(
        &mut self,
    ) -> Result<Option<(K, V)>, Self::Error>;

    /// Returns the number of entries remaining in the map, if known.
    #[inline]
    fn size_hint(&self) -> Option<usize> {
        None
    }
}

/// Provides a `Visitor` access to each element of a sequence in the input.
///
/// This is a trait that a `Decoder` passes to a `Visitor` implementation,
/// which deserializes each item in a sequence.
///
/// Based on [`serde::de::SeqAccess`].
#[async_trait]
pub trait SeqAccess: Send {
    /// The type to return if decoding encounters an error.
    type Error: Error;

    /// Returns `Ok(Some(value))` for the next value in the sequence,
    /// or `Ok(None)` if there are no more remaining items.
    async fn next_element<T: FromStream>(&mut self) -> Result<Option<T>, Self::Error>;

    /// Returns the number of elements remaining in the sequence, if known.
    #[inline]
    fn size_hint(&self) -> Option<usize> {
        None
    }
}

/// This trait describes a visitor responsible for decoding a stream.
///
/// Based on [`serde::de::Visitor`].
#[async_trait]
pub trait Visitor: Send + Sized {
    /// The type which this `Visitor` is responsible for decoding.
    type Value;

    /// Format a message stating what data this `Visitor` expects to receive.
    ///
    /// This is used in error messages. The message should complete the sentence
    /// "This Visitor expects to receive ...", for example the message could be
    /// "an integer between 0 and 64". The message should not be capitalized and
    /// should not end with a period.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result;

    /// The input contains a boolean.
    ///
    /// The default implementation fails with a type error.
    fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
        Err(Error::invalid_type(v, &self))
    }

    /// The input contains an `i8`.
    ///
    /// The default implementation forwards to [`visit_i64`].
    ///
    /// [`visit_i64`]: #method.visit_i64
    fn visit_i8<E: Error>(self, v: i8) -> Result<Self::Value, E> {
        self.visit_i64(v as i64)
    }

    /// The input contains an `i16`.
    ///
    /// The default implementation forwards to [`visit_i64`].
    ///
    /// [`visit_i64`]: #method.visit_i64
    fn visit_i16<E: Error>(self, v: i16) -> Result<Self::Value, E> {
        self.visit_i64(v as i64)
    }

    /// The input contains an `i32`.
    ///
    /// The default implementation forwards to [`visit_i64`].
    ///
    /// [`visit_i64`]: #method.visit_i64
    fn visit_i32<E: Error>(self, v: i32) -> Result<Self::Value, E> {
        self.visit_i64(v as i64)
    }

    /// The input contains an `i64`.
    ///
    /// The default implementation fails with a type error.
    fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
        Err(Error::invalid_type(v, &self))
    }

    /// The input contains a `u8`.
    ///
    /// The default implementation forwards to [`visit_u64`].
    ///
    /// [`visit_u64`]: #method.visit_u64
    fn visit_u8<E: Error>(self, v: u8) -> Result<Self::Value, E> {
        self.visit_u64(v as u64)
    }

    /// The input contains a `u16`.
    ///
    /// The default implementation forwards to [`visit_u64`].
    ///
    /// [`visit_u64`]: #method.visit_u64
    fn visit_u16<E: Error>(self, v: u16) -> Result<Self::Value, E> {
        self.visit_u64(v as u64)
    }

    /// The input contains a `u32`.
    ///
    /// The default implementation forwards to [`visit_u64`].
    ///
    /// [`visit_u64`]: #method.visit_u64
    fn visit_u32<E: Error>(self, v: u32) -> Result<Self::Value, E> {
        self.visit_u64(v as u64)
    }

    /// The input contains a `u64`.
    ///
    /// The default implementation fails with a type error.
    fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
        Err(Error::invalid_type(v, &self))
    }

    /// The input contains an `f32`.
    ///
    /// The default implementation forwards to [`visit_f64`].
    ///
    /// [`visit_f64`]: #method.visit_f64
    fn visit_f32<E: Error>(self, v: f32) -> Result<Self::Value, E> {
        self.visit_f64(v as f64)
    }

    /// The input contains an `f64`.
    ///
    /// The default implementation fails with a type error.
    fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
        Err(Error::invalid_type(v, &self))
    }

    /// The input contains a string and ownership of the string is being given
    /// to the `Visitor`.
    ///
    /// The default implementation fails with a type error.
    #[inline]
    fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
        Err(Error::invalid_type(v, &self))
    }

    /// The input contains a byte array and ownership of the byte array is being
    /// given to the `Visitor`.
    ///
    /// The default implementation fails with a type error.
    #[inline]
    fn visit_byte_buf<E: Error>(self, _v: Vec<u8>) -> Result<Self::Value, E> {
        Err(Error::invalid_type("(byte array)", &self))
    }

    /// The input contains an optional that is absent.
    /// The default implementation fails with a type error.
    #[inline]
    fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
        Err(Error::invalid_type("Option::None", &self))
    }

    /// The input contains an optional that is present.
    /// The default implementation fails with a type error.
    async fn visit_some<D: Decoder>(self, _decoder: &mut D) -> Result<Self::Value, D::Error> {
        Err(Error::invalid_type("Option::Some", &self))
    }

    /// The input contains a key-value map.
    /// The default implementation fails with a type error.
    async fn visit_map<A: MapAccess, E: Error>(self, _map: A) -> Result<Self::Value, E> {
        Err(Error::invalid_type("map", &self))
    }

    /// The input contains a sequence of elements.
    /// The default implementation fails with a type error.
    async fn visit_seq<A: SeqAccess, E: Error>(self, _seq: A) -> Result<Self::Value, E> {
        Err(Error::invalid_type("sequence", &self))
    }
}
