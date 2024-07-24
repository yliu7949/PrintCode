use printpdf::*;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use textwrap::{fill, Options};
use walkdir::WalkDir;

fn main() {
    let (doc, _page1, _layer1) = PdfDocument::new("Code Document", Mm(210.0), Mm(297.0), "Layer 1");
    let font_file = File::open("C://Windows/Fonts/simsun.ttc").unwrap();
    let font = doc.add_external_font(font_file).unwrap();

    let software_name = "微光萌生视频处理软件";
    let version = "V0.7.1";
    let lines_per_page = 50;
    let mut page_number = 1;

    for entry in WalkDir::new("code_folder").into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            let file = File::open(entry.path()).unwrap();
            let reader = BufReader::new(file);

            let mut lines = Vec::new();
            let mut last_line_empty = false;
            for line in reader.lines() {
                let line = line.unwrap();
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
                let header = format!("{} {}", software_name, version);
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

    let output = File::create("output.pdf").unwrap();
    doc.save(&mut BufWriter::new(output)).unwrap();
}
