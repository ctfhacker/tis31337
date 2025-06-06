
use std::env;
use std::fs;
use std::process;

#[derive(Debug)]
struct Emulator {
    acc: i32,
    bak: i32,
}

impl Emulator {
    fn new() -> Self {
        Emulator { acc: 0, bak: 0 }
    }

    fn execute_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return;
        }
        let mut parts = trimmed.split_whitespace();
        let instr = match parts.next() {
            Some(i) => i.to_ascii_lowercase(),
            None => return,
        };

        match instr.as_str() {
            "mov" => {
                // mov src, dst
                let src_dst: Vec<&str> = trimmed[3..].trim().split(',').map(|s| s.trim()).collect();
                if src_dst.len() != 2 {
                    eprintln!("Invalid mov syntax: {}", trimmed);
                    process::exit(1);
                }
                let src = src_dst[0];
                let dst = src_dst[1];
                let val = self.read_value(src);
                self.write_value(dst, val);
            }
            "swp" => {
                std::mem::swap(&mut self.acc, &mut self.bak);
            }
            "save" => {
                self.bak = self.acc;
            }
            "add" => {
                let arg = parts.collect::<Vec<&str>>().join(" ");
                let val = self.read_value(arg.trim());
                self.acc = Self::clamp(self.acc + val);
            }
            _ => {
                eprintln!("Unknown instruction: {}", instr);
                process::exit(1);
            }
        }
    }

    fn read_value(&self, src: &str) -> i32 {
        match src.to_ascii_lowercase().as_str() {
            "acc" => self.acc,
            "bak" => self.bak,
            _ => src.parse::<i32>().unwrap_or_else(|_| {
                eprintln!("Invalid source value: {}", src);
                process::exit(1);
            }),
        }
    }

    fn write_value(&mut self, dst: &str, val: i32) {
        match dst.to_ascii_lowercase().as_str() {
            "acc" => self.acc = Self::clamp(val),
            "bak" => {
                eprintln!("Cannot write directly to bak register");
                process::exit(1);
            },
            _ => {
                eprintln!("Invalid destination: {}", dst);
                process::exit(1);
            }
        }
    }

    fn clamp(val: i32) -> i32 {
        val.max(-999).min(999)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        process::exit(1);
    }
    let filename = &args[1];
    let contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Could not read file: {}", filename);
        process::exit(1);
    });

    let mut emu = Emulator::new();
    for (i, line) in contents.lines().enumerate() {
        emu.execute_line(line);
    }

    println!("Final emulator state:\nacc = {}\nbak = {}", emu.acc, emu.bak);
}
