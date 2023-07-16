use std::{fs, collections::HashMap};

use clap::Parser;
use phf::phf_map;

#[derive(Parser, Debug)]
struct CLI{
    file_location: String,
    #[arg(short, long)]
    output_file: Option<String>
}
#[derive(Clone)]
enum InstructionTypes{
    MathSingles = 0b0000_0001_1100_0000,
    MathDoubles = 0b0000_0000_0000_0000,
    Moves = 0b0000_0010_0000_0000,
    RamMoves = 0b0000_0010_1000_0000,
    Stack = 0b0000_0011_0000_0000,
    Jump = 0b0000_1000_0000_0000,
}
#[derive(Debug)]
struct Macro<'a>{
    params: Vec<&'a str>,
    code: Vec<Instruction<'a>>,
    line: usize,
    length: usize
}

#[derive(Debug, Clone)]
struct Instruction<'a>{
    _x: bool,
    negative: bool,
    zero: bool,
    update: bool,
    opcode: &'a str,
    args: Vec<String>,
    line: usize,
    binary_line: usize
}
impl<'a> Instruction<'a>{
    const INSTRUCTIONS: phf::Map<&'static str, (InstructionTypes, u16)> = phf_map!(
        "mov" => (InstructionTypes::Moves, 0),
        "str" => (InstructionTypes::RamMoves, 0),
        "load" => (InstructionTypes::RamMoves, 1),

        "incr" => (InstructionTypes::MathSingles, 0),
        "decr" => (InstructionTypes::MathSingles, 1),
        "not" => (InstructionTypes::MathSingles, 2),

        "add" => (InstructionTypes::MathDoubles, 0),
        "sub" => (InstructionTypes::MathDoubles, 1),
        "mul" => (InstructionTypes::MathDoubles, 2),
        "and" => (InstructionTypes::MathDoubles, 3),
        "or" =>  (InstructionTypes::MathDoubles, 4),
        "xor" => (InstructionTypes::MathDoubles, 5),
        "cmp" => (InstructionTypes::MathDoubles, 6),

        "push" => (InstructionTypes::Stack, 0),
        "pop" => (InstructionTypes::Stack, 1),

