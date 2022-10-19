use raccoon_macros::memoize;

#[memoize(self: cache)]
fn to_string(value: usize) -> String {
    value.to_string()
}

fn main() {
    let a = to_string(45);
    let b = to_string(45);

    assert_eq!(a, b);
}
