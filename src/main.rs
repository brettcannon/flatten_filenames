use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;  // Need `write_fmt()` method for `writeln!()`.
use std::path;
use std::process;

/// Prints a message to `std::io::stderr`.
fn println_stderr(message: String) {
    let r = writeln!(&mut std::io::stderr(), "{}", message);
    r.expect("failed to write to stderr");
}


/// Extract the leading character of a path.
fn leading_char(path: &path::PathBuf) -> char {
    let filename = path.file_name().expect("dir lacks filename");
    let filename_str = filename.to_str().expect("dir as str");
    filename_str.chars().next().unwrap()
}

/// Check if a `entry` is a directory that doesn't have any special
/// leading characters.
///
/// The characters that signal not to traverse into a directory are
/// '.' and '_'.
fn should_traverse(entry: &fs::DirEntry) -> bool {
    if entry.metadata().unwrap().is_dir() {
        let path = entry.path();
        let leading_char = leading_char(&path);
        if leading_char != '.' && leading_char != '_' {
            true
        } else {
            false
        }
    } else {
        false
    }
}

/// Rename a file with a prefix.
///
/// If the file starts with '.' then skip the renaming.
fn rename(path: &path::PathBuf, prefix: &str) {
    if leading_char(path) == '.' {
        return;
    }
    // XXX Rename file with prefix
    println!("{} - {:?}", prefix, path.file_name());
}

/// Create the filename prefix.
///
/// If a new part starts with '-' or '+' then strip it off.
fn new_prefix(old_prefix: &str, tail: &str) -> String {
    if old_prefix.is_empty() {
        tail.to_string()
    } else {
        // XXX strip off '-' or '+'
        old_prefix.to_string() + " - " + tail
    }
}

/// "Flattens" `directory by prepending `prefix` plus the directories
/// name.
///
/// Certain considerations are taken into account based on the leading
/// character of the directory's name.
fn flatten(directory: &path::PathBuf, prev_prefix: &str) {
    let filename = directory.file_name().expect("directory lacks a tail");
    let path_tail = filename.to_str().expect("can't decode path tail");
    let prefix = new_prefix(prev_prefix, path_tail);
    let prefix_str = prefix.as_str();
    for entry in directory.read_dir().unwrap() {
        let entry = entry.unwrap();
        let entry_path = entry.path();
        if should_traverse(&entry) {
            flatten(&entry_path, prefix_str);
        } else {
            rename(&entry_path, prefix_str);
        }
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

    flatten(&path, "");
}
