use std::any::type_name;
use std::cmp;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut parser = match Parser::new(&args) {
        Ok(a) => a,
        Err(e) => {
            print!("{}", e);
            process::exit(1);
        }
    };

    while parser.hasMoreCommands() {
        parser.print();
        parser.advance();
    }
}

fn parse_filename(args: &[String]) -> Result<String, &'static str> {
    if args.len() < 2 {
        return Err("not enought argument");
    }
    let filename = args[1].clone();
    return Ok(filename);
}

struct Parser {
    code: Vec<String>,
    position: usize,
}

enum Dest {
    Null,
    M,
    D,
    MD,
    A,
    AM,
    AD,
    AMD,
}

enum Jump {
    Null,
    JGT,
    JEQ,
    JGE,
    JLT,
    JNE,
    JLE,
    JMP,
}

#[derive(PartialEq)]
enum CommandType {
    ACOMMAND,
    CCOMMAND,
    LCOMMAND,
}

impl Parser {
    pub fn new(args: &[String]) -> Result<Parser, &'static str> {
        let filename = parse_filename(args);
        let filename = filename?;
        let f = match File::open(filename) {
            Ok(f) => f,
            Err(_) => return Err("cannot open file"),
        };

        let reader = BufReader::new(f);
        let mut buf = Vec::new();

        for line in reader.lines() {
            let line = match line {
                Ok(s) => s,
                Err(_) => continue,
            };

            let right = match line.find("//") {
                Some(right) => right,
                None => line.len(),
            };

            let line = line[0..right].trim();

            if line.len() == 0 {
                continue;
            }
            buf.push(String::from(line));
        }

        Ok(Parser {
            code: buf,
            position: 0,
        })
    }

    fn print(&self) {
        println!("code: {:?}", self.code[self.position]);
        match self.dest() {
            None => println!("dest none"),
            Some(s) => {
                println!("dest = {:?}", s);
            }
        };

        match self.comp() {
            None => println!("comp none"),
            Some(s) => {
                println!("comp = {:?}", s);
            }
        };

        match self.jump() {
            None => println!("jump none"),
            Some(s) => {
                println!("jump = {:?}", s);
            }
        }
    }

    fn next(&mut self) -> Option<&str> {
        let res: Option<&str> = if self.position < self.code.len() {
            Some(&self.code[self.position])
        } else {
            None
        };
        self.position += 1;

        return res;
    }

    fn advance(&mut self) {
        if self.hasMoreCommands() {
            self.position += 1;
        };
    }

    fn hasMoreCommands(&self) -> bool {
        if self.position < self.code.len() {
            true
        } else {
            false
        }
    }

    fn commandType(&self) -> Option<CommandType> {
        if !self.hasMoreCommands() {
            return None;
        }

        if self.code[self.position].starts_with("@") {
            return Some(CommandType::ACOMMAND);
        } else if self.code[self.position].starts_with("(") {
            return Some(CommandType::LCOMMAND);
        } else {
            return Some(CommandType::CCOMMAND);
        }
    }

    fn symbol(&self) -> Option<&str> {
        let res = match self.commandType() {
            None => return None,
            Some(CommandType::CCOMMAND) => return None,
            Some(CommandType::ACOMMAND) => &self.code[self.position],
            Some(CommandType::LCOMMAND) => &self.code[self.position],
        };

        if res.starts_with("@") {
            return Some(&res[1..]);
        }

        if res.starts_with("(") {
            if !res.ends_with(")") {
                print!("symbol input format is incorrect");
                return None;
            }
            return Some(&res[1..res.len() - 1]);
        }

        print!("symbol input format is incorrect");
        None
    }

    fn dest(&self) -> Option<&str> {
        let code = match self.commandType() {
            Some(CommandType::CCOMMAND) => &self.code[self.position],
            _ => return None,
        };
        let right: usize = cmp::max(code.find(";").unwrap_or(0), code.find("=").unwrap_or(0));
        let res = &self.code[self.position][0..right];
        Some(res)
    }

    fn comp(&self) -> Option<&str> {
        let code = match self.commandType() {
            Some(CommandType::CCOMMAND) => &self.code[self.position],
            _ => return None,
        };

        let left: usize = match code.find("=") {
            None => 0,
            Some(num) => num + 1,
        };
        let right: usize = code.find(";").unwrap_or(code.len());

        Some(&self.code[self.position][left..right])
    }

    fn jump(&self) -> Option<&str> {
        let code = match self.commandType() {
            Some(CommandType::CCOMMAND) => &self.code[self.position],
            _ => return None,
        };

        let left: usize = code.find(";")?;

        Some(&self.code[self.position][left..])
    }
}

struct SymbolTable {
    table: HashMap<String, usize>,
    address: usize,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            table: HashMap::new(),
            address: 1024,
        }
    }

    fn addEntry(&mut self, symbol: &String, address: usize) {
        self.table.insert(symbol.clone(), self.address);
        self.address += 1;
    }

    fn contains(&self, symbol: &str) -> bool {
        match self.table.get(symbol) {
            None => false,
            _ => true,
        }
    }

    fn getAddress(&self, symbol: &str) -> Option<&usize> {
        self.table.get(symbol)
    }
}
