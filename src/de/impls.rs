use std::collections::*;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;

use async_trait::async_trait;
use bytes::Bytes;
use futures::future::TryFutureExt;

use super::size_hint;
use super::{Decoder, Error, FromStream, MapAccess, SeqAccess, Visitor};
use std::convert::TryInto;

macro_rules! autodecode {
    ($ty:ident, $visit_method:ident, $decode_method:ident) => {
        #[async_trait]
        impl FromStream for $ty {
            type Context = ();

            async fn from_stream<D: Decoder>(
                _context: Self::Context,
                decoder: &mut D,
            ) -> Result<Self, D::Error> {
                struct AutoVisitor;

                impl Visitor for AutoVisitor {
                    type Value = $ty;

                    fn expecting() -> &'static str {
                        stringify!($ty)
                    }

                    #[inline]
                    fn $visit_method<E: Error>(self, v: $ty) -> Result<Self::Value, E> {
                        Ok(v)
                    }
                }

                decoder.$decode_method(AutoVisitor).await
            }
        }
    };
}

autodecode!(bool, visit_bool, decode_bool);
autodecode!(i8, visit_i8, decode_i8);
autodecode!(i16, visit_i16, decode_i16);
autodecode!(i32, visit_i32, decode_i32);
autodecode!(i64, visit_i64, decode_i64);
autodecode!(u8, visit_u8, decode_u8);
autodecode!(u16, visit_u16, decode_u16);
autodecode!(u32, visit_u32, decode_u32);
autodecode!(u64, visit_u64, decode_u64);
autodecode!(f32, visit_f32, decode_f32);
autodecode!(f64, visit_f64, decode_f64);
autodecode!(String, visit_string, decode_string);

#[async_trait]
impl FromStream for isize {
    type Context = ();

    async fn from_stream<D: Decoder>(cxt: (), decoder: &mut D) -> Result<Self, D::Error> {
        let n: i64 = FromStream::from_stream(cxt, decoder).await?;
        n.try_into().map_err(Error::custom)
    }
}

#[async_trait]
impl FromStream for usize {
    type Context = ();

    async fn from_stream<D: Decoder>(cxt: (), decoder: &mut D) -> Result<Self, D::Error> {
        let n: u64 = FromStream::from_stream(cxt, decoder).await?;
        n.try_into().map_err(Error::custom)
    }
}

////////////////////////////////////////////////////////////////////////////////

struct BytesVisitor;

impl Visitor for BytesVisitor {
    type Value = Bytes;

    fn expecting() -> &'static str {
        "a byte buffer"
    }

    fn visit_byte_buf<E: Error>(self, buf: Vec<u8>) -> Result<Self::Value, E> {
        Ok(Bytes::from(buf))
    }
}

#[async_trait]
impl FromStream for Bytes {
    type Context = ();

    async fn from_stream<D: Decoder>(_: (), decoder: &mut D) -> Result<Self, D::Error> {
        decoder.decode_byte_buf(BytesVisitor).await
    }
}

////////////////////////////////////////////////////////////////////////////////

struct OptionVisitor<T: FromStream> {
    context: T::Context,
    marker: PhantomData<T>,
}

#[async_trait]
impl<T: FromStream> Visitor for OptionVisitor<T> {
    type Value = Option<T>;

    fn expecting() -> &'static str {
        stringify!("optional {}", std::any::type_name::<T>())
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(None)
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(None)
    }

    async fn visit_some<D: Decoder>(self, decoder: &mut D) -> Result<Self::Value, D::Error> {
        T::from_stream(self.context, decoder).map_ok(Some).await
    }
}

#[async_trait]
impl<T: FromStream> FromStream for Option<T> {
    type Context = T::Context;

    async fn from_stream<D: Decoder>(
        context: Self::Context,
        decoder: &mut D,
    ) -> Result<Self, D::Error> {
        let visitor = OptionVisitor {
            context,
            marker: PhantomData,
        };

        decoder.decode_option(visitor).await
    }
}

