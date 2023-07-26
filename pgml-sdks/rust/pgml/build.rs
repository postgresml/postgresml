use std::fs::remove_file;
use std::fs::OpenOptions;
use std::io::Write;

const ADDITIONAL_DEFAULTS_FOR_PYTHON: &[u8] = br#"
def setup_logger(log_level: str) -> None

Json = dict[str, Any]
DateTime = int
"#;

const ADDITIONAL_DEFAULTS_FOR_JAVASCRIPT: &[u8] = br#"
export function setupLogger(log_level: string): void;

export type Json = { [key: string]: any };
export type DateTime = Date;

export function newCollection(name: string, database_url?: string): Collection;
export function newModel(name?: string, task?: string, source?: string, parameters?: string, database_url?: string): Model;
export function newSplitter(name?: string, parameters?: any, database_url?: string): Splitter;
export function newBuiltins(database_url?: string): Builtins;
"#;

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
    file.write_all(ADDITIONAL_DEFAULTS_FOR_PYTHON).unwrap();

    // Remove typescript declaration file that is auto generated each build
    remove_file("./javascript/index.d.ts").ok();
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("./javascript/index.d.ts")
        .unwrap();
    // Add some manual declarations here
    file.write_all(ADDITIONAL_DEFAULTS_FOR_JAVASCRIPT).unwrap();
}
