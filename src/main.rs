mod cli;
mod pdf;
mod source;

use std::error::Error;
use std::time::Instant;

use printpdf::Mm;

use crate::cli::parse_args;
use crate::pdf::{PdfLayout, PdfWriter};
use crate::source::{collect_code_lines, select_lines};

fn main() -> Result<(), Box<dyn Error>> {
    let started_at = Instant::now();
    let result = run();

    println!("[Info] Total elapsed: {:.2?}.", started_at.elapsed());

    result
}

fn run() -> Result<(), Box<dyn Error>> {
    let config = parse_args();

    let source = collect_code_lines(&config.code_folder)?;
    let selection =
        select_lines(source.lines, config.lines_per_page, config.limit_pages);
    let total_selected_pages = selection.lines.len().div_ceil(config.lines_per_page);

    println!("[Info] Source folder: '{}'.", config.code_folder.display());
    if config.verbose {
        print_source_summary(&source.stats);

        if let Some(message) = selection.message {
            println!("{message}");
        }
    }

    println!(
        "[Info] Total lines to write: {} ({} pages).",
        selection.lines.len(),
        total_selected_pages
    );

    let mut pdf_writer = PdfWriter::new(
        &config.font_path(),
        &config.code_name,
        &config.code_version,
        config.lines_per_page,
        (Mm(210.0), Mm(297.0)),
    )?;

    println!("[Info] Font: '{}'.", config.font_path().display());

    if config.verbose {
        print_output_summary(&config, pdf_writer.layout());
    }

    for line in selection.lines {
        pdf_writer.add_line(&line);
    }

    pdf_writer.save(&config.output_path)?;

    println!(
        "[Info] PDF document generated successfully at '{}'.",
        config.output_path.display()
    );

    Ok(())
}

fn print_source_summary(stats: &crate::source::SourceStats) {
    println!(
        "[Info] Files after .gitignore and directory filters: {}.",
        stats.scanned_files_after_gitignore
    );
    println!(
        "[Info] Selected code text files: {}.",
        stats.selected_files.len()
    );
    println!(
        "[Info] Skipped files: {} non-code/document/resource, {} binary/non-UTF-8.",
        stats.skipped_non_code_files, stats.skipped_non_text_files
    );

    let preview_count = 20;
    for path in stats.selected_files.iter().take(preview_count) {
        println!("[Info]   + {}", path.display());
    }

    if stats.selected_files.len() > preview_count {
        println!(
            "[Info]   ... {} more files.",
            stats.selected_files.len() - preview_count
        );
    }
}

fn print_output_summary(config: &crate::cli::CliConfig, layout: PdfLayout) {
    println!("[Info] Output path: '{}'.", config.output_path.display());
    println!(
        "[Info] Page layout: {} lines/page, {:.1}pt body, {:.2}mm line height.",
        config.lines_per_page, layout.body_font_size, layout.body_line_height_mm
    );
    println!(
        "[Info] PDF wrap width: {} code characters after line-number gutter.",
        layout.body_wrap_width
    );
    println!(
        "[Info] Body block: {:.2}mm wide, centered at {:.2}mm left indent.",
        layout.body_block_width_mm, layout.body_left_mm
    );
    println!(
        "[Info] Header rule: {:.2}mm to {:.2}mm, page number at {:.2}mm.",
        layout.header_rule_left_mm,
        layout.header_rule_right_mm,
        layout.header_page_number_x_mm
    );
}
