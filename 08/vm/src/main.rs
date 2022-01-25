use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;

static GENERIC_0: &'static str = "13";

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_filename = get_filename_from_arg(&args).unwrap();
    let output_file = get_output_filename(&input_filename);

    let mut parser = Parser::new(&input_filename).unwrap();
    let mut writer = CodeWriter::new(&output_file).unwrap();

    while parser.has_more_commands() {
        let command: &String = parser.get_command();
        writer.write_down(&format!("// {}\n", command));
        match parser.command_type() {
            CommandType::CPOP => writer.write_pushpop("pop", parser.arg1(), parser.arg2()),
            CommandType::CPUSH => writer.write_pushpop("push", parser.arg1(), parser.arg2()),
            CommandType::CARITHMETIC => writer.write_arithmetic(parser.arg1()),
            CommandType::CLABEL => writer.write_label(parser.arg1()),
            CommandType::CGOTO => writer.write_goto(parser.arg1()),
            CommandType::CIF => writer.write_if(parser.arg1()),
            _ => continue,
        };
        parser.advance();
    }
}

fn get_filename_from_arg(args: &[String]) -> Result<String, &str> {
    if args.len() < 2 {
        return Err("not enought argument");
    }
    let filename = args[1].clone();
    return Ok(filename);
}

fn get_output_filename(filename: &str) -> String {
    let filename = Path::new(filename).file_stem().unwrap().to_str().unwrap();
    let output_file = format!("{}.asm", filename);
    output_file
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

    pub fn get_command(&self) -> &String {
        &self.commands[self.command_pos]
    }

    pub fn command_type(&self) -> CommandType {
        let command = self.get_command();

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
        let command = self.get_command();
        let morpheme: Vec<&str> = command.split_whitespace().collect();

        match self.command_type() {
            CommandType::CRETURN => panic!("arg1 is called while comamnd type is 'return'"),
            CommandType::CARITHMETIC => morpheme[0],
            _ => morpheme[1],
        }
    }

    pub fn arg2(&self) -> &str {
        match self.command_type() {
            CommandType::CARITHMETIC => panic!("arg2 is called while command type is arithmetic"),
            CommandType::CLABEL => panic!("arg2 is called while command type is 'label'"),
            CommandType::CGOTO => panic!("arg2 is called while command type is 'goto'"),
            CommandType::CIF => panic!("arg2 is called while command type is 'if'"),
            CommandType::CRETURN => panic!("arg2 is called while command type is 'return'"),
            _ => {
                let command = &self.commands[self.command_pos];
                let morpheme: Vec<&str> = command.split_whitespace().collect();
                morpheme[2]
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
    return_num: i64,
}

impl CodeWriter {
    pub fn new(filename: &str) -> Result<CodeWriter, io::Error> {
        let writer = BufWriter::new(File::create(filename)?);
        Ok(CodeWriter {
            output_file: writer,
            jmp_point: 0,
            return_num: 0,
        })
    }

    pub fn write_down(&mut self, command: &str) {
        self.output_file.write(command.as_bytes()).unwrap();
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

    fn write_push_from_d_register(&mut self) {
        let assembly_code = "@SP\
                            \nA=M\
                            \nM=D\
                            \n@SP\
                            \nM=M+1\n";
        self.write_down(assembly_code);
    }

    fn write_pop_to_d_register(&mut self) {
        let assembly_code = "@SP\
                            \nM=M-1\
                            \nA=M\
                            \nD=M\n";
        self.write_down(assembly_code);
    }

    fn push(&mut self, segment: &str, index: &str) {
        let data_position = {
            if segment == "constant" {
                format!(
                    "@{}\
                    \nD=A",
                    index
                )
            } else if segment == "temp" {
                format!(
                    "@{}\
                    \nD=M",
                    5 + index.parse::<i32>().unwrap()
                )
            } else if segment == "pointer" {
                format!(
                    "@{}\
                    \nD=M",
                    3 + index.parse::<i32>().unwrap()
                )
            } else if segment == "static" {
                format!(
                    "@{}\
                    \nD=M",
                    index.parse::<i32>().unwrap() + 16
                )
            } else {
                let segment = match segment {
                    "local" => "LCL",
                    "argument" => "ARG",
                    "this" => "THIS",
                    "that" => "THAT",
                    "temp" => "5",
                    _ => panic!("segment not match "),
                };
                format!(
                    "@{}\
                    \nD=M\
                    \n@{}\
                    \nD=D+A\
                    \nA=D\
                    \nD=M",
                    segment, index
                )
            }
        };

        let assembly_code = format!(
            "{}\
            \n@SP\
            \nA=M\
            \nM=D\
            \n@SP\
            \nM=M+1\n\n",
            data_position
        );
        self.write_down(&assembly_code);
    }

    fn pop(&mut self, segment: &str, index: &str) {
        if segment == "temp" {
            let assembly_code = format!(
                "@SP\
                \nM=M-1\
                \nA=M\
                \nD=M\
                \n@{}\
                \nM=D\n\n",
                index.parse::<i32>().unwrap() + 5
            );
            self.write_down(&assembly_code);
            return;
        }

        if segment == "pointer" {
            let assembly_code = format!(
                "@SP\
                \nM=M-1\
                \nA=M\
                \nD=M\
                \n@{}\
                \nM=D\n\n",
                index.parse::<i32>().unwrap() + 3
            );
            self.write_down(&assembly_code);
            return;
        }

        if segment == "static" {
            let assembly_code = format!(
                "@SP\
                \nM=M-1\
                \nA=M\
                \nD=M\
                \n@{}\
                \nM=D\n\n",
                index.parse::<i32>().unwrap() + 16
            );
            self.write_down(&assembly_code);
            return;
        }

        let segment = match segment {
            "local" => "LCL",
            "argument" => "ARG",
            "this" => "THIS",
            "that" => "THAT",
            _ => panic!("segment not match "),
        };

        let assembly_code = format!(
            "@{0}\
            \nD=M\
            \n@{1}\
            \nD=D+A\
            \n@{2}\
            \nM=D\
            \n@SP\
            \nM=M-1\
            \nA=M\
            \nD=M\
            \n@{2}\
            \nA=M\
            \nM=D\n\n",
            segment, index, GENERIC_0
        );
        self.write_down(&assembly_code);
    }

    fn arithmetic_add(&mut self) {
        let assembly_code = self.binary_function("M+D");
        self.write_down(&assembly_code)
    }

    fn arithmetic_sub(&mut self) {
        let assembly_code = self.binary_function("M-D");
        self.write_down(&assembly_code)
    }

    fn arithmetic_neg(&mut self) {
        let assembly_code = self.unary_function("-M");
        self.write_down(&assembly_code)
    }

    fn arithmetic_eq(&mut self) {
        let assembly_code = self.compare_function("eq");
        self.write_down(&assembly_code)
    }

    fn arithmetic_gt(&mut self) {
        let assembly_code = self.compare_function("gt");
        self.write_down(&assembly_code)
    }

    fn arithmetic_lt(&mut self) {
        let assembly_code = self.compare_function("lt");
        self.write_down(&assembly_code)
    }

    fn arithmetic_and(&mut self) {
        let assembly_code = self.binary_function("M&D");
        self.write_down(&assembly_code)
    }

    fn arithmetic_or(&mut self) {
        let assembly_code = self.binary_function("M|D");
        self.write_down(&assembly_code)
    }

    fn arithmetic_not(&mut self) {
        let assembly_code = self.unary_function("!M");
        self.write_down(&assembly_code)
    }

    pub fn write_init() {}

    pub fn write_label(&mut self, label: &str) {
        let assembly_code = format!("({})\n\n", label);
        self.write_down(&assembly_code)
    }

    pub fn write_goto(&mut self, label: &str) {
        let assembly_code = format!(
            "@{}\
            \n0;JMP\
            \n\n",
            label
        );
        self.write_down(&assembly_code)
    }

    pub fn write_if(&mut self, label: &str) {
        self.write_pop_to_d_register();
        let assembly_code = format!(
            "@{}
            \nD;JNE\
            \n\n",
            label
        );
        self.write_down(&assembly_code)
    }

    pub fn write_call(&mut self, functionname: &str, numargs: &str) {
        // push return-address
        let return_label = format!("{}_{}", functionname, self.return_num);
        self.return_num += 1;
        let assembly_code = format!(
            "@{}\
            \nD=A\n",
            return_label
        );
        self.write_down(&assembly_code);
        // push LCL
        let assembly_code = "
            @LCL\
            \nD=M\n";
        self.write_down(assembly_code);
        self.write_push_from_d_register();
        // push ARG
        let assembly_code = "
            @ARG\
            \nD=M\n";
        self.write_down(assembly_code);
        self.write_push_from_d_register();
        // push THIS
        let assembly_code = "
            @THIS\
            \nD=M\n";
        self.write_down(assembly_code);
        self.write_push_from_d_register();
        // push THAT
        let assembly_code = "
            @THAT\
            \nD=M\n";
        self.write_down(assembly_code);
        self.write_push_from_d_register();
        // ARG = SP - n - 5
        let assembly_code = format!(
            "@SP\
            \nD=M\
            \n@5\
            \nD=D-A\
            \n@{}\
            \nD=D-A\
            \n@ARG\
            \nM=D\n",
            numargs
        );
        self.write_down(&assembly_code);
        // LCL = SP
        let assembly_code = "
            @SP\
            \nD=M\
            \n@LCL\
            \nM=D\n";
        self.write_down(assembly_code);
        // goto f
        let assembly_code = format!(
            "@{}\
            \n0;JMP\n",
            functionname
        );
        self.write_down(&assembly_code);
        // (return - address)
        let assembly_code = format!("({})\n\n", return_label);
        self.write_down(&assembly_code);
    }

    pub fn write_return(&mut self) {
        // FRAME = LCL
        let assembly_code = "
            @LCL\
            \nD=M\
            \n@R13\
            \nM=D\n";
        self.write_down(assembly_code);
        // RET = *(FRAME - 5)
        let assembly_code = "
            @5\
            \nD=A\
            \n@13\
            \nA=M-D\
            \nD=M\
            \n@14\
            \nM=D\n";
        self.write_down(assembly_code);
        // *ARG = pop(), SP = ARG + 1
        self.write_pop_to_d_register();
        let assembly_code = "
            @ARG\
            \nA=M\
            \nM=D\
            \n@ARG\
            \nD=M+1
            \n@SP\
            \nM=D\n";
        self.write_down(assembly_code);
        // THAT = *(FRAME - 1)
        let assembly_code = "
            @R13\
            \nM=M-1\
            \nD=M\
            \n@THAT\
            \nM=D\n";
        self.write_down(assembly_code);
        // THIS = * (FRAME - 2)
        let assembly_code = "
            @13\
            \nM=M-1\
            \nD=M\
            \n@THIS\
            \nM=D\n";
        self.write_down(assembly_code);
        // ARG = * (FRAME - 3)
        let assembly_code = "
            @13\
            \nM=M-1;
            \nD=M\
            \n@ARG\
            \nM=D\n";
        self.write_down(assembly_code);
        // LCL = * (FRAME - 4)
        let assembly_code = "
            @13\
            \nM=M-1\
            \nD=M\
            \n@LCL\
            \nM=D\n";
        self.write_down(assembly_code);
        // goto RET
        let assembly_code = "
            @14\
            \nA=M\
            \n0;JMP\n\n";
        self.write_down(assembly_code);
    }

    pub fn write_function(&mut self, function: &str, num_of_locals: &str) {
        // (f)
        let assembly = format!("({})\n", function);
        self.write_down(&assembly);
        // repeat k times: push0
        let assembly = "D=0\n";
        self.write_down(assembly);
        for _ in 0..num_of_locals.parse::<i32>().unwrap() {
            self.write_push_from_d_register();
        }
        let assembly = "\n\n";
        self.write_down(assembly);
    }

    fn binary_function(&self, function: &str) -> String {
        format!(
            "@SP\
            \nM=M-1\
            \nA=M\
            \nD=M\
            \n@SP\
            \nM=M-1\
            \nA=M\
            \nM={}\
            \n@SP\
            \nM=M+1\n\n",
            function
        )
    }

    fn compare_function(&mut self, function: &str) -> String {
        let jmp: String = function.to_ascii_uppercase();
        self.jmp_point += 1;
        format!(
            "@SP\
            \nM=M-1\
            \nA=M\
            \nD=M\
            \n@SP\
            \nM=M-1\
            \nA=M\
            \nD=M-D\
            \n@jump_point{}\
            \nD;J{}\
            \nD=0\
            \n@jump_endpoint{}\
            \n0;JEQ\
            \n(jump_point{})\
            \nD=-1\
            \n(jump_endpoint{})\
            \n@SP\
            \nA=M\
            \nM=D\
            \n@SP\
            \nM=M+1\n\n",
            self.jmp_point, jmp, self.jmp_point, self.jmp_point, self.jmp_point
        )
    }

    fn unary_function(&self, function: &str) -> String {
        format!(
            "@SP\
            \nM=M-1\
            \nA=M\
            \nM={}\
            \n@SP\
            \nM=M+1\n\n",
            function
        )
    }
}
