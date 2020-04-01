// This is a single-file binary.

// Each binary under src/bin/ is its own crate.
// To use src/* modules, import the default library crate.  It has the same name as the package.
use modules_and_binaries::{multi_file_module, single_file_module};

fn main() {
    println!("{} {} {} {}",
             single_file_module::a(),
             multi_file_module::c(),
             multi_file_module::cde(),
             multi_file_module::nested::e(),
    );
    // $ cargo run --bin binary_using_shared_modules
    // AB C CDE E

    // $ cargo build --release && target/release/binary_using_shared_modules
    // AB C CDE E
}
