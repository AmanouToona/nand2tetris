use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // let filename = parse_filename(&args).unwrap_or_else(|err| {
    //     println!("Problem parsing arguments: {}", err);
    //     process::exit(1);
    // });

    // let mut f = File::open(filename).expect("file not found");

    // let mut contents = String::new();
    // f.read_to_string(&mut contents)
    //     .expect("something went wrong reading the file");

    // println!("with text:\n{}", contents);

    let parser = Parser::new(&args);
}

fn parse_filename(args: &[String]) -> Result<String, &'static str> {
    if args.len() < 2 {
        return Err("not enought argument");
    }
    let filename = args[1].clone();
    return Ok(filename);
}

struct Parser {
    code: String,
}

impl Parser {
    pub fn new(args: &[String]) -> Result<Parser, &'static str> {
        let filename = parse_filename(args);
        let filename = match filename {
            Ok(file) => file,
            Err(e) => return Err(e),
        };

        let mut f = match File::open(filename) {
            Ok(f) => f,
            Err(_) => return Err("cannot open file"),
        };

        let mut code = String::new();
        match f.read_to_string(&mut code) {
            Ok(_) => Ok(Parser { code }),
            Err(_) => return Err("cannot read string"),
        }
    }
}
