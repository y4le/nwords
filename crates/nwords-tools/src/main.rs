#![forbid(unsafe_code)]

use std::env;
use std::path::Path;
use std::process::{Command, ExitCode};

use nwords_tools::{
    check_existing_vectors, emit_rust_array, read_word_file, sample_shape, ToolError,
};

fn main() -> ExitCode {
    match run() {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<String, ToolError> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [command] if command == "check-existing" => {
            check_existing_vectors(Path::new("."))?;
            Ok("existing wordlist vectors: ok\n".to_owned())
        }
        [command, root] if command == "check-existing" => {
            check_existing_vectors(Path::new(root))?;
            Ok("existing wordlist vectors: ok\n".to_owned())
        }
        [command, const_name, path] if command == "emit-array" => {
            let words = read_word_file(Path::new(path))?;
            emit_rust_array(const_name, &words)
        }
        [command, paths @ ..] if command == "sample-shape" && !paths.is_empty() => {
            let lists = paths
                .iter()
                .map(|path| read_word_file(Path::new(path)))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(sample_shape(&lists, 64)?.join("\n") + "\n")
        }
        [command, sha_file] if command == "check-sha256" => check_sha256(Path::new(sha_file)),
        _ => Err(ToolError::Usage(USAGE.to_owned())),
    }
}

fn check_sha256(path: &Path) -> Result<String, ToolError> {
    let output = Command::new("sha256sum")
        .arg("-c")
        .arg(path)
        .output()
        .map_err(|error| ToolError::Io {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let mut message = String::from_utf8_lossy(&output.stdout).into_owned();
        message.push_str(&String::from_utf8_lossy(&output.stderr));
        Err(ToolError::Usage(message))
    }
}

const USAGE: &str = "\
usage:
  nwords-tools check-existing [repo-root]
  nwords-tools emit-array <CONST_NAME> <word-file>
  nwords-tools sample-shape <word-file>...
  nwords-tools check-sha256 <SHA256SUMS>
";
