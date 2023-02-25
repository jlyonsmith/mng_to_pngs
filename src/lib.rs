mod log_macros;
mod mng;

use clap::Parser;
use core::fmt::Arguments;
use mng::{crc32, null_crc32, Chunk, MngFile};
use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

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
    output_dir: PathBuf,

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

        let mut chunks: Vec<Chunk> = vec![];

        MngFile::get_chunks(cli.input_file, &mut chunks)?;

        fs::create_dir_all(&cli.output_dir)?;

        let mut png_file: Option<fs::File> = None;
        let mut index = 0;

        for chunk in chunks.iter() {
            match chunk {
                Chunk::IHdr {
                    width,
                    height,
                    bit_depth,
                    color_type,
                    compression,
                    filter,
                    interlace,
                } => {
                    let mut file = File::create(cli.output_dir.join(format!("{:05}.png", index)))?;
                    let sig: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

                    file.write(&sig)?;
                    file.write(&13_u32.to_be_bytes())?;

                    let mut buf: Vec<u8> = Vec::with_capacity(4 + 13 + 4);

                    buf.write("IHDR".as_bytes())?;
                    buf.write(&width.to_be_bytes())?;
                    buf.write(&height.to_be_bytes())?;
                    buf.write(&[*bit_depth, *color_type, *compression, *filter, *interlace])?;

                    file.write(&buf)?;
                    file.write(&crc32(null_crc32(), &buf).to_be_bytes())?;

                    png_file = Some(file)
                }
                Chunk::IDat { data } => {
                    if let Some(ref mut file) = png_file {
                        file.write(&(data.len() as u32).to_be_bytes())?;

                        let mut buf: Vec<u8> = Vec::with_capacity(4);

                        buf.write("IDAT".as_bytes())?;

                        file.write(&buf)?;
                        file.write(&data)?;
                        file.write(&crc32(crc32(null_crc32(), &buf), &data).to_be_bytes())?;
                    }
                }
                Chunk::IEnd => {
                    if let Some(ref mut file) = png_file {
                        file.write(&0_u32.to_be_bytes())?;

                        let mut buf: Vec<u8> = Vec::with_capacity(4);

                        buf.write("IEND".as_bytes())?;

                        file.write(&buf)?;
                        file.write(&crc32(null_crc32(), &buf).to_be_bytes())?;

                        png_file = None;
                        index += 1;
                    }
                }
            }
        }

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
