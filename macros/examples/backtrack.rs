use raccoon_macros::backtrack;

struct Test {
    value: u32,
}

impl Test {
    #[backtrack(state = self.value)]
    fn save_even(&mut self, x: u32) -> Option<()> {
        // Eagerly set value
        self.value = x;

        if x % 2 == 0 {
            return Some(());
        }
        None
    }
}

fn main() {
    let mut t = Test { value: 2 };

    t.save_even(4);
    assert_eq!(t.value, 4);

    t.save_even(3);
    assert_eq!(t.value, 4);

    t.save_even(6);
    assert_eq!(t.value, 6);
}
