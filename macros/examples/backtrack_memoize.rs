use std::num::NonZeroUsize;

use lru::LruCache;
use raccoon_macros::{backtrack, memoize};
use rand::random;

struct Test {
    cache: LruCache<[u8; 32], Option<u32>>,
    cursor: u32,
}

impl Test {
    #[memoize(cache = self.cache, key_extension = self.cursor)]
    #[backtrack(state = self.cursor)]
    fn check_even(&mut self, x: u32) -> Option<u32> {
        // Eagerly set cursor
        self.cursor += 1;

        if x % 2 == 0 {
            return Some(x + random::<u32>());
        }

        None
    }
}

fn main() {
    let mut t = Test {
        cache: LruCache::new(NonZeroUsize::new(10).unwrap()),
        cursor: 0,
    };

    let v = t.check_even(2);
    for _ in 0..5 {
        t.cursor = 0;
        assert_eq!(v, t.check_even(2));
    }

    // Advance the cursor
    t.cursor = 10;

    let v = t.check_even(3);
    for _ in 0..5 {
        assert_eq!(v, t.check_even(3));
        assert_eq!(t.cursor, 10);
    }
}
