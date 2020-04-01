// This module is defined in a directory.

// Use but don't export.
mod internal;
// Use and export.
pub mod nested;

pub fn c() -> String { String::from("C") }

pub fn cde() -> String {
    // https://users.rust-lang.org/t/what-is-right-ways-to-concat-strings/3780/14
    [&c() as &str, &internal::d(), &nested::e()].concat()
}
