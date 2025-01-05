use std::process;

use rustyline::{error::ReadlineError, Editor};
use writing_interpreters::interpreter::{memory::Memory, repl::RepMaker, RuntimeError};

/// Read a line at a time, printing the input back out
fn read_print_loop() -> Result<(), RuntimeError> {
    // establish a repl input history file path
    let history_file = match dirs::home_dir() {
        Some(mut path) => {
            path.push(".evalrus_history");
            Some(String::from(path.to_str().unwrap()))
        }
        None => None,
    };

    // () means no completion support (TODO)
    // Another TODO - find a more suitable alternative to rustyline
    let mut reader = Editor::<()>::new();

    // Try to load the repl history file
    if let Some(ref path) = history_file {
        if let Err(err) = reader.load_history(&path) {
            eprintln!("Could not read history: {}", err);
        }
    }

    let mem = Memory::new();
    let rep_maker = RepMaker {};
    let rep = mem.mutate(&rep_maker, ())?;

    // repl
    loop {
        let readline = reader.readline("> ");

        match readline {
            // valid input
            Ok(line) => {
                reader.add_history_entry(&line);
                mem.mutate(&rep, line)?;
            }

            // some kind of program termination condition
            Err(e) => {
                if let Some(ref path) = history_file {
                    reader.save_history(&path).unwrap_or_else(|err| {
                        eprintln!("could not save input history in {}: {}", path, err);
                    });
                }

                // EOF is fine
                if let ReadlineError::Eof = e {
                    return Ok(());
                } else {
                    return Err(RuntimeError::from(e));
                }
            }
        }
    }
}

fn main() {
    // otherwise begin a repl
    read_print_loop().unwrap_or_else(|err| {
        eprintln!("Terminated: {}", err);
        process::exit(1);
    });
}
