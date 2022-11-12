use std::num::NonZeroUsize;

use lru::LruCache;
use raccoon_macros::memoize;
use rand::random;

struct Test {
    cache: LruCache<[u8; 32], u32>,
    cursor: u32,
}

impl Test {
    #[memoize(cache = self.cache, key_extension = self.cursor)]
    fn get_random(&mut self, x: u32) -> u32 {
        x + random::<u32>()
    }
}

fn main() {
    let mut t = Test {
        cache: LruCache::new(NonZeroUsize::new(5).unwrap()),
        cursor: 0,
    };

    let v = t.get_random(1);
    for _ in 0..5 {
        assert_eq!(v, t.get_random(1));
    }

    // Update the cursor
    t.cursor = 1;

    let v = t.get_random(1);
    for _ in 0..5 {
        assert_eq!(v, t.get_random(1));
    }
}
