use clap::{Arg, Command};
use printpdf::*;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use textwrap::{fill, Options};
use walkdir::WalkDir;

fn main() {
    let matches = Command::new("printcode")
        .version("0.1.0")
        .author("Your Name")
        .about("Generates a PDF from code files with pagination and custom headers.")
        .arg(
            Arg::new("font-dir")
                .short('f')
                .long("font-dir")
                .value_name("FONT_DIR")
                .help("Directory where the font files are located")
                .required(true)
                .num_args(1),
        )
        .arg(
            Arg::new("font-name")
                .short('t')
                .long("font-name")
                .value_name("FONT_NAME")
                .help("Name of the font file to use")
                .required(true)
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
                .required(true)
                .num_args(1),
        )
        .get_matches();

    let font_dir = matches.get_one::<String>("font-dir").unwrap();
    let font_name = matches.get_one::<String>("font-name").unwrap();
    let code_folder = matches.get_one::<String>("code-folder").unwrap();
    let verbose = matches.get_flag("verbose");
    let code_name = matches.get_one::<String>("code-name").unwrap();
    let code_version = matches.get_one::<String>("code-version").unwrap();

    let (doc, _page1, _layer1) = PdfDocument::new("Code Document", Mm(210.0), Mm(297.0), "Layer 1");
    let font_path = format!("{}/{}", font_dir, font_name);
    let font_file = File::open(font_path).expect("Failed to open font file");
    let font = doc.add_external_font(font_file).expect("Failed to add font");

    let lines_per_page = 50;
    let mut page_number = 1;

    for entry in WalkDir::new(code_folder).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            let file = File::open(entry.path()).expect("Failed to open code file");
            let reader = BufReader::new(file);

            let mut lines = Vec::new();
            let mut last_line_empty = false;
            for line in reader.lines() {
                let line = line.expect("Failed to read line");
                if line.trim().is_empty() {
                    if !last_line_empty {
                        lines.push(line);
                        last_line_empty = true;
                    }
                } else {
                    lines.push(line);
                    last_line_empty = false;
                }
            }

            let mut index = 0;
            let mut line_index = 1;

            while index < lines.len() {
                let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                let current_layer = doc.get_page(page).get_layer(layer);

                // 添加页眉文字
                let header = format!("{} {}", code_name, code_version);
                current_layer.use_text(&header, 10.0, Mm(78.0), Mm(278.5), &font);

                // 绘制下划线
                let line = Line {
                    points: vec![
                        (Point::new(Mm(20.0), Mm(277.0)), false),
                        (Point::new(Mm(184.0), Mm(277.0)), false),
                    ],
                    is_closed: false,
                };
                current_layer.set_outline_thickness(1.2);
                current_layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
                current_layer.add_line(line);

                // 添加页码
                current_layer.use_text(format!("{}", page_number), 11.0, Mm(189.0), Mm(276.5), &font);

                // 添加行号和代码
                let mut y = 272.0;
                let mut page_lines_count = 0;

                while page_lines_count < lines_per_page && index < lines.len() {
                    let line = &lines[index];
                    let wrapped_line = fill(line, Options::new(90).subsequent_indent("    "));
                    for wrapped_line in wrapped_line.lines() {
                        if page_lines_count >= lines_per_page {
                            break;
                        }
                        current_layer.use_text(
                            format!("{:>4}    {}", line_index, wrapped_line), // 行号后面隔4个空格
                            11.0, // 字号
                            Mm(6.0),
                            Mm(y),
                            &font,
                        );
                        y -= 5.4;
                        line_index += 1;
                        page_lines_count += 1;
                    }
                    index += 1;
                }

                page_number += 1;
                line_index = 1; // Reset line number for the next page
            }
        }
    }

    let output = File::create("output.pdf").expect("Failed to create output file");
    doc.save(&mut BufWriter::new(output)).expect("Failed to save PDF document");

    if verbose {
        println!("PDF document generated successfully.");
    }
}