////////////////////////////////////////////////////////////////////////////////

struct PhantomDataVisitor<T: ?Sized> {
    marker: PhantomData<T>,
}

impl<T: Send + ?Sized> Visitor for PhantomDataVisitor<T> {
    type Value = PhantomData<T>;

    fn expecting() -> &'static str {
        "unit"
    }

    #[inline]
    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        Ok(PhantomData)
    }
}

#[async_trait]
impl<T: Send + ?Sized> FromStream for PhantomData<T> {
    type Context = ();

    async fn from_stream<D: Decoder>(_context: (), decoder: &mut D) -> Result<Self, D::Error> {
        let visitor = PhantomDataVisitor {
            marker: PhantomData,
        };

        decoder.decode_unit(visitor).await
    }
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! decode_seq {
    (
        $ty:ident < T $(: $tbound1:ident $(+ $tbound2:ident)*)* $(, $typaram:ident : $bound1:ident $(+ $bound2:ident)*)* >,
        $access:ident,
        $clear:expr,
        $with_capacity:expr,
        $reserve:expr,
        $insert:expr
    ) => {
        #[async_trait]
        impl<T $(, $typaram)*> FromStream for $ty<T $(, $typaram)*>
        where
            T: FromStream $(+ $tbound1 $(+ $tbound2)*)*,
            $($typaram: $bound1 $(+ $bound2)*,)*
            T::Context: Copy
        {
            type Context = T::Context;

            async fn from_stream<D: Decoder>(context: Self::Context, decoder: &mut D) -> Result<Self, D::Error> {
                struct SeqVisitor<C, T $(, $typaram)*> {
                    context: C,
                    marker: PhantomData<$ty<T $(, $typaram)*>>,
                }

                #[async_trait]
                impl<T $(, $typaram)*> Visitor for SeqVisitor<T::Context, T $(, $typaram)*>
                where
                    T: FromStream $(+ $tbound1 $(+ $tbound2)*)*,
                    $($typaram: $bound1 $(+ $bound2)*,)*
                    T::Context: Copy
                {
                    type Value = $ty<T $(, $typaram)*>;

                    fn expecting() -> &'static str {
                        "a sequence"
                    }

                    async fn visit_seq<A: SeqAccess>(self, mut $access: A) -> Result<Self::Value, A::Error> {
                        let mut values = $with_capacity;

                        while let Some(value) = $access.next_element(self.context).await? {
                            $insert(&mut values, value);
                        }

                        Ok(values)
                    }
                }

                let visitor = SeqVisitor { context, marker: PhantomData };
                decoder.decode_seq(visitor).await
            }
        }
    }
}

decode_seq!(
    BinaryHeap<T: Ord>,
    seq,
    BinaryHeap::clear,
    BinaryHeap::with_capacity(size_hint::cautious(seq.size_hint())),
    BinaryHeap::reserve,
    BinaryHeap::push
);

decode_seq!(
    BTreeSet<T: Eq + Ord>,
    seq,
    BTreeSet::clear,
    BTreeSet::new(),
    nop_reserve,
    BTreeSet::insert
);

decode_seq!(
    LinkedList<T>,
    seq,
    LinkedList::clear,
    LinkedList::new(),
    nop_reserve,
    LinkedList::push_back
);

decode_seq!(
    HashSet<T: Eq + Hash, S: BuildHasher + Default + Send>,
    seq,
    HashSet::clear,
    HashSet::with_capacity_and_hasher(size_hint::cautious(seq.size_hint()), S::default()),
    HashSet::reserve,
    HashSet::insert
);

decode_seq!(
    VecDeque<T>,
    seq,
    VecDeque::clear,
    VecDeque::with_capacity(size_hint::cautious(seq.size_hint())),
    VecDeque::reserve,
    VecDeque::push_back
);

