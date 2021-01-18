//! Generic data structure serialization framework based on [`serde::ser`].
//!
//! The two most important traits in this module are [`ToStream`] and [`Encoder`].
//!
//!  - **A type that implements `ToStream` is a data structure** that can be
//!    encoded to any stream format supported by `destream`, and conversely
//!  - **A type that implements `Encoder` is a data format** that can
//!    encode any supported data structure into a stream.
//!
//! # The ToStream trait
//!
//! `destream` provides [`ToStream`] implementations for many Rust primitive and
//! standard library types. The complete list is below. All of these can be
//! encoded automatically using `destream`.
//!
//! # Implementations of `ToStream` provided by `destream`
//!
//!  - **Primitive types**:
//!    - bool
//!    - i8, i16, i32, i64, i128, isize
//!    - u8, u16, u32, u64, u128, usize
//!    - f32, f64
//!    - str
//!    - &T and &mut T
//!  - **Compound types**:
//!    - \[T\]
//!    - \[T; 0\] through \[T; 32\]
//!    - tuples up to size 16
//!  - **Common standard library types**:
//!    - String
//!    - Option\<T\>
//!    - Result\<T, E\>
//!    - PhantomData\<T\>
//!  - **Wrapper types**:
//!    - Box\<T\>
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
//! # The IntoStream trait
//!
//! Often when encoding a stream, a value needs to be encoded which may outlive the calling function
//! context. For this reason, the `encode_map`, `encode_seq`, and `encode_stream` methods accept
//! a value which implements [`IntoStream`]. A borrow of a `ToStream` value automatically implements
//! `IntoStream`, so you can still call `encode_*` on a borrowed value, but with the advantage
//! that you can also encode an owned value into a stream of any lifetime by implementing
//! `IntoStream`.
//!
//! # Implementations of `IntoStream` provided by `destream`
//!
//!  - All `ToStream` types above, except \[T; 0\] through \[T; 32\]
//!  - &T and &mut T
//!  - MapStream<Item = Result<(K, V), E>>
//!  - SeqStream<Item = Result<T, E>>

use std::convert::Infallible;
use std::fmt;

use futures::Stream;

mod impls;

pub trait Error {
    fn custom<I: fmt::Display>(info: I) -> Self;
}

impl Error for Infallible {
    fn custom<I: fmt::Display>(_info: I) -> Self {
        panic!("Infallible operation failed!")
    }
}

/// Disambiguates a map from a sequence when encoding a stream.
pub struct MapStream<E, K, V, S: Stream<Item = Result<(K, V), E>>> {
    source: S,
}

impl<
        'en,
        E: fmt::Display + 'en,
        K: IntoStream<'en> + 'en,
        V: IntoStream<'en> + 'en,
        S: Stream<Item = Result<(K, V), E>> + 'en,
    > MapStream<E, K, V, S>
{
    fn into_inner(self) -> S {
        self.source
    }
}

impl<'en, E: 'en, K: 'en, V: 'en, S: Stream<Item = Result<(K, V), E>> + 'en> From<S>
    for MapStream<E, K, V, S>
{
    fn from(source: S) -> Self {
        Self { source }
    }
}

/// Returned from `Encoder::encode_map`.
pub trait EncodeMap<'en> {
    /// Must match the `Ok` type of the parent `Encoder`.
    type Ok: Stream + 'en;

    /// Must match the `Error` type of the parent `Encoder`.
    type Error: Error + 'en;

    /// Encode a map key.
    ///
    /// If possible, `ToStream` implementations are encouraged to use `encode_entry` instead as it
    /// may be implemented more efficiently in some formats compared to a pair of calls to
    /// `encode_key` and `encode_value`.
    fn encode_key<T: IntoStream<'en> + 'en>(&mut self, key: T) -> Result<(), Self::Error>;

    /// Encode a map value.
    ///
    /// # Panics
    ///
    /// Calling `encode_value` before `encode_key` is incorrect and is allowed to panic or produce
    /// bogus results.
    fn encode_value<T: IntoStream<'en> + 'en>(&mut self, value: T) -> Result<(), Self::Error>;

    /// Encode a map entry consisting of a key and a value.
    ///
    /// The default implementation delegates to [`encode_key`] and [`encode_value`].
    /// This is appropriate for encoders that do not care about performance or are not able to
    /// optimize `encode_entry` any further.
    ///
    /// [`ToStream`]: ../trait.ToStream.html
    /// [`encode_key`]: #tymethod.encode_key
    /// [`encode_value`]: #tymethod.encode_value
    fn encode_entry<K: IntoStream<'en> + 'en, V: IntoStream<'en> + 'en>(
        &mut self,
        key: K,
        value: V,
    ) -> Result<(), Self::Error> {
        self.encode_key(key)?;
        self.encode_value(value)
    }

    /// Finish encoding the map.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Disambiguates a sequence from a map when encoding a stream.
