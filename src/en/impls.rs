use std::collections::*;
use std::fmt;
use std::hash::{BuildHasher, Hash};
use std::rc;
use std::sync;

use super::{Encoder, Error, ToStream};
use crate::EncodeTuple;

macro_rules! autoencode {
    ($ty:ident, $method:ident $($cast:tt)*) => {
        impl ToStream for $ty {
            fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
                encoder.$method(*self $($cast)*)
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

impl<'a> ToStream for &'a str {
    fn to_stream<E: Encoder>(
        &self,
        encoder: E,
    ) -> Result<<E as Encoder>::Ok, <E as Encoder>::Error> {
        encoder.encode_str(self)
    }
}

impl ToStream for String {
    fn to_stream<E: Encoder>(
        &self,
        encoder: E,
    ) -> Result<<E as Encoder>::Ok, <E as Encoder>::Error> {
        encoder.encode_str(self)
    }
}

impl<'a> ToStream for fmt::Arguments<'a> {
    fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.collect_str(self)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<T: ToStream> ToStream for Option<T> {
    fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
        match *self {
            Some(ref value) => encoder.encode_some(value),
            None => encoder.encode_none(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

// Does not require T: ToStream.
impl<T> ToStream for [T; 0] {
    fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
        let seq = encoder.encode_tuple(0)?;
        seq.end()
    }
}

macro_rules! encode_array {
    ($($len:tt)+) => {
        $(
            impl<T: ToStream> ToStream for [T; $len] {
                fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
                    let mut seq = encoder.encode_tuple($len)?;
                    for e in self {
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

impl<T: ToStream> ToStream for [T] {
    fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.collect_seq(self)
    }
}

macro_rules! encode_seq {
    ($ty:ident < T $(: $tbound1:ident $(+ $tbound2:ident)*)* $(, $typaram:ident : $bound:ident)* >) => {
        impl<T $(, $typaram)*> ToStream for $ty<T $(, $typaram)*>
        where
            T: ToStream $(+ $tbound1 $(+ $tbound2)*)*,
            $($typaram: $bound,)*
        {
            fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
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

impl ToStream for () {
    fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
        encoder.encode_unit()
    }
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! encode_tuple {
    ($($len:expr => ($($n:tt $name:ident)+))+) => {
        $(
            impl<$($name),+> ToStream for ($($name,)+)
            where
                $($name: ToStream,)+
            {
                fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
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
        impl<K, V $(, $typaram)*> ToStream for $ty<K, V $(, $typaram)*>
        where
            K: ToStream $(+ $kbound1 $(+ $kbound2)*)*,
            V: ToStream,
            $($typaram: $bound,)*
        {
            fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
                encoder.collect_map(self)
            }
        }
    }
}

encode_map!(BTreeMap<K: Ord, V>);
encode_map!(HashMap<K: Eq + Hash, V, H: BuildHasher>);

////////////////////////////////////////////////////////////////////////////////

macro_rules! encode_ref {
    (
        $(#[doc = $doc:tt])*
        <$($desc:tt)+
    ) => {
        $(#[doc = $doc])*
        impl <$($desc)+ {
            fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
                (**self).to_stream(encoder)
            }
        }
    };
}

encode_ref!(<'a, T: ?Sized> ToStream for &'a T where T: ToStream);
encode_ref!(<'a, T: ?Sized> ToStream for &'a mut T where T: ToStream);
encode_ref!(<T: ?Sized> ToStream for Box<T> where T: ToStream);
encode_ref!(<T: ?Sized> ToStream for rc::Rc<T> where T: ToStream);
encode_ref!(<T: ?Sized> ToStream for sync::Arc<T> where T: ToStream);

////////////////////////////////////////////////////////////////////////////////

impl<T: ToStream + Copy> ToStream for std::cell::Cell<T> {
    fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
        self.get().to_stream(encoder)
    }
}

impl<T: ToStream> ToStream for std::cell::RefCell<T> {
    fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
        match self.try_borrow() {
            Ok(value) => value.to_stream(encoder),
            Err(_) => Err(E::Error::custom("already mutably borrowed")),
        }
    }
}

impl<T: ToStream> ToStream for std::sync::Mutex<T> {
    fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
        match self.lock() {
            Ok(locked) => locked.to_stream(encoder),
            Err(_) => Err(E::Error::custom("lock poison error while serializing")),
        }
    }
}

impl<T: ToStream> ToStream for std::sync::RwLock<T> {
    fn to_stream<E: Encoder>(&self, encoder: E) -> Result<E::Ok, E::Error> {
        match self.read() {
            Ok(locked) => locked.to_stream(encoder),
            Err(_) => Err(E::Error::custom("lock poison error while serializing")),
        }
    }
}
