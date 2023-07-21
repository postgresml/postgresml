use std::fs::remove_file;
use std::fs::OpenOptions;
use std::io::Write;

fn main() {
    // Remove python stub file that is auto generated each build
    remove_file("./python/pgml/pgml.pyi").ok();
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("./python/pgml/pgml.pyi")
        .unwrap();
    // Add our opening function declaration here
    file.write_all(
        b"\ndef setup_logger(log_level: str) -> None\n",
    )
    .unwrap();


    // Remove typescript declaration file that is auto generated each build
    remove_file("./javascript/index.d.ts").ok();
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("./javascript/index.d.ts")
        .unwrap();
    // Add our opening function declaration here
    file.write_all(
        b"\nexport function newDatabase(connection_string: string): Promise<Database>;\n",
    )
    .unwrap();
}
