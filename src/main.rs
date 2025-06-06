
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug)]
struct Emulator {
    acc: i32,
    bak: i32,
}

impl Emulator {
    fn new() -> Self {
        Emulator { acc: 0, bak: 0 }
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
            _ => {
                return Err(format!("Unknown instruction: {}", tokens[0]));
            }
        }
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
    for (lineno, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading line {}: {}", lineno + 1, e);
                std::process::exit(1);
            }
        };
        if let Err(e) = emu.execute(&line) {
            eprintln!("Error on line {}: {}", lineno + 1, e);
            std::process::exit(1);
        }
    }
    emu.print_state();
}