pub struct SeqStream<E, T, S: Stream<Item = Result<T, E>>> {
    source: S,
}

impl<
        'en,
        E: fmt::Display + 'en,
        T: IntoStream<'en> + 'en,
        S: Stream<Item = Result<T, E>> + 'en,
    > SeqStream<E, T, S>
{
    fn into_inner(self) -> S {
        self.source
    }
}

impl<'en, E: 'en, T: 'en, S: Stream<Item = Result<T, E>> + 'en> From<S> for SeqStream<E, T, S> {
    fn from(source: S) -> Self {
        Self { source }
    }
}

/// Returned from `Encoder::encode_seq`.
pub trait EncodeSeq<'en> {
    /// Must match the `Ok` type of the parent `Encoder`.
    type Ok: Stream + 'en;

    /// Must match the `Error` type of the parent `Encoder`.
    type Error: Error + 'en;

    /// Encode the next element in the sequence.
    fn encode_element<V: IntoStream<'en> + 'en>(&mut self, value: V) -> Result<(), Self::Error>;

    /// Finish encoding the sequence.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// Returned from `Encoder::encode_tuple`.
pub trait EncodeTuple<'en> {
    /// Must match the `Ok` type of the parent `Encoder`.
    type Ok: Stream + 'en;

    /// Must match the `Error` type of the parent `Encoder`.
    type Error: Error + 'en;

    /// Encode the next element in the tuple.
    fn encode_element<V: IntoStream<'en> + 'en>(&mut self, value: V) -> Result<(), Self::Error>;

    /// Finish encoding the tuple.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}

