use std::env;
use std::error::Error;
use std::io::Write;  // Need `write_fmt()` method for `writeln!()`.
use std::path;
use std::process;


fn println_stderr(message: String) {
    let r = writeln!(&mut std::io::stderr(), "{}", message);
    r.expect("failed to write to stderr");
}


fn flatten(directory: &path::Path, prefix: String) {
    println!("{:?} {}", directory, prefix);
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

    println!("Got {} as an argument", directory);
    let path = match path::Path::new(&directory).canonicalize() {
        Ok(o) => o,  // Using o.as_path() won't work as `o` leaves the scope.
        Err(e) => {
            println_stderr(e.description().to_string());
            process::exit(1);
        }
    };

    flatten(path.as_path(), "".to_string());
}
