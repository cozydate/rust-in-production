// This is a single-file binary.

// Cargo.toml specifies the path to the package's files.
use package1;

fn main() {
    println!("{} {}",
             package1::get_t(),
             package1::mod1::get_u(),
    );
    // $ cargo run --bin binary_using_local_package
    // T U
}