decode_seq!(
    Vec<T>,
    seq,
    Vec::clear,
    Vec::with_capacity(size_hint::cautious(seq.size_hint())),
    Vec::reserve,
    Vec::push
);

////////////////////////////////////////////////////////////////////////////////

struct ArrayVisitor<C, T> {
    context: C,
    marker: PhantomData<T>,
}

impl<C, T> ArrayVisitor<C, T> {
    fn new(context: C) -> Self {
        ArrayVisitor {
            context,
            marker: PhantomData,
        }
    }
}

#[async_trait]
impl<T: FromStream> Visitor for ArrayVisitor<T::Context, [T; 0]> {
    type Value = [T; 0];

    fn expecting() -> &'static str {
        "a zero-length tuple"
    }

    async fn visit_seq<A: SeqAccess>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let next: Option<T> = seq.next_element(self.context).await?;
        match next {
            None => Ok([]),
            Some(_) => Err(Error::invalid_length(0, Self::expecting())),
        }
    }
}

#[async_trait]
impl<T: FromStream> FromStream for [T; 0] {
    type Context = T::Context;

    async fn from_stream<D: Decoder>(
        context: T::Context,
        decoder: &mut D,
    ) -> Result<Self, <D as Decoder>::Error> {
        decoder
            .decode_tuple(0, ArrayVisitor::<T::Context, [T; 0]>::new(context))
            .await
    }
}

macro_rules! decode_array {
    ($($len:expr => ($($n:tt)+))+) => {
        $(
            #[async_trait]
            impl<T: FromStream> Visitor for ArrayVisitor<T::Context, [T; $len]>
            where T::Context: Copy
            {
                type Value = [T; $len];

                fn expecting() -> &'static str {
                    concat!("an array of length ", $len)
                }

                async fn visit_seq<A: SeqAccess>(
                    self,
                    mut seq: A
                ) -> Result<Self::Value, A::Error> {
                    Ok([$(
                        match seq.next_element(self.context).await? {
                            Some(val) => val,
                            None => return Err(Error::invalid_length($n, Self::expecting())),
                        }
                    ),+])
                }
            }

            #[async_trait]
            impl<T: FromStream> FromStream for [T; $len] where T::Context: Copy {
                type Context = T::Context;

                async fn from_stream<D: Decoder>(
                    context: T::Context,
                    decoder: &mut D
                ) -> Result<Self, D::Error> {
                    decoder.decode_tuple(
                        $len,
                        ArrayVisitor::<T::Context, [T; $len]>::new(context)).await
                }
            }
        )+
    }
}

decode_array! {
    1 => (0)
    2 => (0 1)
    3 => (0 1 2)
    4 => (0 1 2 3)
    5 => (0 1 2 3 4)
    6 => (0 1 2 3 4 5)
    7 => (0 1 2 3 4 5 6)
    8 => (0 1 2 3 4 5 6 7)
    9 => (0 1 2 3 4 5 6 7 8)
    10 => (0 1 2 3 4 5 6 7 8 9)
    11 => (0 1 2 3 4 5 6 7 8 9 10)
    12 => (0 1 2 3 4 5 6 7 8 9 10 11)
    13 => (0 1 2 3 4 5 6 7 8 9 10 11 12)
    14 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13)
    15 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14)
    16 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15)
    17 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16)
    18 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17)
    19 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18)
    20 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19)
    21 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20)
    22 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21)
    23 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22)
    24 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23)
    25 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24)
    26 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25)
    27 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26)
    28 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27)
    29 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28)
    30 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29)
    31 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30)
    32 => (0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31)
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! decode_tuple {
    ($($len:tt => ($($n:tt $name:ident)+))+) => {
        $(
            #[async_trait]
            impl<$($name: FromStream<Context = ()>),+> FromStream for ($($name,)+) {
                type Context = ();

                async fn from_stream<D: Decoder>(_context: (), decoder: &mut D) -> Result<Self, D::Error> {
                    struct TupleVisitor<$($name,)+> {
                        marker: PhantomData<($($name,)+)>,
                    }

                    #[async_trait]
                    #[allow(non_snake_case)]
                    impl<$($name: FromStream<Context = ()>),+> Visitor for TupleVisitor<$($name,)+> {
                        type Value = ($($name,)+);

                        fn expecting() -> &'static str {
                            concat!("a tuple of size ", $len)
                        }

                        async fn visit_seq<A: SeqAccess>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                            $(
                                let $name = match seq.next_element(()).await? {
                                    Some(value) => value,
                                    None => return Err(Error::invalid_length($n, Self::expecting())),
                                };
                            )+

                            Ok(($($name,)+))
                        }
                    }

                    decoder.decode_tuple($len, TupleVisitor { marker: PhantomData }).await
                }
            }
        )+
    }
}