        "j" => (InstructionTypes::Jump, 0),

    );
    fn init(line_num: usize, bin_line: usize, line: &'a str) -> Result<Self, String>{
        let mut x = line.split(" ");
        let instruction = match x.next(){
            Some(x) => x,
            None => return Err(format!("error on line {line_num}")),
        };
        let temp_args: Vec<&str> = x.collect();
        let temp_args = temp_args.join("");

        let temp_args: Vec<&str> = temp_args.split(",").collect();

        let mut to_return: Instruction<'a> = Self{
            opcode: instruction,
            _x: false,
            negative: false,
            zero: false,
            update: false,
            args: temp_args.into_iter().map(|w| w.to_owned()).collect(),
            line: line_num+1,
            binary_line: bin_line
        };

        for ins in Instruction::INSTRUCTIONS.keys(){
            if instruction.starts_with(ins) {
                let instr = instruction.split(ins).nth(1);
                if let Some(x) = instr {
                    if x.contains("s"){
                        to_return.update = true;
                    }
                    if x.contains("z") || x.contains("eq"){
                        to_return.zero = true;
                    }
                    if x.contains("n") || x.contains("lt"){
                        to_return.negative = true;
                    }
                    if x.is_empty() {
                        to_return.opcode = instruction;
                    } else {
                        to_return.opcode = instruction.split(x).next().expect("wtf");
                    }
                }
            }
        }
        //println!("{:?}", to_return);
        Ok(to_return)

    }
    fn reg_to_int(register: &str) -> u16{
        match register{
            "r0" => 0,
            "r1" => 1,
            "r2" => 2,
            "r3" => 3,
            _ => 0b111
        }
    }
    fn repr(&self) -> Result<Vec<u16>, String> {
        let (instr, opcode) = match Self::INSTRUCTIONS.get(self.opcode){
            Some(x) => x.to_owned(),
            None => return Err(format!("{}: invalid opcode\n->{}\n{:?}", self.line, self.opcode, self)),
        };
        let mut return_instruction = instr.clone() as u16;


        //add the flags
        let flags: u16 = (self.negative as u16) << 2 | (self.zero as u16) << 1 | (self.update as u16);
        return_instruction |= flags << 12;

        let mut to_return: Vec<u16> = Vec::new();
        match instr {
            InstructionTypes::Moves => {
                //xxxx 0010 0CCD DEEE
                //c = opcode
                //d = arg 1
                //e = arg 2
                return_instruction |= opcode << 5;
                if self.args.len() != 2 {
                    return Err(format!("{}: not enough arguments\n{}", self.line, self.opcode));
                }
                let arg1 = Instruction::reg_to_int(&self.args[0]);
                if arg1 == 0b111 {
                    return Err(format!("{}: {} invalid first argument \"{}\"", self.line, self.opcode, self.args[0]))
                }
                return_instruction |= arg1 << 3;
                let x = Instruction::reg_to_int(&self.args[1]);
                return_instruction |= x;
                to_return.push(return_instruction);
                
                //value comes in next 16 bits

                if x == 0b111{
                    let second = self.args[1].strip_prefix("#");
                    let second = match second {
                        Some(x) => x,
                        None => return Err(format!("{}: invalid second argument", self.line))
                    };
                    let second = second.parse::<i16>();
                    let second = match second {
                        Ok(x) => x,
                        Err(_) => return Err(format!("{}: invalid value after #", self.line))
                    };
                    to_return.push(second as u16);
                }
            },
            InstructionTypes::MathSingles => {
                //xxxx 0001 11cc ceee
                //c = opcode
                //e = argument
                return_instruction |= opcode << 3;
                if self.args.len() != 1 {
                    return Err(format!("{}: argument error\n{}", self.line, self.opcode));
                }
                let instr = Instruction::reg_to_int(&self.args[0]);
                return_instruction |= instr;
                to_return.push(return_instruction);

                if instr == 0b111 {
                    let second = self.args[1].strip_prefix("#");
                    let second = match second {
                        Some(x) => x,
                        None => return Err(format!("{}: invalid second argument", self.line))
                    };
                    let second = second.parse::<i16>();
                    let second = match second {
                        Ok(x) => x,
                        Err(_) => return Err(format!("{}: invalid value after #", self.line))
                    };
                    to_return.push(second as u16);
                }
            },
            InstructionTypes::MathDoubles => {
                //xxxx 000c cccd deee
                //c opcode
                //d register
                //e register or if all 1 next byte as imm
                return_instruction |= opcode << 5;
                if self.args.len() != 2 {
                    return Err(format!("{}: not enough arguments\n{}", self.line, self.opcode));
                }
                return_instruction |= Instruction::reg_to_int(&self.args[0]) << 3;
                let x = Instruction::reg_to_int(&self.args[1]);
                return_instruction |= x;
                to_return.push(return_instruction);
                
                //value comes in next 16 bits

                if x == 0b111{
                    let second = self.args[1].strip_prefix("#");
                    let second = match second {
                        Some(x) => x,
                        None => return Err(format!("{}: invalid second argument", self.line))
                    };
                    let second = second.parse::<i16>();
                    let second = match second {
                        Ok(x) => x,
                        Err(_) => return Err(format!("{}: invalid value after #", self.line))
                    };
                    to_return.push(second as u16);
                }
            },
            InstructionTypes::RamMoves => {
                return_instruction |= opcode << 5;
                if self.args.len() != 2 {
                    return Err(format!("{}: not enough arguments\n{}", self.line, self.opcode));
                }
                return_instruction |= Instruction::reg_to_int(&self.args[0]) << 3;
                let x = Instruction::reg_to_int(&self.args[1]);
                return_instruction |= x;
                to_return.push(return_instruction);
                
                //value comes in next 16 bits

                if x == 0b111{
                    let second = self.args[1].strip_prefix("[").map(|x| x.strip_suffix("]"));
                    let second = match second {
                        Some(Some(x)) => x,
                        _ => return Err(format!("{}: invalid second argument", self.line))
                    };
                    let second = second.parse::<i16>();
                    let second = match second {
                        Ok(x) => x,
                        Err(_) => return Err(format!("{}: invalid value inside []", self.line))
                    };
                    to_return.push(second as u16);
                }

            },
            InstructionTypes::Stack => {
                return_instruction |= opcode << 4;
                if self.args.len() != 1 {
                    return Err(format!("{}: arugment error\n{}", self.line, self.opcode));
                }
                let instr = Instruction::reg_to_int(&self.args[0]);
                return_instruction |= instr;
                to_return.push(return_instruction);

                if instr == 0b111 {
                    let second = self.args[1].strip_prefix("#");
                    let second = match second {
                        Some(x) => x,
                        None => return Err(format!("{}: invalid second argument", self.line))
                    };
                    let second = second.parse::<i16>();
                    let second = match second {
                        Ok(x) => x,
                        Err(_) => return Err(format!("{}: invalid value after #", self.line))
                    };
                    to_return.push(second as u16);
                }
            },
            InstructionTypes::Jump => {
                if self.args.len() != 1 {
                    return Err(format!("{}: argument error\n{}", self.line, self.opcode));
                }
                //needs beter working
                let second = self.args[0].parse::<u16>();
                let second = match second {
                    Ok(x) => x,
                    Err(_) => return Err(format!("{}: invalid second argument", self.line)),
                };
                to_return.push(return_instruction);
                to_return.push(second);
                
            }
        }
        
        //add the registers or specify if a byte should be added afterwards

        return Ok(to_return);
    }
}

