// This is a single-file binary.

// Each binary under src/bin/ is its own crate.
// To use src/* modules, import the default library crate.  It has the same name as the package.
use modules_and_binaries::{module1, module2};

fn main() {
    println!("{} {} {} {}",
             module1::a(),
             module2::c(),
             module2::cde(),
             module2::nested::e(),
    );
    // $ cargo run --bin binary_using_shared_modules
    // AB C CDE E

    // $ cargo build --release && target/release/binary_using_shared_modules
    // AB C CDE E
}
