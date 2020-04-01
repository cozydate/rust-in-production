// Each binary under src/bin/ is its own crate.
// To use src/* modules, import the default library crate.  It has the same name as the package.
extern crate modules_and_binaries;

use modules_and_binaries::{module1, module2};

mod internal;

fn main() {
    println!("{} {} {} {} {}",
             module1::a(),
             module2::c(),
             module2::cde(),
             module2::nested::e(),
             internal::f(),
    );

    // $ cargo run --bin binary2
    // AB C CDE E F

    // $ cargo build --release && target/release/binary2
    // AB C CDE E F
}
