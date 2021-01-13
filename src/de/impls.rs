use std::collections::*;
use std::fmt;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;

use async_trait::async_trait;
use futures::future::TryFutureExt;

use super::size_hint;
use super::{Decoder, Error, FromStream, SeqAccess, Visitor};

macro_rules! autodecode {
    ($ty:ident, $visit_method:ident, $decode_method:ident) => {
        #[async_trait]
        impl FromStream for $ty {
            async fn from_stream<D: Decoder>(decoder: &mut D) -> Result<Self, D::Error> {
                struct AutoVisitor;

                impl Visitor for AutoVisitor {
                    type Value = $ty;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(stringify!($ty))
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

////////////////////////////////////////////////////////////////////////////////

struct OptionVisitor<T> {
    marker: PhantomData<T>,
}

#[async_trait]
impl<T: FromStream> Visitor for OptionVisitor<T> {
    type Value = Option<T>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "optional {}", std::any::type_name::<T>())
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
        T::from_stream(decoder).map_ok(Some).await
    }
}

#[async_trait]
impl<T: FromStream> FromStream for Option<T> {
    async fn from_stream<D: Decoder>(decoder: &mut D) -> Result<Self, D::Error> {
        let visitor = OptionVisitor {
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

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("unit")
    }

    #[inline]
    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        Ok(PhantomData)
    }
}

#[async_trait]
impl<T: Send + ?Sized> FromStream for PhantomData<T> {
    async fn from_stream<D: Decoder>(decoder: &mut D) -> Result<Self, D::Error> {
        let visitor = PhantomDataVisitor {
            marker: PhantomData,
        };

        decoder.decode_unit_struct("PhantomData", visitor).await
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
        {
            async fn from_stream<D: Decoder>(decoder: &mut D) -> Result<Self, D::Error> {
                struct SeqVisitor<T $(, $typaram)*> {
                    marker: PhantomData<$ty<T $(, $typaram)*>>,
                }

                #[async_trait]
                impl<T $(, $typaram)*> Visitor for SeqVisitor<T $(, $typaram)*>
                where
                    T: FromStream $(+ $tbound1 $(+ $tbound2)*)*,
                    $($typaram: $bound1 $(+ $bound2)*,)*
                {
                    type Value = $ty<T $(, $typaram)*>;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a sequence")
                    }

                    async fn visit_seq<A: SeqAccess>(self, mut $access: A) -> Result<Self::Value, A::Error> {
                        let mut values = $with_capacity;

                        while let Some(value) = $access.next_element().await? {
                            $insert(&mut values, value);
                        }

                        Ok(values)
                    }
                }

                let visitor = SeqVisitor { marker: PhantomData };
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
