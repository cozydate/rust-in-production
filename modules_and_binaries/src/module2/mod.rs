// This module is defined in a directory.

// Use the `internal` module but don't export it.
mod internal;
// Use and export the `nested` module.
pub mod nested;

pub fn c() -> String { String::from("C") }

pub fn cde() -> String {
    // https://users.rust-lang.org/t/what-is-right-ways-to-concat-strings/3780/14
    [&c() as &str, &internal::d(), &nested::e()].concat()
}
