use adamantium_files as files;
use std::ffi::OsString;
use std::io::{self, Write};

const HELP: &str = "Usage: adamantium-files <command> <path> [argument]
Commands:
  read <path>               Write raw file bytes to stdout
  write <path> <text>       Create or overwrite a UTF-8 text file
  append <path> <text>      Append UTF-8 text without adding a newline
  exists <path>             Print true or false
  copy <source> <target>    Copy a file, replacing an existing target
  rename <source> <target>  Rename a file or directory
  remove <path>            Remove a file
  mkdir <path>             Create directories, including missing parents
  rmdir <path>             Remove an empty directory
  list <path>              Print sorted entry names, one per line
  --help                   Show this help
";

fn run(args: &[OsString]) -> Result<(), (u8, String)> {
    if args.len() == 1 && args[0] == "--help" {
        return io::stdout()
            .write_all(HELP.as_bytes())
            .map_err(|error| (1, error.to_string()));
    }
    let command = args.first().and_then(|arg| arg.to_str()).unwrap_or("");
    let arity = match command {
        "read" | "exists" | "remove" | "mkdir" | "rmdir" | "list" => 2,
        "write" | "append" | "copy" | "rename" => 3,
        _ => return Err((2, HELP.into())),
    };
    if args.len() != arity {
        return Err((2, HELP.into()));
    }
    let path = &args[1];
    let result: io::Result<()> = (|| {
        match command {
            "read" => io::stdout().write_all(&files::read(path)?)?,
            "write" | "append" => {
                let text = args[2].to_str().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "text must be UTF-8")
                })?;
                if command == "write" {
                    files::write(path, text)?;
                } else {
                    files::append(path, text)?;
                }
            }
            "exists" => writeln!(io::stdout(), "{}", files::exists(path)?)?,
            "copy" => {
                files::copy(path, &args[2])?;
            }
            "rename" => files::rename(path, &args[2])?,
            "remove" => files::remove_file(path)?,
            "mkdir" => files::create_dir(path)?,
            "rmdir" => files::remove_dir(path)?,
            "list" => {
                for name in files::list_dir(path)? {
                    writeln!(io::stdout(), "{}", name.display())?;
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    })();
    result.map_err(|error| (1, format!("{command}: {}: {error}", path.to_string_lossy())))
}

fn main() -> std::process::ExitCode {
    match run(&std::env::args_os().skip(1).collect::<Vec<_>>()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err((code, message)) => {
            let _ = writeln!(io::stderr(), "{message}");
            std::process::ExitCode::from(code)
        }
    }
}
