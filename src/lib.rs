mod log_macros;
mod mng;

use clap::Parser;
use core::fmt::Arguments;
use mng::{MngError, MngFile};
use std::{error::Error, path::PathBuf};

pub trait MngToPngLog {
    fn output(self: &Self, args: Arguments);
    fn warning(self: &Self, args: Arguments);
    fn error(self: &Self, args: Arguments);
}

pub struct MngToPngTool<'a> {
    log: &'a dyn MngToPngLog,
}

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// The MNG file
    #[arg(value_name = "MNG_FILE")]
    input_file: PathBuf,

    /// The output directory
    #[arg(value_name = "OUTPUT_DIRECTORY")]
    output_file: PathBuf,

    /// Optional output file prefix
    #[arg(value_name = "PREFIX")]
    prefix: Option<String>,
}

impl<'a> MngToPngTool<'a> {
    pub fn new(log: &'a dyn MngToPngLog) -> MngToPngTool {
        MngToPngTool { log }
    }

    pub fn run(
        self: &mut Self,
        args: impl IntoIterator<Item = std::ffi::OsString>,
    ) -> Result<(), Box<dyn Error>> {
        let cli = match Cli::try_parse_from(args) {
            Ok(m) => m,
            Err(err) => {
                output!(self.log, "{}", err.to_string());
                return Ok(());
            }
        };

        let mng_file = MngFile::open();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        struct TestLogger;

        impl TestLogger {
            fn new() -> TestLogger {
                TestLogger {}
            }
        }

        impl MngToPngLog for TestLogger {
            fn output(self: &Self, _args: Arguments) {}
            fn warning(self: &Self, _args: Arguments) {}
            fn error(self: &Self, _args: Arguments) {}
        }

        let logger = TestLogger::new();
        let mut tool = MngToPngTool::new(&logger);
        let args: Vec<std::ffi::OsString> = vec!["".into(), "--help".into()];

        tool.run(args).unwrap();
    }
}