/// A data format that can encode and stream any data structure supported by destream.
///
/// Based on [`serde::ser::Serializer`].
pub trait Encoder<'en>: Sized {
    /// The output type produced by this `Encoder`.
    type Ok: Stream + 'en;

    /// The type returned when an encoding error is encountered.
    type Error: Error + 'en;

    /// Type returned from [`encode_map`] for streaming the content of the map.
    ///
    /// [`encode_map`]: #tymethod.encode_map
    type EncodeMap: EncodeMap<'en, Ok = Self::Ok, Error = Self::Error>;

    /// Type returned from [`encode_seq`] for streaming the content of the sequence.
    ///
    /// [`encode_seq`]: #tymethod.encode_seq
    type EncodeSeq: EncodeSeq<'en, Ok = Self::Ok, Error = Self::Error>;

    /// Type returned from [`encode_tuple`] for streaming the content of the tuple.
    ///
    /// [`encode_tuple`]: #tymethod.encode_tuple
    type EncodeTuple: EncodeTuple<'en, Ok = Self::Ok, Error = Self::Error>;

    /// Encode a `bool`.
    fn encode_bool(self, v: bool) -> Result<Self::Ok, Self::Error>;

    /// Encode an `i8`.
    fn encode_i8(self, v: i8) -> Result<Self::Ok, Self::Error>;

    /// Encode an `i16`.
    fn encode_i16(self, v: i16) -> Result<Self::Ok, Self::Error>;

    /// Encode an `i32`.
    fn encode_i32(self, v: i32) -> Result<Self::Ok, Self::Error>;

    /// Encode an `i64`.
    fn encode_i64(self, v: i64) -> Result<Self::Ok, Self::Error>;

    /// Encode a `u8`.
    fn encode_u8(self, v: u8) -> Result<Self::Ok, Self::Error>;

    /// Encode a `u16`.
    fn encode_u16(self, v: u16) -> Result<Self::Ok, Self::Error>;

    /// Encode a `u32`.
    fn encode_u32(self, v: u32) -> Result<Self::Ok, Self::Error>;

    /// Encode a `u64`.
    fn encode_u64(self, v: u64) -> Result<Self::Ok, Self::Error>;

    /// Encode an `f32` value.
    fn encode_f32(self, v: f32) -> Result<Self::Ok, Self::Error>;

    /// Encode an `f64` value.
    fn encode_f64(self, v: f64) -> Result<Self::Ok, Self::Error>;

    /// Encode a `&str`.
    fn encode_str(self, v: &str) -> Result<Self::Ok, Self::Error>;

    /// Encode a [`None`] value.
    ///
    /// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
    fn encode_none(self) -> Result<Self::Ok, Self::Error>;

    /// Encode a [`Some(T)`] value.
    ///
    /// [`Some(T)`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.Some
    fn encode_some<T: IntoStream<'en> + 'en>(self, value: T) -> Result<Self::Ok, Self::Error>;

    /// Encode a `()` value.
    fn encode_unit(self) -> Result<Self::Ok, Self::Error>;

    /// Begin encoding a map.
    /// This call must be followed by zero or more calls to `encode_key` and `encode_value`,
    /// then `end`.
    ///
    /// The argument is the number of elements in the map, which may or may not be computable before
    /// iterating over the map.
    fn encode_map(self, len: Option<usize>) -> Result<Self::EncodeMap, Self::Error>;

    /// Given a stream of encodable key-value pairs, return a stream encoded as a map.
    fn encode_map_stream<
        K: IntoStream<'en> + 'en,
        V: IntoStream<'en> + 'en,
        S: Stream<Item = Result<(K, V), Self::Error>> + Send + Unpin + 'en,
    >(
        self,
        map: S,
    ) -> Result<Self::Ok, Self::Error>;

    /// Begin encoding a variably sized sequence.
    /// This call must be followed by zero or more calls to `encode_element`, then `end`.
    ///
    /// The argument is the number of elements in the sequence, which may or may not be computable
    /// before iterating over the sequence.
    fn encode_seq(self, len: Option<usize>) -> Result<Self::EncodeSeq, Self::Error>;

    /// Given a stream of encodable values, return a stream encoded as a sequence.
    fn encode_seq_stream<
        T: IntoStream<'en> + 'en,
        S: Stream<Item = Result<T, Self::Error>> + Send + Unpin + 'en,
    >(
        self,
        seq: S,
    ) -> Result<Self::Ok, Self::Error>;

    /// Begin encoding a statically sized sequence whose length will be known at decoding time
    /// without looking at the encoded data.
    /// This call must be followed by zero or more calls to `encode_element`, then `end`.
    fn encode_tuple(self, len: usize) -> Result<Self::EncodeTuple, Self::Error>;

    /// Collect an iterator as a sequence.
    ///
    /// The default implementation encodes each item yielded by the iterator using [`encode_seq`].
    /// Implementors should not need to override this method.
    ///
    /// [`encode_seq`]: #tymethod.encode_seq
    fn collect_seq<T: IntoStream<'en> + 'en, I: IntoIterator<Item = T>>(
        self,
        iter: I,
    ) -> Result<Self::Ok, Self::Error> {
        let iter = iter.into_iter();
        let mut encoder = self.encode_seq(iterator_len_hint(&iter))?;
        for item in iter {
            encoder.encode_element(item)?;
        }

        encoder.end()
    }

    /// Collect an iterator as a map.
    ///
    /// The default implementation encodes each pair yielded by the iterator using [`encode_map`].
    /// Implementors should not need to override this method.
    ///
    /// [`encode_map`]: #tymethod.encode_map
    fn collect_map<K, V, I>(self, iter: I) -> Result<Self::Ok, Self::Error>
    where
        K: IntoStream<'en> + 'en,
        V: IntoStream<'en> + 'en,
        I: IntoIterator<Item = (K, V)>,
    {
        let iter = iter.into_iter();
        let mut encoder = self.encode_map(iterator_len_hint(&iter))?;
        for (key, value) in iter {
            encoder.encode_entry(key, value)?;
        }

        encoder.end()
    }

    /// Encode a string produced by an implementation of `Display`.
    fn collect_str<T: fmt::Display + ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        self.encode_str(&value.to_string())
    }
}

/// A data structure which can be borrowed to serialize into any supported stream encoding.
pub trait ToStream<'en> {
    /// Serialize this value into the given encoder.
    fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error>;
}

/// A data structure that can be serialized into any supported stream encoding.
///
/// This trait is automatically implemented for a borrow of `ToStream`, so prefer implementing
/// `ToStream` itself.
pub trait IntoStream<'en> {
    /// Take ownership of this value and serialize it into the given encoder.
    fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error>;
}

fn iterator_len_hint<I>(iter: &I) -> Option<usize>
where
    I: Iterator,
{
    match iter.size_hint() {
        (lo, Some(hi)) if lo == hi => Some(lo),
        _ => None,
    }
}
