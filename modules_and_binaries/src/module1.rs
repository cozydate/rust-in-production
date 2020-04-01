// This is a module defined in a single file.

// Not exported.
fn fn_b() -> String { String::from("B") }

// Exported.
pub fn a() -> String {
    ["A", &fn_b()].concat()
}
