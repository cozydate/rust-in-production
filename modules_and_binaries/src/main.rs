mod module1;
mod module2;

fn main() {
    println!("{} {} {} {}",
             module1::a(), module2::c(), module2::cde(), module2::nested::e());
    // $ cargo run
    // AB C CDE E

    // Cargo builds src/main.rs into a binary with the same name as the package.
    // $ cargo build --release
    // $ target/release/modules_and_binaries
    // AB C CDE E
}
