use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use textwrap::{fill, Options};

const SOURCE_WRAP_WIDTH: usize = 78;
const MAX_PAGES_WITH_LIMIT: usize = 60;
const KEEP_PAGES_AT_EACH_END: usize = 30;
const TEXT_SAMPLE_SIZE: usize = 8192;

const CODE_EXTENSIONS: &[&str] = &[
    "asm", "bat", "c", "cc", "clj", "cljs", "cmake", "cpp", "cs", "css", "cxx", "dart",
    "ex", "exs", "fs", "fsi", "fsx", "go", "groovy", "h", "hpp", "hrl", "hs", "htm",
    "html", "java", "jl", "js", "jsx", "kt", "kts", "lua", "m", "mm", "php", "pl",
    "pm", "ps1", "py", "r", "rb", "rs", "scala", "scss", "sh", "sql", "swift", "toml",
    "ts", "tsx", "vb", "vue", "xml", "yaml", "yml", "zig",
];

const CODE_FILENAMES: &[&str] = &[
    "build",
    "build.bazel",
    "build.boot",
    "build.gradle",
    "build.gradle.kts",
    "cargo.toml",
    "cmakelists.txt",
    "dockerfile",
    "flake.nix",
    "gradle.properties",
    "justfile",
    "makefile",
    "meson.build",
    "package.json",
    "pom.xml",
    "rakefile",
    "setup.cfg",
    "vagrantfile",
];

const SKIPPED_DIRECTORIES: &[&str] = &[
    ".cargo",
    ".git",
    ".github",
    ".gradle",
    ".idea",
    ".next",
    ".nuxt",
    ".vscode",
    "build",
    "coverage",
    "dist",
    "node_modules",
    "out",
    "target",
    "vendor",
];

pub struct LineSelection {
    pub lines: Vec<String>,
    pub message: Option<String>,
}

pub struct SourceCollection {
    pub lines: Vec<String>,
    pub stats: SourceStats,
}

#[derive(Default)]
pub struct SourceStats {
    pub scanned_files_after_gitignore: usize,
    pub selected_files: Vec<PathBuf>,
    pub skipped_non_code_files: usize,
    pub skipped_non_text_files: usize,
}

pub fn collect_code_lines(
    code_folder: &Path,
) -> Result<SourceCollection, std::io::Error> {
    let mut builder = WalkBuilder::new(code_folder);
    builder
        .hidden(false)
        .ignore(false)
        .parents(false)
        .git_global(false)
        .git_exclude(false)
        .git_ignore(true)
        .require_git(false)
        .filter_entry(|entry| !is_skipped_directory(entry))
        .sort_by_file_path(|left, right| left.cmp(right));

    let paths = builder
        .build()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .map(|file_type| file_type.is_file())
                .unwrap_or(false)
        })
        .map(|entry| entry.path().to_path_buf())
        .collect::<Vec<PathBuf>>();

    let mut all_lines = Vec::new();
    let mut stats = SourceStats::default();
    for path in paths {
        stats.scanned_files_after_gitignore += 1;

        if !is_code_file(&path) {
            stats.skipped_non_code_files += 1;
            continue;
        }

        if !is_likely_text_file(&path).unwrap_or(false) {
            stats.skipped_non_text_files += 1;
            continue;
        }

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        for line_result in reader.lines() {
            let line = line_result?;
            if !line.trim().is_empty() {
                let wrapped_line = fill(&line, Options::new(SOURCE_WRAP_WIDTH));
                all_lines.extend(wrapped_line.lines().map(str::to_owned));
            }
        }

        stats.selected_files.push(path);
    }

    Ok(SourceCollection {
        lines: all_lines,
        stats,
    })
}

fn is_skipped_directory(entry: &ignore::DirEntry) -> bool {
    if !entry
        .file_type()
        .map(|file_type| file_type.is_dir())
        .unwrap_or(false)
    {
        return false;
    }

    let Some(file_name) = entry.path().file_name().and_then(|name| name.to_str())
    else {
        return false;
    };

    let file_name = file_name.to_ascii_lowercase();
    SKIPPED_DIRECTORIES
        .binary_search(&file_name.as_str())
        .is_ok()
}

fn is_code_file(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    let file_name = file_name.to_ascii_lowercase();
    if CODE_FILENAMES.binary_search(&file_name.as_str()).is_ok() {
        return true;
    }

    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            let extension = extension.to_ascii_lowercase();
            CODE_EXTENSIONS.binary_search(&extension.as_str()).is_ok()
        })
        .unwrap_or(false)
}

