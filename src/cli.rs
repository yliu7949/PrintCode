use std::num::NonZeroUsize;
use std::path::PathBuf;

use clap::{Arg, Command};

const ABOUT: &str = concat!(
    "Generates a PDF from code files with pagination ",
    "and custom headers."
);

pub struct CliConfig {
    pub font_dir: PathBuf,
    pub font_name: String,
    pub code_folder: PathBuf,
    pub verbose: bool,
    pub code_name: String,
    pub code_version: String,
    pub output_path: PathBuf,
    pub limit_pages: bool,
    pub lines_per_page: usize,
}

impl CliConfig {
    pub fn font_path(&self) -> PathBuf {
        self.font_dir.join(&self.font_name)
    }
}

pub fn parse_args() -> CliConfig {
    let matches = Command::new("printcode")
        .version(env!("CARGO_PKG_VERSION"))
        .author("yliu7949")
        .about(ABOUT)
        .arg(
            Arg::new("font-dir")
                .short('f')
                .long("font-dir")
                .value_name("FONT_DIR")
                .help("Directory where the font files are located")
                .default_value(default_font_dir())
                .num_args(1),
        )
        .arg(
            Arg::new("font-name")
                .short('t')
                .long("font-name")
                .value_name("FONT_NAME")
                .help("Name of the font file to use")
                .default_value(default_font_name())
                .num_args(1),
        )
        .arg(
            Arg::new("code-folder")
                .short('d')
                .long("code-folder")
                .value_name("CODE_FOLDER")
                .help("Directory containing code files")
                .required(true)
                .num_args(1),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .help("Print detailed information")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("code-name")
                .short('n')
                .long("code-name")
                .value_name("CODE_NAME")
                .help("Code name for the PDF document")
                .required(true)
                .num_args(1),
        )
        .arg(
            Arg::new("code-version")
                .short('v')
                .long("code-version")
                .value_name("CODE_VERSION")
                .help("Code version for the PDF document")
                .default_value("V1.0.0")
                .num_args(1),
        )
        .arg(
            Arg::new("output-path")
                .short('o')
                .long("output-path")
                .value_name("OUTPUT_FILE")
                .help("Path to the output PDF document")
                .default_value("output.pdf")
                .num_args(1),
        )
        .arg(
            Arg::new("limit-pages")
                .long("limit-pages")
                .short('l')
                .help(
                    "Limit the PDF to first 30 and last 30 pages \
                    if total exceeds 60 pages",
                )
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("lines-per-page")
                .long("lines-per-page")
                .value_name("LINES")
                .help("Number of code lines written on each PDF page")
                .default_value("50")
                .value_parser(clap::value_parser!(NonZeroUsize))
                .num_args(1),
        )
        .get_matches();

    CliConfig {
        verbose: matches.get_flag("verbose"),
        font_dir: PathBuf::from(matches.get_one::<String>("font-dir").unwrap()),
        font_name: matches.get_one::<String>("font-name").unwrap().to_owned(),
        code_folder: PathBuf::from(matches.get_one::<String>("code-folder").unwrap()),
        code_name: matches.get_one::<String>("code-name").unwrap().to_owned(),
        code_version: matches
            .get_one::<String>("code-version")
            .unwrap()
            .to_owned(),
        output_path: PathBuf::from(matches.get_one::<String>("output-path").unwrap()),
        limit_pages: matches.get_flag("limit-pages"),
        lines_per_page: matches
            .get_one::<NonZeroUsize>("lines-per-page")
            .unwrap()
            .get(),
    }
}

#[cfg(target_os = "windows")]
fn default_font_dir() -> &'static str {
    "C:/Windows/Fonts"
}

#[cfg(target_os = "windows")]
fn default_font_name() -> &'static str {
    "simsun.ttc"
}

#[cfg(target_os = "macos")]
fn default_font_dir() -> &'static str {
    "/System/Library/Fonts"
}

#[cfg(target_os = "macos")]
fn default_font_name() -> &'static str {
    "SFNSMono.ttf"
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn default_font_dir() -> &'static str {
    "/usr/share/fonts"
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn default_font_name() -> &'static str {
    "truetype/dejavu/DejaVuSansMono.ttf"
}
