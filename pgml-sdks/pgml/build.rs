use std::fs::remove_file;
use std::fs::OpenOptions;
use std::io::Write;

const ADDITIONAL_DEFAULTS_FOR_PYTHON: &[u8] = br#"
def init_logger(level: Optional[str] = "", format: Optional[str] = "") -> None
async def migrate() -> None

Json = Any
DateTime = int
"#;

const ADDITIONAL_DEFAULTS_FOR_JAVASCRIPT: &[u8] = br#"
export function init_logger(level?: string, format?: string): void;
export function migrate(): Promise<void>;

export type Json = { [key: string]: any };
export type DateTime = Date;

export function newCollection(name: string, database_url?: string): Collection;
export function newModel(name?: string, source?: string, parameters?: Json): Model;
export function newSplitter(name?: string, parameters?: Json): Splitter;
export function newBuiltins(database_url?: string): Builtins;
export function newPipeline(name: string, model?: Model, splitter?: Splitter, parameters?: Json): Pipeline;
"#;

fn main() {
    // Remove python stub file that is auto generated each build
    let path = std::env::var("PYTHON_STUB_FILE");
    if let Ok(path) = path {
        remove_file(&path).ok();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)
            .unwrap();
        // Add our opening function declaration here
        file.write_all(ADDITIONAL_DEFAULTS_FOR_PYTHON).unwrap();
    }

    let path = std::env::var("TYPESCRIPT_DECLARATION_FILE");
    if let Ok(path) = path {
        // Remove typescript declaration file that is auto generated each build
        remove_file(&path).ok();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)
            .unwrap();
        // Add some manual declarations here
        file.write_all(ADDITIONAL_DEFAULTS_FOR_JAVASCRIPT).unwrap();
    }
}
