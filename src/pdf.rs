use std::fs;
use std::io;
use std::path::Path;

use printpdf::{
    Color, Line, LinePoint, Mm, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Point, Pt, Rgb, TextItem,
};
use rusttype::{Font, Scale};
use textwrap::{fill, Options};

const HEADER_FONT_SIZE: f32 = 10.0;
const HEADER_RULE_LEFT_GAP_MM: f32 = 7.0;
const HEADER_RULE_RIGHT_GAP_MM: f32 = 6.0;
const HEADER_RULE_Y_MM: f32 = 277.0;
const HEADER_TEXT_Y_MM: f32 = 278.5;
const HEADER_PAGE_NUMBER_Y_MM: f32 = 276.5;
const BODY_FONT_SIZE: f32 = 9.5;
const BODY_START_Y_MM: f32 = 272.0;
const BODY_LINE_HEIGHT_MM: f32 = 5.2;
const BODY_HORIZONTAL_MARGIN_MM: f32 = 12.0;
const BODY_MIN_WRAP_WIDTH: usize = 40;
const BODY_MAX_WRAP_WIDTH: usize = 110;
const LINE_NUMBER_SAMPLE: &str = "0000";
const LINE_NUMBER_PREFIX_SAMPLE: &str = "0000    ";

type PdfResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct PdfLayout {
    pub body_font_size: f32,
    pub body_line_height_mm: f32,
    pub body_left_mm: f32,
    pub body_block_width_mm: f32,
    pub body_wrap_width: usize,
    pub header_rule_left_mm: f32,
    pub header_rule_right_mm: f32,
    pub header_page_number_x_mm: f32,
}

struct BodyLayout {
    left_indent: Mm,
    block_width_mm: f32,
    line_number_width_mm: f32,
    wrap_width: usize,
}

pub struct PdfWriter {
    doc: PdfDocument,
    font: PdfFontHandle,
    code_name: String,
    code_version: String,
    lines_per_page: usize,
    page_number: usize,
    line_index: usize,
    page_ops: Vec<Op>,
    pages: Vec<PdfPage>,
    page_dimensions: (Mm, Mm),
    header_left_indent: Mm,
    body_left_indent: Mm,
    body_block_width_mm: f32,
    line_number_width_mm: f32,
    body_wrap_width: usize,
}

