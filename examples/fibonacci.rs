use std::mem;

use destream::en::{Encoder, IntoStream, SeqStream};
use futures::stream;
use num_bigint::BigUint;
use num_traits::identities::One;

struct Fibonacci {
    one_ago: BigUint,
    two_ago: BigUint,
}

impl Default for Fibonacci {
    fn default() -> Self {
        Self {
            one_ago: BigUint::one(),
            two_ago: BigUint::one(),
        }
    }
}

impl Iterator for Fibonacci {
    type Item = BigUint;

    fn next(&mut self) -> Option<Self::Item> {
        let next = &self.one_ago + &self.two_ago;
        mem::swap(&mut self.one_ago, &mut self.two_ago);
        self.one_ago = next.clone();
        Some(next)
    }
}

impl<'en> IntoStream<'en> for Fibonacci {
    fn into_stream<E: Encoder<'en>>(self, encoder: E) -> Result<E::Ok, E::Error> {
        let iter = self.into_iter().map(|i| i.to_u64_digits());
        SeqStream::from(stream::iter(iter)).into_stream(encoder)
    }
}

fn main() {
    // the destream crate only provides a common set of traits to implement streaming codecs
    // for example implementations, see:
    //   - destream_json: https://docs.rs/destream_json
    //   - tbon: https://docs.rs/tbon
}
