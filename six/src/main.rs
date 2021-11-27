// use std::any::type_name;
use std::collections::HashMap;
use std::env;
use std::fs;
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

    let mut symboltable = SymbolTable::new();
    let mut addresses: Vec<String> = Vec::new();

    // first loop to collect symbol (Xxx)
    let mut address_num: usize = 0;
    while parser.has_more_commands() {
        match parser.command_type() {
            Some(CommandType::LCOMMAND) => {
                let symbol = match parser.symbol() {
                    Some(sym) => sym,
                    None => panic!(),
                };
                symboltable.add_entry(symbol.to_string(), address_num);
            }
            _ => address_num += 1,
        }
        parser.advance();
    }

    parser.reset();

    // second loop
    address_num = 16;
    while parser.has_more_commands() {
        let mut binary_code = "".to_string();

        match parser.command_type() {
            Some(CommandType::ACOMMAND) => {
                let symbol = match parser.symbol() {
                    Some(sym) => sym,
                    None => panic!(),
                };
                if symbol.parse::<usize>().is_ok() {
                    binary_code =
                        format!("{:0>16}", format!("{:b}", symbol.parse::<usize>().unwrap()));
                } else {
                    if !symboltable.contains(&symbol.to_string()) {
                        symboltable.add_entry(symbol.to_string(), address_num);
                        address_num += 1;
                    };
                    let address = symboltable.get_address(symbol).unwrap();
                    binary_code = format!("{:0>16}", format!("{:b}", address))
                }
            }
            Some(CommandType::CCOMMAND) => {
                let comp = parser.comp().unwrap_or("null");
                let dest = parser.dest().unwrap_or("null");
                let jump = parser.jump().unwrap_or("null");
                let comp = code::comp(comp);
                let dest = code::dest(dest);
                let jump = code::jump(jump);
                binary_code = format!("111{}{}{}", comp, dest, jump);
            }
            _ => (),
        }

        if binary_code.len() != 0 {
            addresses.push(binary_code);
        }

        parser.advance();
    }

    let address = addresses.join("\n");
    fs::write(format!("{}.hack", "test"), address).unwrap();
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

    fn reset(&mut self) {
        self.position = 0;
    }

    fn print(&self) {
        // print code for debug;
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

    fn advance(&mut self) {
        if self.has_more_commands() {
            self.position += 1;
        };
    }

    fn has_more_commands(&self) -> bool {
        if self.position < self.code.len() {
            true
        } else {
            false
        }
    }

    fn command_type(&self) -> Option<CommandType> {
        if !self.has_more_commands() {
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
        let res = match self.command_type() {
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
        let code = match self.command_type() {
            Some(CommandType::CCOMMAND) => &self.code[self.position],
            _ => return None,
        };

        let right: usize = code.find("=")?;
        let res = &self.code[self.position][0..right];
        Some(res)
    }

    fn comp(&self) -> Option<&str> {
        let code = match self.command_type() {
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
        let code = match self.command_type() {
            Some(CommandType::CCOMMAND) => &self.code[self.position],
            _ => return None,
        };

        let left: usize = code.find(";")? + 1;

        Some(&self.code[self.position][left..])
    }
}

struct SymbolTable {
    table: HashMap<String, usize>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        let mut new_table = SymbolTable {
            table: HashMap::new(),
        };

        new_table.table.insert("SP".to_string(), 0);
        new_table.table.insert("LCL".to_string(), 1);
        new_table.table.insert("ARG".to_string(), 2);
        new_table.table.insert("THIS".to_string(), 3);
        new_table.table.insert("THAT".to_string(), 4);
        new_table.table.insert("R0".to_string(), 0);
        new_table.table.insert("R1".to_string(), 1);
        new_table.table.insert("R2".to_string(), 2);
        new_table.table.insert("R3".to_string(), 3);
        new_table.table.insert("R4".to_string(), 4);
        new_table.table.insert("R5".to_string(), 5);
        new_table.table.insert("R6".to_string(), 6);
        new_table.table.insert("R7".to_string(), 7);
        new_table.table.insert("R8".to_string(), 8);
        new_table.table.insert("R9".to_string(), 9);
        new_table.table.insert("R10".to_string(), 10);
        new_table.table.insert("R11".to_string(), 11);
        new_table.table.insert("R12".to_string(), 12);
        new_table.table.insert("R13".to_string(), 13);
        new_table.table.insert("R14".to_string(), 14);
        new_table.table.insert("R15".to_string(), 15);
        new_table.table.insert("SCREEN".to_string(), 16384);
        new_table.table.insert("KBD".to_string(), 24576);

        new_table
    }

    fn add_entry(&mut self, symbol: String, address: usize) {
        if self.contains(&symbol) {
            return;
        } else {
            self.table.insert(symbol, address);
        }
    }

    fn contains(&self, symbol: &str) -> bool {
        match self.table.get(symbol) {
            None => false,
            _ => true,
        }
    }

    fn get_address(&self, symbol: &str) -> Option<&usize> {
        self.table.get(symbol)
    }
}

pub mod code {
    // code modele
    pub fn dest(mnemonic: &str) -> &str {
        match mnemonic {
            "M" => "001",
            "D" => "010",
            "MD" => "011",
            "A" => "100",
            "AM" => "101",
            "AD" => "110",
            "AMD" => "111",
            _ => "000",
        }
    }

    pub fn comp(mnemonic: &str) -> &str {
        match mnemonic {
            "0" => "0101010",
            "1" => "0111111",
            "-1" => "0111010",
            "D" => "0001100",
            "A" => "0110000",
            "!D" => "0001101",
            "!A" => "0110001",
            "-D" => "0001111",
            "-A" => "0110011",
            "D+1" => "0011111",
            "A+1" => "0110111",
            "D-1" => "0001110",
            "A-1" => "0110010",
            "D+A" => "0000010",
            "D-A" => "0010011",
            "A-D" => "0000111",
            "D&A" => "0000000",
            "D|A" => "0010101",
            "M" => "1110000",
            "!M" => "1110001",
            "-M" => "1110011",
            "M+1" => "1110111",
            "M-1" => "1110010",
            "D+M" => "1000010",
            "D-M" => "1010011",
            "M-D" => "1000111",
            "D&M" => "1000000",
            "D|M" => "1010101",
            _ => panic!("unexpected mnemonic was passed {}", mnemonic),
        }
    }

    pub fn jump(mnemonic: &str) -> &str {
        match mnemonic {
            "JGT" => "001",
            "JEQ" => "010",
            "JGE" => "011",
            "JLT" => "100",
            "JNE" => "101",
            "JLE" => "110",
            "JMP" => "111",
            _ => "000",
        }
    }
}