decode_tuple! {
    1  => (0 T0)
    2  => (0 T0 1 T1)
    3  => (0 T0 1 T1 2 T2)
    4  => (0 T0 1 T1 2 T2 3 T3)
    5  => (0 T0 1 T1 2 T2 3 T3 4 T4)
    6  => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
    7  => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
    8  => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
    9  => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
    10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
    11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
    12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
    13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
    14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
    15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
    16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! decode_map {
    (
        $ty:ident < K $(: $kbound1:ident $(+ $kbound2:ident)*)*, V $(, $typaram:ident : $bound1:ident $(+ $bound2:ident)*)* >,
        $access:ident,
        $with_capacity:expr
    ) => {
        #[async_trait]
        impl<K, V $(, $typaram)*> FromStream for $ty<K, V $(, $typaram)*>
        where
            K: FromStream<Context = ()> $(+ $kbound1 $(+ $kbound2)*)*,
            V: FromStream<Context = ()>,
            $($typaram: $bound1 $(+ $bound2)*),*
        {
            type Context = ();

            async fn from_stream<D: Decoder>(
                _context: (),
                decoder: &mut D
            ) -> Result<Self, D::Error> {
                struct MapVisitor<K, V $(, $typaram)*> {
                    marker: PhantomData<$ty<K, V $(, $typaram)*>>,
                }

                #[async_trait]
                impl<K, V $(, $typaram)*> Visitor for MapVisitor<K, V $(, $typaram)*>
                where
                    K: FromStream<Context = ()> $(+ $kbound1 $(+ $kbound2)*)*,
                    V: FromStream<Context = ()>,
                    $($typaram: $bound1 $(+ $bound2)*),*
                {
                    type Value = $ty<K, V $(, $typaram)*>;

                    fn expecting() -> &'static str {
                        "a map"
                    }

                    async fn visit_map<A: MapAccess>(
                        self,
                        mut $access: A
                    ) -> Result<Self::Value, A::Error> {
                        let mut values = $with_capacity;

                        while let Some(key) = $access.next_key(()).await? {
                            let value = $access.next_value(()).await?;
                            values.insert(key, value);
                        }

                        Ok(values)
                    }
                }

                let visitor = MapVisitor { marker: PhantomData };
                decoder.decode_map(visitor).await
            }
        }
    }
}

decode_map!(
    BTreeMap<K: Ord, V>,
    map,
    BTreeMap::new()
);

decode_map!(
    HashMap<K: Eq + Hash, V, S: BuildHasher + Default + Send>,
    map,
    HashMap::with_capacity_and_hasher(size_hint::cautious(map.size_hint()), S::default())
);

////////////////////////////////////////////////////////////////////////////////

struct UnitVisitor;

impl Visitor for UnitVisitor {
    type Value = ();

    fn expecting() -> &'static str {
        "a unit value ()"
    }

    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
        Ok(())
    }
}

#[async_trait]
impl FromStream for () {
    type Context = ();

    async fn from_stream<D: Decoder>(_context: (), decoder: &mut D) -> Result<Self, D::Error> {
        decoder.decode_unit(UnitVisitor).await
    }
}
