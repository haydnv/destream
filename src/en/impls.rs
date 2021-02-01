use std::collections::*;
use std::fmt;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;

use bytes::Bytes;
use futures::{Stream, TryStreamExt};

use super::{EncodeTuple, Encoder, IntoStream, MapStream, SeqStream, ToStream};

macro_rules! autoencode {
    ($ty:ident, $method:ident $($cast:tt)*) => {
        impl<'en> ToStream<'en> for $ty {
            fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
                encoder.$method(*self $($cast)*)
            }
        }

        impl<'en> IntoStream<'en> for $ty {
            fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
                encoder.$method(self $($cast)*)
            }
        }
    }
}

autoencode!(bool, encode_bool);
autoencode!(isize, encode_i64 as i64);
autoencode!(i8, encode_i8);
autoencode!(i16, encode_i16);
autoencode!(i32, encode_i32);
autoencode!(i64, encode_i64);
autoencode!(usize, encode_u64 as u64);
autoencode!(u8, encode_u8);
autoencode!(u16, encode_u16);
autoencode!(u32, encode_u32);
autoencode!(u64, encode_u64);
autoencode!(f32, encode_f32);
autoencode!(f64, encode_f64);

////////////////////////////////////////////////////////////////////////////////

impl<'a, 'en> IntoStream<'en> for &'a str
where
    'a: 'en,
{
    fn into_stream<E: Encoder<'en>>(
        self,
        encoder: E,
    ) -> Result<<E as Encoder<'en>>::Ok, <E as Encoder<'en>>::Error> {
        encoder.encode_str(self)
    }
}

impl<'a, 'en> ToStream<'en> for &'a str
where
    'a: 'en,
{
    fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.encode_str(self)
    }
}

impl<'en> IntoStream<'en> for String {
    fn into_stream<E: Encoder<'en>>(
        self,
        encoder: E,
    ) -> Result<<E as Encoder<'en>>::Ok, <E as Encoder<'en>>::Error> {
        encoder.encode_str(&self)
    }
}

impl<'en> ToStream<'en> for String {
    fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.encode_str(self)
    }
}

impl<'a, 'en> IntoStream<'en> for fmt::Arguments<'a>
where
    'a: 'en,
{
    fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.collect_str(&self)
    }
}

impl<'a, 'en> ToStream<'en> for fmt::Arguments<'a>
where
    'a: 'en,
{
    fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.collect_str(self)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'en> ToStream<'en> for Bytes {
    fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.encode_bytes(self)
    }
}

impl<'en> IntoStream<'en> for Bytes {
    fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.encode_bytes(&self)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'en, T: IntoStream<'en> + 'en> IntoStream<'en> for Option<T> {
    fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
        match self {
            Some(value) => encoder.encode_some(value),
            None => encoder.encode_none(),
        }
    }
}

impl<'en, T: ToStream<'en> + 'en> ToStream<'en> for Option<T> {
    fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
        match *self {
            Some(ref value) => encoder.encode_some(value),
            None => encoder.encode_none(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'en, T: ?Sized> IntoStream<'en> for PhantomData<T> {
    fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.encode_unit()
    }
}

impl<'en, T: ?Sized> ToStream<'en> for PhantomData<T> {
    fn to_stream<E: Encoder<'en>>(&self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.encode_unit()
    }
}

////////////////////////////////////////////////////////////////////////////////

// Does not require T: IntoStream.
impl<'en, T> IntoStream<'en> for [T; 0] {
    fn into_stream<E: Encoder<'en>>(
        self,
        encoder: E,
    ) -> Result<<E as Encoder<'en>>::Ok, <E as Encoder<'en>>::Error> {
        let seq = encoder.encode_tuple(0)?;
        seq.end()
    }
}

// Does not require T: ToStream.
impl<'en, T> ToStream<'en> for [T; 0] {
    fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
        let seq = encoder.encode_tuple(0)?;
        seq.end()
    }
}

macro_rules! encode_array {
    ($($len:tt)+) => {
        $(
            impl<'a, 'en, T: ToStream<'en> + 'en> IntoStream<'en> for &'a [T; $len] where 'a: 'en {
                fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
                    let mut seq = encoder.encode_tuple($len)?;
                    for e in self {
                        seq.encode_element(e)?;
                    }
                    seq.end()
                }
            }

            impl<'a, 'en, T: ToStream<'en> + 'en> ToStream<'en> for &'a [T; $len] where 'a: 'en {
                fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
                    let mut seq = encoder.encode_tuple($len)?;
                    for e in *self {
                        seq.encode_element(e)?;
                    }
                    seq.end()
                }
            }
        )+
    }
}

encode_array! {
    01 02 03 04 05 06 07 08 09 10
    11 12 13 14 15 16 17 18 19 20
    21 22 23 24 25 26 27 28 29 30
    31 32
}

////////////////////////////////////////////////////////////////////////////////

impl<'a, 'en, T: ToStream<'en> + 'en> IntoStream<'en> for &'a [T]
where
    'a: 'en,
{
    fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.collect_seq(self)
    }
}

impl<'a, 'en, T: ToStream<'en>> ToStream<'en> for &'a [T]
where
    'a: 'en,
{
    fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.collect_seq(*self)
    }
}

