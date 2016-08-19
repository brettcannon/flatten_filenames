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
pub fn leading_char(path: &path::PathBuf) -> char {
    let filename = path.file_name().expect("path lacks filename");
    let filename_str = filename.to_str().expect("filename as str");
    filename_str.chars().next().unwrap()
}

/// Check if a `entry` is a directory that doesn't have any special
/// leading characters.
///
/// The characters that signal not to traverse into a directory are
/// '.' and '_'.
pub fn should_traverse(entry: &fs::DirEntry) -> bool {
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
pub fn new_prefix(old_prefix: &str, tail: &str) -> String {
    if old_prefix.is_empty() {
        tail.to_string()
    } else {
        let mut postfix = tail;
        if tail[0..1] == "+".to_string() || tail[0..1] == "-".to_string() {
            postfix = &tail[1..];
        }

        old_prefix.to_string() + " - " + postfix
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

#[cfg(test)]
mod test {
    use super::*;

    use std::fs;
    use std::path;

    extern crate tempdir;

    #[test]
    fn leading_char_for_filename() {
        let mut path = path::PathBuf::new();
        path.push("/tmp");
        path.push("file.txt");
        assert_eq!(leading_char(&path), 'f');
    }

    #[test]
    fn should_traverse_not_dir() {
        // Create a temporary directory.
        let tmp_dir = tempdir::TempDir::new("test");
        if tmp_dir.is_err() {
            return;
        }
        let tmp_dir = tmp_dir.unwrap();

        // Create a file.
        let tmp_dir_path = tmp_dir.path();
        let mut path_buf = tmp_dir_path.to_path_buf();
        path_buf.push("file.txt");
        let f = fs::File::create(&path_buf);
        if f.is_err() {
            return;
        }
        let f = f.unwrap();
        // Flush the file.
        if f.sync_all().is_err() {
            return;
        }

        // Get the temporary directory's content.
        let read_dir = path_buf.read_dir();
        if read_dir.is_err() {
            return;
        }
        let entry_item = read_dir.unwrap().last();
        let entry_option = entry_item.unwrap();
        let entry = entry_option.unwrap();

        assert!(!should_traverse(&entry));
    }

    #[test]
    fn should_traverse_not_leading_dot_or_underscore() {
        let tmp_dir = tempdir::TempDir::new("test");
        if tmp_dir.is_err() {
            return;
        }
        let tmp_dir = tmp_dir.unwrap();

        let dir_builder = fs::DirBuilder::new();
        let tmp_dir_path = tmp_dir.path();
        let mut path_buf = tmp_dir_path.to_path_buf();
        path_buf.push(".directory");
        if dir_builder.create(path_buf.as_path()).is_err() {
            return;
        } else {
            path_buf.pop();
        }

        path_buf.push("_directory");
        if dir_builder.create(path_buf.as_path()).is_err() {
            return;
        } else {
            path_buf.pop();
        }

        // Get the temporary directory's content.
        let read_dir = path_buf.read_dir();
        if read_dir.is_err() {
            return;
        }

        let mut count = 0;
        for entry in read_dir.unwrap() {
            assert!(!should_traverse(&entry.unwrap()));
            count += 1;
        }
        assert_eq!(2, count);
    }

    #[test]
    fn should_traverse_directory() {
let tmp_dir = tempdir::TempDir::new("test");
        if tmp_dir.is_err() {
            return;
        }
        let tmp_dir = tmp_dir.unwrap();

        let dir_builder = fs::DirBuilder::new();
        let tmp_dir_path = tmp_dir.path();
        let mut path_buf = tmp_dir_path.to_path_buf();
        path_buf.push("directory");
        if dir_builder.create(path_buf.as_path()).is_err() {
            return;
        } else {
            path_buf.pop();
        }

        // Get the temporary directory's content.
        let read_dir = path_buf.read_dir();
        if read_dir.is_err() {
            return;
        }

        let mut count = 0;
        for entry in read_dir.unwrap() {
            assert!(should_traverse(&entry.unwrap()));
            count += 1;
        }
        assert_eq!(1, count);
    }

    #[test]
    fn new_prefix_empty_old_prefix() {
        assert_eq!("tail", new_prefix("", "tail"));
    }

    #[test]
    fn new_prefix_leading_dash_or_plus() {
        assert_eq!("a - b", new_prefix("a", "-b"));
        assert_eq!("a - b", new_prefix("a", "+b"));
    }

    #[test]
    fn new_prefix_works() {
        assert_eq!("a - b", new_prefix("a", "b"));
        assert_eq!("a - b - c", new_prefix("a - b", "c"));
    }
}