struct AsmParser<'a>{
    is_comment: bool,
    current_macro: Option<(&'a str, Macro<'a>)>,

    macro_list: HashMap<&'a str, Macro<'a>>,
    outLine: i64
}

impl<'a> AsmParser<'a>{
    
    fn new() -> Self{
        Self{
            is_comment: false,
            current_macro: None,
            macro_list: HashMap::new(),
            outLine: -1
        }
    }
    fn parse(&mut self, content: &'a str) -> Result<String, String>{
        let mut output = String::new();
        for (line_num,line) in content.lines().enumerate() {
            let r = self.parse_line(line_num, line);
            match r {
                Err(x) => return Err(x),
                Ok(Some(data)) => {
                    //output += &format!("{}:\t", self.outLine);
                    output += &data;
                    output += "\n";
                },
                _ => ()
            }
        }
        Ok(output)
    }
    fn parse_line(&mut self, line_num:usize, line: &'a str) -> Result<Option<String>, String>{
        //check if starts with ; for comments or ;= for multi line comments
        //ends a multiline comment and adds the code after it for parsing
        let line = match line.find("=;") {
            Some(_) => {
                self.is_comment = false;
                //println!("comment ended");
                line.split("=;").nth(1).unwrap()
            },
            None => {
                if self.is_comment {
                    return Ok(None);
                }
                line
            },
        };
        //starts a multiline comment and add the code before it for parsing
        let line = match line.find(";="){
            Some(_) => {
                self.is_comment = true;
                //println!("comment started");
                line.split(";=").next().unwrap()
            },
            None => line,
        };
        //parse a single line comment and add code before it for parsing
        let line = match line.find(";"){
            Some(_) => line.split(";").next().unwrap(),
            None => line,
        };
        let line = line.trim();

        if line.ends_with(":"){
            let ind = line.find(":").unwrap();
            let name = line.get(0..ind).unwrap();
            let realind = self.outLine;
            return Ok(Some(format!("({name}): {realind}")));
        }

        if line.starts_with("@macro"){
            //new macro defenition
            let (start_of_params, end_of_params) = match (line.find("("), line.find(")")) {
                (Some(a), Some(b)) => (a+1,b),
                _ => {
                    return Err(format!("{}: invalid macro definition", line_num));
                }
            };
            let macro_name = line.get(0..start_of_params-1).unwrap().strip_prefix("@macro ").unwrap().trim();
            //println!("funcname = {:?}", macro_name);
            let args = line.get(start_of_params..end_of_params).unwrap();
            let args: Vec<&str> = args.split(",").map(|f| f.trim()).collect();
            self.current_macro = Some(
                (macro_name,
                Macro{
                    params: args,
                    code: Vec::new(),
                    line: line_num,
                    length: 0
                })
            );
            return Ok(None);
        }
        if line == "}" && self.current_macro.is_some(){
            let (macro_name, c_macro) = self.current_macro.take().unwrap();
            println!("ending macro {} with length of {}", macro_name, c_macro.length);
            self.macro_list.insert(macro_name, c_macro);
            //self.macro_list.push(self.current_macro.take().unwrap());
            return Ok(None);
        } else if line == "}" {
            return Err(format!("{}: ending macro but no macro was started", line_num+1));
        }

        if line.is_empty(){
            return Ok(None);
        }
        
        println!("editor: {}, real:{}", line_num, self.outLine);


        if line.starts_with("@"){
            let (func_name, args) = match (line.find("("), line.find(")")){
                (Some(a), Some(b)) => (line.get(1..a).unwrap(), line.get(a+1..b).unwrap()),
                _ => return Err(format!("{}: error in macro call",line_num+1)),
            };
            //println!("in call {}", func_name);
            let args: Vec<&str> = args.split(",").map(|l| l.trim()).collect();


            //println!(";call of macro {} {:?}", func_name, args);
            let macro_to_add = self.macro_list.get(func_name);
            if macro_to_add.is_none() {
                return Err(format!("{}: macro {} does not exist", line_num+1, func_name));
            }
            let macro_to_add = macro_to_add.unwrap();
            
            //check if amount of args given is same as required
            if args.len() != macro_to_add.params.len() {
                return Err(format!("{}: macro {} does not have same amount of args as the definition at {}", line_num+1, func_name, macro_to_add.line+1));
            }

            //println!("macro = {:?}", macro_to_add);
            //println!("args = {:?}", args);
            //println!("a = {:?}", macro_to_add.params);
            let mut out = String::new();
            for mut instr in macro_to_add.code.clone() {
                //println!("args = {:?}, params = {:?}", args, instr.args);
                
                for (index, arg) in macro_to_add.params.iter().enumerate(){
                    for arg_to_replace in instr.args.iter_mut(){
                        if arg_to_replace == arg {
                            *arg_to_replace = args[index].to_string();
                        }
                    }
                }
                /*for (index, arg) in instr.args.iter_mut().enumerate() {
                    if macro_to_add.params.iter().any(|ell| ell == arg) {
                        *arg = args[index].to_string();
                    }
                }*/
                //println!("-----------------\n{:?}", instr);
                let res = instr.repr();
                let res = match res {
                    Ok(a) => a,
                    Err(err_line) => {
                        return Err(format!("{}: in macro expansion {}\n\t{}", line_num+1, func_name, err_line));
                    },
                };
                out += &format!("{}:", instr.binary_line);
                for ell in res {
                    out += &format!("{:#x}\t", ell);
                }
                out += "\n";
            }
            return Ok(Some(out));
        };
        self.outLine += 1;
        

        let instruction = Instruction::init(line_num, self.outLine as usize, line);



        //we are currently in a macro definition
        if let Some((_name, x)) = &mut self.current_macro {
            //macros need special compiler errors so handle it differently

            match instruction {
                Ok(ins) => {
                    x.length += 1;
                    x.code.push(ins);
                    return Ok(None);
                },
                Err(line) => {
                    return Err(format!("error in macro definiton {}", line));
                },
            }
        }
        self.outLine += 1;
        let res = instruction?.repr()?;


        let mut out = String::new();

        for ell in res {
            out += &format!("{:#x}\t", ell);
        }

        Ok(Some(out))
    }
}

fn main(){
    let data = CLI::parse();
    let content = match fs::read_to_string(&data.file_location){
        Ok(x) => x,
        Err(_) => {
            println!("file not found");
            return;
        },
    };
    let mut parser = AsmParser::new();
    let result = parser.parse(&content);
    
    let output_file = data.output_file.unwrap_or(data.file_location + ".o");

    match result{
        Err(x) => println!("{x}"),
        Ok(x) => fs::write(output_file, x).expect("error writing to file"),
    };
}
