use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = parse_filename(&args).unwrap();
    let mut parser = Parser::new(&filename).unwrap();

    let output_file = format!("{}.asm", filename);
    let mut writer = CodeWriter::new(&output_file).unwrap();

    while parser.has_more_commands() {
        match parser.command_type() {
            CommandType::CPOP => writer.write_pushpop("pop", parser.arg1(), parser.arg2()),
            CommandType::CPUSH => writer.write_pushpop("push", parser.arg1(), parser.arg2()),
            CommandType::CARITHMETIC => writer.write_arithmetic(parser.arg1()),
            _ => continue,
        };
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
    commands: Vec<String>,
    command_pos: usize,
}

impl Parser {
    pub fn new(filename: &str) -> Result<Parser, &str> {
        let f = match File::open(filename) {
            Ok(f) => f,
            Err(_) => {
                println!("{}", filename);
                return Err("cannot open file !!");
            }
        };

        let reader = BufReader::new(f);
        let mut buf = Vec::new();

        for line in reader.lines() {
            let line = match line {
                Ok(s) => s,
                Err(_) => continue,
            };
            // delete comment and space
            let right = match line.find("//") {
                Some(num) => num,
                None => line.len(),
            };

            let line = line[..right].trim();

            if line.len() == 0 {
                continue;
            };

            buf.push(String::from(line));
        }

        Ok(Parser {
            commands: buf,
            command_pos: 0,
        })
    }

    pub fn has_more_commands(&self) -> bool {
        if self.command_pos < self.commands.len() {
            return true;
        } else {
            return false;
        }
    }

    pub fn advance(&mut self) {
        if !self.has_more_commands() {
            panic!("advance is called though no more command !!")
        }
        self.command_pos += 1;
    }

    pub fn command_type(&self) -> CommandType {
        let command = &self.commands[self.command_pos];
        if command.starts_with("push") {
            return CommandType::CPUSH;
        } else if command.starts_with("pop") {
            return CommandType::CPOP;
        } else if command.starts_with("label") {
            return CommandType::CLABEL;
        } else if command.starts_with("goto") {
            return CommandType::CGOTO;
        } else if command.starts_with("if") {
            return CommandType::CIF;
        } else if command.starts_with("function") {
            return CommandType::CFUNCTION;
        } else if command.starts_with("call") {
            return CommandType::CCALL;
        } else if command.starts_with("return") {
            return CommandType::CRETURN;
        } else {
            return CommandType::CARITHMETIC;
        }
    }

    pub fn arg1(&self) -> &str {
        let command = &self.commands[self.command_pos];

        match self.command_type() {
            CommandType::CARITHMETIC => {
                let right = match command.find(" ") {
                    None => command.len(),
                    Some(num) => num,
                };
                &command[..right]
            }
            CommandType::CRETURN => panic!("arg1 is called wile comamnd type is 'return'"),
            _ => {
                let left = match command.find(" ") {
                    None => 0,
                    Some(num) => num + 1,
                };
                let command = &command[left..];

                let right = match command.find(" ") {
                    None => command.len(),
                    Some(num) => num,
                };
                &command[..right]
            }
        }
    }

    pub fn arg2(&self) -> &str {
        match self.command_type() {
            CommandType::CARITHMETIC => panic!(""),
            CommandType::CLABEL => panic!(""),
            CommandType::CGOTO => panic!(""),
            CommandType::CIF => panic!(""),
            CommandType::CRETURN => panic!(""),
            _ => {
                let command = &self.commands[self.command_pos];
                let left = command.find(" ").unwrap() + 1;
                let args = &command[left..];
                let left = args.find(" ").unwrap() + 1;
                let arg = &args[left..];
                arg
            }
        }
    }
}

enum CommandType {
    CARITHMETIC,
    CPUSH,
    CPOP,
    CLABEL,
    CGOTO,
    CIF,
    CFUNCTION,
    CRETURN,
    CCALL,
}

struct CodeWriter {
    output_file: BufWriter<File>,
    jmp_point: i64,
}

impl CodeWriter {
    pub fn new(filename: &str) -> Result<CodeWriter, io::Error> {
        let writer = BufWriter::new(File::create(filename)?);
        Ok(CodeWriter {
            output_file: writer,
            jmp_point: 0,
        })
    }

