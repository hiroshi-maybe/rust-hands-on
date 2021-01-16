use std::fs::File;
use std::io::{self, ErrorKind, Read};

fn main() {
    {
        let res = File::open("hello.txt");
        let f = match res {
            Ok(file) => file,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => match File::create("hello.txt") {
                    Ok(fc) => fc,
                    Err(e) => panic!("roblem creating the file: {:?}", e),
                },
                other_error => {
                    panic!("Problem opening the file: {:?}", other_error);
                }
            },
        };

        println!("Opened file 1: {:?}", f);
    }

    {
        let f = File::open("hello.txt").unwrap_or_else(|error| {
            if error.kind() == ErrorKind::NotFound {
                File::create("hello.txt").unwrap_or_else(|error| {
                    panic!("roblem creating the file: {:?}", error);
                })
            } else {
                panic!("Problem opening the file: {:?}", error);
            }
        });

        println!("Opened file 2: {:?}", f);
    }

    {
        let f = File::open("hello.txt").expect("Failed to open hello.txt");
        println!("Opened file 3: {:?}", f);
    }

    {
        fn read_username_from_file() -> Result<String, io::Error> {
            let f = File::open("hello.txt");
            let mut f = match f {
                Ok(f) => f,
                Err(e) => return Err(e),
            };

            let mut s = String::new();
            match f.read_to_string(&mut s) {
                Ok(_) => Ok(s),
                Err(e) => Err(e),
            }
        }

        println!("Read username from file 1: {:?}", read_username_from_file());
    }

    {
        fn read_username_from_file() -> Result<String, io::Error> {
            let mut s = String::new();
            //let mut f = File::open("hello.txt")?;
            //f.read_to_string(&mut s)?;
            File::open("hello.txt")?.read_to_string(&mut s)?;

            Ok(s)
        }
        println!("Read username from file 2: {:?}", read_username_from_file());
    }
}
