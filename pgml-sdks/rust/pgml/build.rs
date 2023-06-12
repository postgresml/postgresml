use std::fs::remove_file;

fn main() {
    // Remove python stub file that is auto generated each build
    remove_file("python/pgml/pgml.pyi").ok();
}
