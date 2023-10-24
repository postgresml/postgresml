use owo_colors::OwoColorize;
use std::fs::{read_to_string, File};
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process::Command;

macro_rules! unwrap_or_exit {
    ($i:expr) => {
        match $i {
            Ok(v) => v,
            Err(e) => {
                error!("{}:{}:{} {e}", file!(), line!(), column!());

                std::process::exit(1);
            }
        }
    };
}

macro_rules! debug1 {
    ($e:expr) => {
        debug!("{}:{}:{} {}", file!(), line!(), column!(), $e);
    };
}

pub(crate) use debug1;
pub(crate) use unwrap_or_exit;

pub fn info(value: &str) {
    println!("{}", value.green());
}

pub fn error(value: &str) {
    println!("{}", value.red());
}

pub fn warn(value: &str) {
    println!("{}", value.yellow());
}

pub fn execute_command(command: &mut Command) -> std::io::Result<String> {
    debug!("Executing {:?}", command);

    let output = match command.output() {
        Ok(output) => output,
        Err(err) => {
            return Err(err);
        }
    };

    let stderr = unwrap_or_exit!(String::from_utf8(output.stderr)).to_string();
    let stdout = unwrap_or_exit!(String::from_utf8(output.stdout)).to_string();

    if !output.status.success() {
        debug!(
            "{} failed: {}",
            command.get_program().to_str().unwrap(),
            stderr,
        );

        return Err(std::io::Error::new(ErrorKind::Other, stderr));
    }

    if !stderr.is_empty() {
        warn!("{}", stderr);
    }

    if !stdout.is_empty() {
        info!("{}", stdout);
    }

    Ok(stdout.clone() + &stderr)
}

pub fn write_to_file(path: &Path, content: &str) -> std::io::Result<()> {
    debug!("writing to file: {}", path.display());

    let mut file = File::create(path)?;

    file.write_all(content.as_bytes())?;

    Ok(())
}

#[allow(dead_code)]
pub fn compare_files(path1: &Path, path2: &Path) -> std::io::Result<bool> {
    let content1 = read_to_string(path1)?;
    let content2 = read_to_string(path2)?;

    Ok(compare_strings(&content1, &content2))
}

pub fn compare_strings(string1: &str, string2: &str) -> bool {
    // TODO: faster string comparison method needed.
    string1.trim() == string2.trim()
}

pub fn psql_output(query: &str) -> std::io::Result<String> {
    let mut cmd = Command::new("psql");
    cmd.arg("-c").arg(query).arg("-t").arg("-d").arg("postgres");

    let output = execute_command(&mut cmd)?;
    Ok(output.trim().to_string())
}

pub fn print(s: &str) {
    print!("{}", s);
    let _ = std::io::stdout().flush();
}

macro_rules! ok_or_error {
    ($what:expr, $expr:block, $howto:expr) => {{
        use std::io::Write;
        print!("{}...", $what);
        let _ = std::io::stdout().flush();

        if $expr {
            crate::util::info("ok");
        } else {
            crate::util::error("error");
            println!("{}", $howto);
            std::process::exit(1);
        }
    }};
}

pub(crate) use ok_or_error;
