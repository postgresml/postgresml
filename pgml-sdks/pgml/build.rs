use std::fs::remove_file;
use std::fs::OpenOptions;
use std::io::Write;

const ADDITIONAL_DEFAULTS_FOR_PYTHON: &[u8] = br#"
def init_logger(level: Optional[str] = "", format: Optional[str] = "") -> None
def SingleFieldPipeline(name: str, model: Optional[Model] = None, splitter: Optional[Splitter] = None, parameters: Optional[Json] = Any) -> Pipeline
async def migrate() -> None

Json = Any
DateTime = int
GeneralJsonIterator = Any
GeneralJsonAsyncIterator = Any
"#;

const ADDITIONAL_DEFAULTS_FOR_JAVASCRIPT: &[u8] = br#"
export function init_logger(level?: string, format?: string): void;
export function newSingleFieldPipeline(name: string, model?: Model, splitter?: Splitter, parameters?: Json): Pipeline;
export function migrate(): Promise<void>;

export type Json = any;
export type DateTime = Date;
export type GeneralJsonIterator = any;
export type GeneralJsonAsyncIterator = any;

export function newCollection(name: string, database_url?: string): Collection;
export function newModel(name?: string, source?: string, parameters?: Json): Model;
export function newSplitter(name?: string, parameters?: Json): Splitter;
export function newBuiltins(database_url?: string): Builtins;
export function newPipeline(name: string, schema?: Json): Pipeline;
export function newTransformerPipeline(task: string, model?: string, args?: Json, database_url?: string): TransformerPipeline;
export function newOpenSourceAI(database_url?: string): OpenSourceAI;
"#;

fn main() {
    // Remove python stub file that is auto generated each build
    let path = std::env::var("PYTHON_STUB_FILE");
    if let Ok(path) = path {
        remove_file(&path).ok();
        let mut file = OpenOptions::new()
            .create(true)
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
            .append(true)
            .open(path)
            .unwrap();
        // Add some manual declarations here
        file.write_all(ADDITIONAL_DEFAULTS_FOR_JAVASCRIPT).unwrap();
    }
}