fn is_likely_text_file(path: &Path) -> Result<bool, std::io::Error> {
    let mut file = File::open(path)?;
    let mut sample = vec![0; TEXT_SAMPLE_SIZE];
    let bytes_read = file.read(&mut sample)?;
    sample.truncate(bytes_read);

    if sample.contains(&0) {
        return Ok(false);
    }

    Ok(std::str::from_utf8(&sample).is_ok())
}

pub fn select_lines(
    all_lines: Vec<String>,
    lines_per_page: usize,
    limit_pages: bool,
) -> LineSelection {
    if !limit_pages {
        return LineSelection {
            lines: all_lines,
            message: None,
        };
    }

    let total_lines = all_lines.len();
    let total_pages = total_lines.div_ceil(lines_per_page);
    let keep_lines = KEEP_PAGES_AT_EACH_END * lines_per_page;

    if total_pages > MAX_PAGES_WITH_LIMIT {
        let first_part = all_lines
            .iter()
            .take(keep_lines)
            .cloned()
            .collect::<Vec<_>>();
        let last_part = all_lines
            .iter()
            .rev()
            .take(keep_lines)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>();

        LineSelection {
            lines: [first_part, last_part].concat(),
            message: Some(format_limit_exceeded_message(total_pages, total_lines)),
        }
    } else {
        LineSelection {
            lines: all_lines,
            message: Some(format!(
                "[Info] Total pages ({} pages, {} lines) within the limit of {} pages.",
                total_pages, total_lines, MAX_PAGES_WITH_LIMIT
            )),
        }
    }
}

fn format_limit_exceeded_message(total_pages: usize, total_lines: usize) -> String {
    format!(
        "[Info] Total pages ({} pages, {} lines) exceed {} pages. \
        Limiting to first and last {} pages.",
        total_pages, total_lines, MAX_PAGES_WITH_LIMIT, KEEP_PAGES_AT_EACH_END
    )
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn code_filter_accepts_common_code_files() {
        assert!(is_code_file(Path::new("src/main.rs")));
        assert!(is_code_file(Path::new("Dockerfile")));
        assert!(is_code_file(Path::new("CMakeLists.txt")));
        assert!(is_code_file(Path::new("package.json")));
    }

    #[test]
    fn code_filter_rejects_docs_and_assets() {
        assert!(!is_code_file(Path::new("README.md")));
        assert!(!is_code_file(Path::new("manual.pdf")));
        assert!(!is_code_file(Path::new("diagram.svg")));
        assert!(!is_code_file(Path::new("spec.docx")));
    }

    #[test]
    fn collect_code_lines_skips_docs_and_binary_files() {
        let dir = temp_test_dir();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src").join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.join("README.md"), "# Project\n").unwrap();
        fs::write(dir.join("src").join("generated.rs"), b"fn ok() {}\0ignored")
            .unwrap();
        fs::create_dir_all(dir.join("target").join("debug")).unwrap();
        fs::write(
            dir.join("target").join("debug").join("build_script.rs"),
            "fn generated() {}\n",
        )
        .unwrap();
        fs::create_dir_all(dir.join(".idea")).unwrap();
        fs::write(dir.join(".idea").join("workspace.xml"), "<project />\n").unwrap();

        let source = collect_code_lines(&dir).unwrap();

        assert_eq!(source.lines, vec!["fn main() {}"]);
        assert_eq!(source.stats.selected_files.len(), 1);
        assert_eq!(source.stats.skipped_non_text_files, 1);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn collect_code_lines_honors_scoped_gitignore_files_first() {
        let dir = temp_test_dir();
        fs::create_dir_all(dir.join("nested")).unwrap();
        fs::write(dir.join(".gitignore"), "root_ignored.rs\n").unwrap();
        fs::write(dir.join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.join("root_ignored.rs"), "fn ignored() {}\n").unwrap();
        fs::write(dir.join("nested").join(".gitignore"), "*.ts\n!keep.ts\n").unwrap();
        fs::write(dir.join("nested").join("skip.ts"), "const skip = true;\n").unwrap();
        fs::write(dir.join("nested").join("keep.ts"), "const keep = true;\n").unwrap();

        let source = collect_code_lines(&dir).unwrap();

        assert_eq!(source.lines, vec!["fn main() {}", "const keep = true;"]);
        assert_eq!(source.stats.selected_files.len(), 2);

        fs::remove_dir_all(dir).unwrap();
    }

    fn temp_test_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("printcode-source-test-{nanos}"))
    }
}
