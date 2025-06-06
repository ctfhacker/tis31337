

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;


#[derive(Debug)]
struct Emulator {
    acc: i32,
    bak: i32,
    pc: usize,
    labels: HashMap<String, usize>,
    program: Vec<Instruction>,
}

#[derive(Debug, Clone)]
enum Operand {
    Acc,
    Bak,
    Imm(i32),
    Label(String),
}

#[derive(Debug, Clone)]
enum Instruction {
    Mov(Operand, Operand),
    Swp,
    Save,
    Add(Operand),
    Jmp(String),
    Jez(String),
    Jnz(String),
    Jgz(String),
    Jlz(String),
    Ret,
    Label(String),
    Nop,
}

impl Emulator {
    fn new() -> Self {
        Emulator {
            acc: 0,
            bak: 0,
            pc: 0,
            labels: HashMap::new(),
            program: Vec::new(),
        }
    }

    #[inline(always)]
    fn clamp_acc(&mut self) {
        if self.acc > 999 {
            self.acc = 999;
        } else if self.acc < -999 {
            self.acc = -999;
        }
    }

    #[inline(always)]
    fn execute(&mut self, instr: &Instruction) -> Result<(), String> {
        match instr {
            Instruction::Mov(src, dst) => {
                let value = self.read_value(src)?;
                self.write_value(dst, value)?;
                self.pc += 1;
            }
            Instruction::Swp => {
                std::mem::swap(&mut self.acc, &mut self.bak);
                self.pc += 1;
            }
            Instruction::Save => {
                self.bak = self.acc;
                self.pc += 1;
            }
            Instruction::Add(src) => {
                let value = self.read_value(src)?;
                self.acc += value;
                self.clamp_acc();
                self.pc += 1;
            }
            Instruction::Jmp(label) => {
                if let Some(&target) = self.labels.get(label) {
                    self.pc = target;
                } else {
                    return Err(format!("Unknown label: {}", label));
                }
            }
            Instruction::Jez(label) => {
                if self.acc == 0 {
                    if let Some(&target) = self.labels.get(label) {
                        self.pc = target;
                    } else {
                        return Err(format!("Unknown label: {}", label));
                    }
                } else {
                    self.pc += 1;
                }
            }
            Instruction::Jnz(label) => {
                if self.acc != 0 {
                    if let Some(&target) = self.labels.get(label) {
                        self.pc = target;
                    } else {
                        return Err(format!("Unknown label: {}", label));
                    }
                } else {
                    self.pc += 1;
                }
            }
            Instruction::Jgz(label) => {
                if self.acc > 0 {
                    if let Some(&target) = self.labels.get(label) {
                        self.pc = target;
                    } else {
                        return Err(format!("Unknown label: {}", label));
                    }
                } else {
                    self.pc += 1;
                }
            }
            Instruction::Jlz(label) => {
                if self.acc < 0 {
                    if let Some(&target) = self.labels.get(label) {
                        self.pc = target;
                    } else {
                        return Err(format!("Unknown label: {}", label));
                    }
                } else {
                    self.pc += 1;
                }
            }
            Instruction::Ret => {
                self.pc = self.program.len();
            }
            Instruction::Label(_) => {
                self.pc += 1;
            }
            Instruction::Nop => {
                self.pc += 1;
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn read_value(&self, src: &Operand) -> Result<i32, String> {
        match src {
            Operand::Acc => Ok(self.acc),
            Operand::Bak => Ok(self.bak),
            Operand::Imm(v) => Ok(*v),
            Operand::Label(s) => Err(format!("Cannot use label '{}' as value", s)),
        }
    }

    #[inline(always)]
    fn write_value(&mut self, dst: &Operand, value: i32) -> Result<(), String> {
        match dst {
            Operand::Acc => {
                self.acc = value;
                self.clamp_acc();
                Ok(())
            }
            Operand::Bak => Err("Cannot write directly to bak".to_string()),
            Operand::Imm(_) => Err("Cannot write to immediate value".to_string()),
            Operand::Label(s) => Err(format!("Cannot write to label '{}" , s)),
        }
    }

    fn print_state(&self) {
        println!("acc: {}", self.acc);
        println!("bak: {}", self.bak);
    }

    fn parse_operand(token: &str) -> Operand {
        match token.to_lowercase().as_str() {
            "acc" => Operand::Acc,
            "bak" => Operand::Bak,
            _ => {
                if let Ok(v) = token.parse::<i32>() {
                    Operand::Imm(v)
                } else {
                    Operand::Label(token.to_string())
                }
            }
        }
    }

    fn parse_line(line: &str) -> Instruction {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Instruction::Nop;
        }
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.is_empty() {
            return Instruction::Nop;
        }
        match tokens[0].to_lowercase().as_str() {
            "mov" => {
                if tokens.len() == 3 {
                    let src = tokens[1].trim_end_matches(',');
                    let dst = tokens[2];
                    Instruction::Mov(Self::parse_operand(src), Self::parse_operand(dst))
                } else {
                    Instruction::Nop
                }
            }
            "swp" => Instruction::Swp,
            "save" => Instruction::Save,
            "add" => {
                if tokens.len() == 2 {
                    Instruction::Add(Self::parse_operand(tokens[1]))
                } else {
                    Instruction::Nop
                }
            }
            "jmp" => {
                if tokens.len() == 2 {
                    Instruction::Jmp(tokens[1].to_string())
                } else {
                    Instruction::Nop
                }
            }
            "jez" => {
                if tokens.len() == 2 {
                    Instruction::Jez(tokens[1].to_string())
                } else {
                    Instruction::Nop
                }
            }
            "jnz" => {
                if tokens.len() == 2 {
                    Instruction::Jnz(tokens[1].to_string())
                } else {
                    Instruction::Nop
                }
            }
            "jgz" => {
                if tokens.len() == 2 {
                    Instruction::Jgz(tokens[1].to_string())
                } else {
                    Instruction::Nop
                }
            }
            "jlz" => {
                if tokens.len() == 2 {
                    Instruction::Jlz(tokens[1].to_string())
                } else {
                    Instruction::Nop
                }
            }
            "ret" => Instruction::Ret,
            _ => {
                if tokens[0].ends_with(':') {
                    let label = tokens[0].trim_end_matches(':').to_string();
                    Instruction::Label(label)
                } else {
                    Instruction::Nop
                }
            }
        }
    }

    fn load_program(&mut self, lines: Vec<String>) {
        self.program.clear();
        self.labels.clear();
        for (idx, line) in lines.iter().enumerate() {
            let instr = Self::parse_line(line);
            if let Instruction::Label(ref label) = instr {
                self.labels.insert(label.clone(), idx);
            }
            self.program.push(instr);
        }
        self.pc = 0;
    }

    fn run(&mut self) {
        let prog_len = self.program.len();
        while self.pc < prog_len {
            let instr = &self.program[self.pc];
            if let Err(e) = self.execute(instr) {
                eprintln!("Error on line {}: {}", self.pc + 1, e);
                std::process::exit(1);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];
    let file = File::open(filename).unwrap_or_else(|_| {
        eprintln!("Could not open file: {}", filename);
        std::process::exit(1);
    });
    let reader = BufReader::new(file);
    let mut emu = Emulator::new();
    let lines: Vec<String> = reader
        .lines()
        .map(|l| l.unwrap_or_else(|_| String::new()))
        .collect();
    emu.load_program(lines);
    emu.run();
    emu.print_state();
}
