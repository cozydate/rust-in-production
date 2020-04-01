// This is a binary with its own internal modules, each in its own file.

mod internal;

fn main() {
    println!("{}", internal::f(),
    );
    // $ cargo run --bin binary_with_internal_modules
    // F
}
