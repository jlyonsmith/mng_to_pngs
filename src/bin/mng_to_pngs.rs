use colored::Colorize;
use core::fmt::Arguments;
use mng_to_pngs::{error, MngToPngLog, MngToPngTool};

struct MngToPngLogger;

impl MngToPngLogger {
    fn new() -> MngToPngLogger {
        MngToPngLogger {}
    }
}

impl MngToPngLog for MngToPngLogger {
    fn output(self: &Self, args: Arguments) {
        println!("{}", args);
    }
    fn warning(self: &Self, args: Arguments) {
        eprintln!("{}", format!("warning: {}", args).yellow());
    }
    fn error(self: &Self, args: Arguments) {
        eprintln!("{}", format!("error: {}", args).red());
    }
}

fn main() {
    let logger = MngToPngLogger::new();

    if let Err(error) = MngToPngTool::new(&logger).run(std::env::args_os()) {
        error!(logger, "{}", error);
        std::process::exit(1);
    }
}
