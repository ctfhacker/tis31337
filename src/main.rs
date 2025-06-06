
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use std::collections::HashMap;

#[derive(Debug)]
struct Emulator {
    acc: i32,
    bak: i32,
    pc: usize,
    labels: HashMap<String, usize>,
    program: Vec<String>,
}

impl Emulator {
    fn new() -> Self {
        Emulator { acc: 0, bak: 0, pc: 0, labels: HashMap::new(), program: Vec::new() }
    }

    fn clamp_acc(&mut self) {
        if self.acc > 999 {
            self.acc = 999;
        } else if self.acc < -999 {
            self.acc = -999;
        }
    }

    fn execute(&mut self, line: &str) -> Result<(), String> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(());
        }
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.is_empty() {
            return Ok(());
        }
        match tokens[0].to_lowercase().as_str() {
            "mov" => {
                // mov src, dst
                if tokens.len() != 3 {
                    return Err(format!("Invalid mov syntax: {}", line));
                }
                let src = tokens[1].trim_end_matches(',');
                let dst = tokens[2];
                let value = self.read_value(src)?;
                self.write_value(dst, value)?;
            }
            "swp" => {
                if tokens.len() != 1 {
                    return Err(format!("Invalid swp syntax: {}", line));
                }
                std::mem::swap(&mut self.acc, &mut self.bak);
            }
            "save" => {
                if tokens.len() != 1 {
                    return Err(format!("Invalid save syntax: {}", line));
                }
                self.bak = self.acc;
            }
            "add" => {
                if tokens.len() != 2 {
                    return Err(format!("Invalid add syntax: {}", line));
                }
                let value = self.read_value(tokens[1])?;
                self.acc += value;
                self.clamp_acc();
            }
            // New opcodes for control flow
            "jmp" => {
                if tokens.len() != 2 {
                    return Err(format!("Invalid jmp syntax: {}", line));
                }
                let label = tokens[1];
                if let Some(&target) = self.labels.get(label) {
                    self.pc = target;
                    return Ok(()); // skip pc increment in run loop
                } else {
                    return Err(format!("Unknown label: {}", label));
                }
            }
            "jez" => { // jump if acc == 0
                if tokens.len() != 2 {
                    return Err(format!("Invalid jez syntax: {}", line));
                }
                let label = tokens[1];
                if self.acc == 0 {
                    if let Some(&target) = self.labels.get(label) {
                        self.pc = target;
                        return Ok(());
                    } else {
                        return Err(format!("Unknown label: {}", label));
                    }
                }
            }
            "jnz" => { // jump if acc != 0
                if tokens.len() != 2 {
                    return Err(format!("Invalid jnz syntax: {}", line));
                }
                let label = tokens[1];
                if self.acc != 0 {
                    if let Some(&target) = self.labels.get(label) {
                        self.pc = target;
                        return Ok(());
                    } else {
                        return Err(format!("Unknown label: {}", label));
                    }
                }
            }
            "jgz" => { // jump if acc > 0
                if tokens.len() != 2 {
                    return Err(format!("Invalid jgz syntax: {}", line));
                }
                let label = tokens[1];
                if self.acc > 0 {
                    if let Some(&target) = self.labels.get(label) {
                        self.pc = target;
                        return Ok(());
                    } else {
                        return Err(format!("Unknown label: {}", label));
                    }
                }
            }
            "jlz" => { // jump if acc < 0
                if tokens.len() != 2 {
                    return Err(format!("Invalid jlz syntax: {}", line));
                }
                let label = tokens[1];
                if self.acc < 0 {
                    if let Some(&target) = self.labels.get(label) {
                        self.pc = target;
                        return Ok(());
                    } else {
                        return Err(format!("Unknown label: {}", label));
                    }
                }
            }
            "ret" => {
                // For now, ret just ends the program
                self.pc = self.program.len();
                return Ok(());
            }
            _ => {
                if tokens[0].ends_with(':') {
                    // Label, ignore
                    return Ok(());
                }
                return Err(format!("Unknown instruction: {}", tokens[0]));
            }
        }
        self.pc += 1;
        Ok(())
    }

    fn read_value(&self, src: &str) -> Result<i32, String> {
        match src.to_lowercase().as_str() {
            "acc" => Ok(self.acc),
            "bak" => Ok(self.bak),
            _ => {
                // Try to parse as integer
                src.parse::<i32>().map_err(|_| format!("Invalid source: {}", src))
            }
        }
    }

    fn write_value(&mut self, dst: &str, value: i32) -> Result<(), String> {
        match dst.to_lowercase().as_str() {
            "acc" => {
                self.acc = value;
                self.clamp_acc();
                Ok(())
            }
            // bak cannot be written to directly
            "bak" => Err("Cannot write directly to bak".to_string()),
            _ => Err(format!("Invalid destination: {}", dst)),
        }
    }

    fn print_state(&self) {
        println!("acc: {}", self.acc);
        println!("bak: {}", self.bak);
    }
    // Load program and collect labels
    fn load_program(&mut self, lines: Vec<String>) {
        self.program = lines;
        self.labels.clear();
        for (idx, line) in self.program.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.ends_with(':') {
                let label = trimmed.trim_end_matches(':').to_string();
                self.labels.insert(label, idx);
            }
        }
        self.pc = 0;
    }
    // Run the loaded program
    fn run(&mut self) {
        while self.pc < self.program.len() {
            let line = &self.program[self.pc];
            if let Err(e) = self.execute(line) {
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

    // Examples of control flow usage:
    println!("\n--- Control Flow Examples ---");
    println!("\nExample 1: Loop 5 times\n");
    println!("mov 5, acc\nloop: add -1\njnz loop\nret\n");
    println!("\nExample 2: If acc is zero, jump to label\n");
    println!("mov 0, acc\njez is_zero\nmov 1, acc\nret\nis_zero: mov 42, acc\nret\n");
    println!("\nExample 3: Jump forward and return\n");
    println!("mov 10, acc\njmp end\nmov 99, acc\nend: ret\n");
}