impl PdfWriter {
    pub fn new(
        font_path: &Path,
        code_name: &str,
        code_version: &str,
        lines_per_page: usize,
        page_dimensions: (Mm, Mm),
    ) -> PdfResult<Self> {
        let font_bytes = fs::read(font_path)?;
        let mut warnings = Vec::new();
        let parsed_font = ParsedFont::from_bytes(&font_bytes, 0, &mut warnings)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("failed to parse font file '{}'", font_path.display()),
                )
            })?;

        let title = format!("{code_name} {code_version}");
        let mut doc = PdfDocument::new(&title);
        doc.metadata.info.creator = "PrintCode".to_owned();
        doc.metadata.info.producer = "https://github.com/yliu7949/PrintCode".to_owned();

        let font_id = doc.add_font(&parsed_font);
        let header_left_indent =
            header_left_indent(&font_bytes, &title, page_dimensions.0);
        let body_layout = body_layout(&font_bytes, page_dimensions.0);

        let mut writer = Self {
            doc,
            font: PdfFontHandle::External(font_id),
            code_name: code_name.to_owned(),
            code_version: code_version.to_owned(),
            lines_per_page,
            page_number: 1,
            line_index: 1,
            page_ops: Vec::new(),
            pages: Vec::new(),
            page_dimensions,
            header_left_indent,
            body_left_indent: body_layout.left_indent,
            body_block_width_mm: body_layout.block_width_mm,
            line_number_width_mm: body_layout.line_number_width_mm,
            body_wrap_width: body_layout.wrap_width,
        };

        writer.write_header();
        Ok(writer)
    }

    pub fn add_line(&mut self, line: &str) {
        if self.line_index > self.lines_per_page {
            self.new_page();
        }

        let wrapped_line = fill(
            line,
            Options::new(self.body_wrap_width).subsequent_indent("    "),
        );
        let mut y =
            BODY_START_Y_MM - BODY_LINE_HEIGHT_MM * (self.line_index - 1) as f32;
        for wrapped_line in wrapped_line.lines() {
            if self.line_index > self.lines_per_page {
                self.new_page();
                y = BODY_START_Y_MM;
            }

            self.write_text(
                &format!("{:>4}    {}", self.line_index, wrapped_line),
                BODY_FONT_SIZE,
                self.body_left_indent,
                Mm(y),
            );

            y -= BODY_LINE_HEIGHT_MM;
            self.line_index += 1;
        }
    }

    pub fn save(mut self, output_pdf_path: &Path) -> PdfResult<()> {
        self.finish_page();

        let mut warnings = Vec::new();
        let bytes = self
            .doc
            .with_pages(self.pages)
            .save(&PdfSaveOptions::default(), &mut warnings);
        fs::write(output_pdf_path, bytes)?;

        Ok(())
    }

    pub fn layout(&self) -> PdfLayout {
        PdfLayout {
            body_font_size: BODY_FONT_SIZE,
            body_line_height_mm: BODY_LINE_HEIGHT_MM,
            body_left_mm: self.body_left_indent.0,
            body_block_width_mm: self.body_block_width_mm,
            body_wrap_width: self.body_wrap_width,
            header_rule_left_mm: self.header_rule_left().0,
            header_rule_right_mm: self.header_rule_right().0,
            header_page_number_x_mm: self.header_page_number_x().0,
        }
    }

    fn write_header(&mut self) {
        let header = format!("{} {}", self.code_name, self.code_version);
        self.write_text(
            &header,
            HEADER_FONT_SIZE,
            self.header_left_indent,
            Mm(HEADER_TEXT_Y_MM),
        );

        self.page_ops.push(Op::SetOutlineThickness { pt: Pt(1.2) });
        self.page_ops.push(Op::SetOutlineColor {
            col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
        });
        self.page_ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point::new(self.header_rule_left(), Mm(HEADER_RULE_Y_MM)),
                        bezier: false,
                    },
                    LinePoint {
                        p: Point::new(self.header_rule_right(), Mm(HEADER_RULE_Y_MM)),
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });

        self.write_text(
            &self.page_number.to_string(),
            BODY_FONT_SIZE,
            self.header_page_number_x(),
            Mm(HEADER_PAGE_NUMBER_Y_MM),
        );
    }

    fn header_rule_left(&self) -> Mm {
        Mm(self.body_left_indent.0
            + self.line_number_width_mm
            + HEADER_RULE_LEFT_GAP_MM)
    }

    fn header_rule_right(&self) -> Mm {
        Mm(self.header_page_number_x().0 - HEADER_RULE_RIGHT_GAP_MM)
    }

    fn header_page_number_x(&self) -> Mm {
        Mm(self.page_dimensions.0 .0 - self.body_left_indent.0)
    }

    fn write_text(&mut self, text: &str, font_size: f32, x: Mm, y: Mm) {
        self.page_ops.extend([
            Op::StartTextSection,
            Op::SetTextCursor {
                pos: Point::new(x, y),
            },
            Op::SetFont {
                font: self.font.clone(),
                size: Pt(font_size),
            },
            Op::SetLineHeight { lh: Pt(font_size) },
            Op::SetFillColor {
                col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
            },
            Op::ShowText {
                items: vec![TextItem::Text(text.to_owned())],
            },
            Op::EndTextSection,
        ]);
    }

    fn new_page(&mut self) {
        self.finish_page();
        self.page_number += 1;
        self.line_index = 1;
        self.write_header();
    }

    fn finish_page(&mut self) {
        let ops = std::mem::take(&mut self.page_ops);
        self.pages.push(PdfPage::new(
            self.page_dimensions.0,
            self.page_dimensions.1,
            ops,
        ));
    }
}

fn header_left_indent(font_bytes: &[u8], text: &str, page_width: Mm) -> Mm {
    let header_width = calculate_text_width(font_bytes, text, HEADER_FONT_SIZE)
        .unwrap_or_else(|| estimate_text_width(text, HEADER_FONT_SIZE));

    (page_width - Mm(header_width)) / 2.0
}

fn body_layout(font_bytes: &[u8], page_width: Mm) -> BodyLayout {
    let line_number_width = text_width(font_bytes, LINE_NUMBER_SAMPLE, BODY_FONT_SIZE);
    let prefix_width =
        text_width(font_bytes, LINE_NUMBER_PREFIX_SAMPLE, BODY_FONT_SIZE);
    let char_width = text_width(font_bytes, "M", BODY_FONT_SIZE);
    let available_width = page_width.0 - BODY_HORIZONTAL_MARGIN_MM * 2.0 - prefix_width;
    let wrap_width = ((available_width / char_width).floor() as usize)
        .clamp(BODY_MIN_WRAP_WIDTH, BODY_MAX_WRAP_WIDTH);
    let block_width_mm = prefix_width + char_width * wrap_width as f32;
    let left_indent = Mm((page_width.0 - block_width_mm) / 2.0);

    BodyLayout {
        left_indent,
        block_width_mm,
        line_number_width_mm: line_number_width,
        wrap_width,
    }
}

fn calculate_text_width(font_bytes: &[u8], text: &str, font_size: f32) -> Option<f32> {
    let font = Font::try_from_bytes(font_bytes)?;
    let scale = Scale::uniform(font_size);
    let str_width: f32 = font
        .glyphs_for(text.chars())
        .map(|glyph| glyph.scaled(scale).h_metrics().advance_width)
        .sum();

    Some(str_width * 25.4 / 72.0)
}

fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.5 * 25.4 / 72.0
}

fn text_width(font_bytes: &[u8], text: &str, font_size: f32) -> f32 {
    calculate_text_width(font_bytes, text, font_size)
        .unwrap_or_else(|| estimate_text_width(text, font_size))
}
