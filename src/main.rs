use std::env;
use std::error::Error;
use std::io::Write;  // Need `write_fmt()` method for `writeln!()`.
use std::path;
use std::process;

/// Prints a message to `std::io::stderr`.
fn println_stderr(message: String) {
    let r = writeln!(&mut std::io::stderr(), "{}", message);
    r.expect("failed to write to stderr");
}

/// "Flattens" `directory by prepending `prefix` plus the directories
/// name.
///
/// Certain considerations are taken into account based on the leading
/// character of the directory's name.
fn flatten(directory: &path::PathBuf, prev_prefix: String) {
    let path_tail = directory.file_name().expect("directory lacks a tail");
    let prefix = prev_prefix + path_tail.to_str().expect("can't decode path tail");
    for entry in directory.read_dir().unwrap() {
        let entry = entry.unwrap();
        println!("{:?}", entry.file_name());
    }
}


fn main() {
    // Parse arguments.
    let mut args = env::args();
    // Program name (argument 0).
    args.next().expect("no program name specified!?!");

    // Directory to process (argument 1).
    let directory = match args.next() {
        Some(dir) => dir,
        None => {
            println_stderr("Expected an argument".to_string());
            process::exit(1);
        }
    };

    // Already consumed all the arguments that I care about.
    if args.next().is_some() {
        println_stderr(format!("expected only 1 argument, not {}", args.len() + 1));
        process::exit(1);
    }

    let path = match path::Path::new(&directory).canonicalize() {
        Ok(o) => o,  // Using o.as_path() won't work as `o` leaves the scope.
        Err(e) => {
            println_stderr(e.description().to_string());
            process::exit(1);
        }
    };

    if !path.is_dir() {
        println_stderr("argument is not a directory".to_string());
        process::exit(1);
    }

    flatten(&path, "".to_string());
}