    pub fn write_arithmetic(&mut self, command: &str) {
        match command {
            "add" => self.arithmetic_add(),
            "sub" => self.arithmetic_sub(),
            "neg" => self.arithmetic_neg(),
            "eq" => self.arithmetic_eq(),
            "gt" => self.arithmetic_gt(),
            "lt" => self.arithmetic_lt(),
            "and" => self.arithmetic_and(),
            "or" => self.arithmetic_or(),
            "not" => self.arithmetic_not(),
            _ => panic!("{} is not a arithmetic command !!", command),
        }
    }

    pub fn write_pushpop(&mut self, command: &str, segment: &str, index: &str) {
        match command {
            "push" => self.push(segment, index),
            "pop" => self.pop(segment, index),
            _ => panic!("{} is not push pop command !!", command),
        }
    }

    fn push(&mut self, segment: &str, index: &str) {
        let data_position = {
            if segment == "constant" {
                "D=A"
            } else {
                "D=M"
            }
        };

        let assembly_code = format!(
            "@{}
{}
@SP
A=M
M=D
@SP
M=M+1\n\n",
            index, data_position
        );
        self.output_file.write(assembly_code.as_bytes()).unwrap();
    }

    fn pop(&mut self, segment: &str, index: &str) {
        let assembly_code = format!(
            "@SP
M=M-1
A=M
D=M
@{}.{}
M=D\n\n",
            segment, index
        );
        self.output_file.write(assembly_code.as_bytes()).unwrap();
    }

    fn arithmetic_add(&mut self) {
        let assembly_code = self.binary_function("M+D");
        self.output_file.write(assembly_code.as_bytes()).unwrap();
    }

    fn arithmetic_sub(&mut self) {
        let assembly_code = self.binary_function("M-D");
        self.output_file.write(assembly_code.as_bytes()).unwrap();
    }

    fn arithmetic_neg(&mut self) {
        let assembly_code = self.unary_function("-M");
        self.output_file.write(assembly_code.as_bytes()).unwrap();
    }

    fn arithmetic_eq(&mut self) {
        let assembly_code = self.compare_function("eq");
        self.output_file.write(assembly_code.as_bytes()).unwrap();
    }

    fn arithmetic_gt(&mut self) {
        let assembly_code = self.compare_function("gt");
        self.output_file.write(assembly_code.as_bytes()).unwrap();
    }

    fn arithmetic_lt(&mut self) {
        let assembly_code = self.compare_function("lt");
        self.output_file.write(assembly_code.as_bytes()).unwrap();
    }

    fn arithmetic_and(&mut self) {
        let assembly_code = self.binary_function("M&D");
        self.output_file.write(assembly_code.as_bytes()).unwrap();
    }

    fn arithmetic_or(&mut self) {
        let assembly_code = self.binary_function("M|D");
        self.output_file.write(assembly_code.as_bytes()).unwrap();
    }

    fn arithmetic_not(&mut self) {
        let assembly_code = self.unary_function("!M");
        self.output_file.write(assembly_code.as_bytes()).unwrap();
    }

    fn binary_function(&self, function: &str) -> String {
        format!(
            "@SP
M=M-1
A=M
D=M
@SP
M=M-1
A=M
M={}
@SP
M=M+1\n\n",
            function
        )
    }

    fn compare_function(&mut self, function: &str) -> String {
        let jmp: String = function.to_ascii_uppercase();
        self.jmp_point += 1;
        format!(
            "@SP
M=M-1
A=M
D=M
@SP
M=M-1
A=M
D=M-D
@jump_point{}
D;J{}
D=0
@jump_endpoint{}
0;JEQ
(jump_point{})
D=-1
(jump_endpoint{})
@SP
A=M
M=D
@SP
M=M+1\n\n",
            self.jmp_point, jmp, self.jmp_point, self.jmp_point, self.jmp_point
        )
    }

    fn unary_function(&self, function: &str) -> String {
        format!(
            "@SP
M=M-1
A=M
M={}
@SP
M=M+1\n\n",
            function
        )
    }
}
