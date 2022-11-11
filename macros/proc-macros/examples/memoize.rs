use std::num::NonZeroUsize;

use lru::LruCache;
// use raccoon_proc_macros::memoize;

struct T {
    _cache: LruCache<[u8; 32], u32>,
    cursor: u32,
}

impl T {
    // TODO(appcypher): Fix this!
    // #[memoize(cache = self.cache, key_extra = self.cursor)]
    fn identity(&mut self, x: u32) -> u32 {
        x
    }
}

fn main() {
    let mut t = T {
        _cache: LruCache::new(NonZeroUsize::new(5).unwrap()),
        cursor: 0,
    };

    let v = t.identity(1);
    println!("{}", v);

    let v = t.identity(1);
    println!("{}", v);

    t.cursor = 1;

    let v = t.identity(1);
    println!("{}", v);

    let v = t.identity(1);
    println!("{}", v);

    // For test write a MockCache that counts calls to get.
}