macro_rules! encode_seq {
    ($ty:ident < T $(: $tbound1:ident $(+ $tbound2:ident)*)* $(, $typaram:ident : $bound:ident)* >) => {
        impl<'en, T $(, $typaram)*> IntoStream<'en> for $ty<T $(, $typaram)*>
        where
            T: IntoStream<'en> + 'en $(+ $tbound1 $(+ $tbound2)*)*,
            $($typaram: $bound,)*
        {
            fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
                encoder.collect_seq(self)
            }
        }

        impl<'en, T $(, $typaram)*> ToStream<'en> for $ty<T $(, $typaram)*>
        where
            T: ToStream<'en> + 'en $(+ $tbound1 $(+ $tbound2)*)*,
            $($typaram: $bound,)*
        {
            fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
                encoder.collect_seq(self)
            }
        }
    }
}

encode_seq!(BinaryHeap<T: Ord>);
encode_seq!(BTreeSet<T: Ord>);
encode_seq!(HashSet<T: Eq + Hash, H: BuildHasher>);
encode_seq!(LinkedList<T>);
encode_seq!(Vec<T>);
encode_seq!(VecDeque<T>);

////////////////////////////////////////////////////////////////////////////////

macro_rules! encode_tuple {
    ($($len:expr => ($($n:tt $name:ident)+))+) => {
        $(
            impl<'en, $($name),+> IntoStream<'en> for ($($name,)+)
            where
                $($name: IntoStream<'en> + 'en,)+
            {
                fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
                    let mut tuple = encoder.encode_tuple($len)?;
                    $(
                        tuple.encode_element(self.$n)?;
                    )+
                    tuple.end()
                }
            }

            impl<'en, $($name),+> ToStream<'en> for ($($name,)+)
            where
                $($name: ToStream<'en> + 'en,)+
            {
                fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
                    let mut tuple = encoder.encode_tuple($len)?;
                    $(
                        tuple.encode_element(&self.$n)?;
                    )+
                    tuple.end()
                }
            }
        )+
    }
}

encode_tuple! {
    1 => (0 T0)
    2 => (0 T0 1 T1)
    3 => (0 T0 1 T1 2 T2)
    4 => (0 T0 1 T1 2 T2 3 T3)
    5 => (0 T0 1 T1 2 T2 3 T3 4 T4)
    6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
    7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
    8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
    9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
    10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
    11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
    12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
    13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
    14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
    15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
    16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! encode_map {
    ($ty:ident < K $(: $kbound1:ident $(+ $kbound2:ident)*)*, V $(, $typaram:ident : $bound:ident)* >) => {
        impl<'en, K, V $(, $typaram)*> IntoStream<'en> for $ty<K, V $(, $typaram)*>
        where
            K: IntoStream<'en> + 'en $(+ $kbound1 $(+ $kbound2)*)*,
            V: IntoStream<'en> + 'en,
            $($typaram: $bound,)*
        {
            fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
                encoder.collect_map(self)
            }
        }

        impl<'en, K, V $(, $typaram)*> ToStream<'en> for $ty<K, V $(, $typaram)*>
        where
            K: ToStream<'en> + 'en $(+ $kbound1 $(+ $kbound2)*)*,
            V: ToStream<'en> + 'en,
            $($typaram: $bound,)*
        {
            fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
                encoder.collect_map(self)
            }
        }
    }
}

encode_map!(BTreeMap<K: Ord, V>);
encode_map!(HashMap<K: Eq + Hash, V, H: BuildHasher>);

////////////////////////////////////////////////////////////////////////////////

impl<'en, T: IntoStream<'en> + 'en> IntoStream<'en> for Box<T> {
    fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
        (*self).into_stream(encoder)
    }
}

impl<'en, T: ToStream<'en> + 'en> ToStream<'en> for Box<T> {
    fn to_stream<E: Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
        (&**self).to_stream(encoder)
    }
}

macro_rules! encode_ref {
    (
        $(#[doc = $doc:tt])*
        <$($desc:tt)+
    ) => {
        $(#[doc = $doc])*
        impl <$($desc)+ {
            fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
                (*self).to_stream(encoder)
            }
        }
    };
}

encode_ref!(<'a, 'en, T: ?Sized> IntoStream<'en> for &'a T where T: ToStream<'en> + 'en, 'a: 'en);
encode_ref!(<'a, 'en, T: ?Sized> IntoStream<'en> for &'a mut T where T: ToStream<'en> + 'en, 'a: 'en);

////////////////////////////////////////////////////////////////////////////////

impl<
        'en,
        Err: fmt::Display + 'en,
        K: IntoStream<'en> + 'en,
        V: IntoStream<'en> + 'en,
        S: Stream<Item = Result<(K, V), Err>> + Send + Unpin + 'en,
    > IntoStream<'en> for MapStream<Err, K, V, S>
{
    fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.encode_map_stream(self.into_inner().map_err(super::Error::custom))
    }
}

impl<
        'en,
        Err: fmt::Display + 'en,
        T: IntoStream<'en> + 'en,
        S: Stream<Item = Result<T, Err>> + Send + Unpin + 'en,
    > IntoStream<'en> for SeqStream<Err, T, S>
{
    fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.encode_seq_stream(self.into_inner().map_err(super::Error::custom))
    }
}
