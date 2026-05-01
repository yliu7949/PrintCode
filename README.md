<p align="center">
  <img width="190" src="https://raw.githubusercontent.com/yliu7949/PrintCode/master/logo.svg" style="text-align: center;" alt="PrintCode logo">
</p>


# PrintCode

[![License](https://img.shields.io/github/license/yliu7949/PrintCode)](https://github.com/yliu7949/PrintCode/blob/master/LICENSE)
[![Build Status](https://github.com/yliu7949/PrintCode/actions/workflows/rust.yml/badge.svg)](https://github.com/yliu7949/PrintCode/actions?query=workflow%3ARust)
[![Github Downloads](https://img.shields.io/github/downloads/yliu7949/PrintCode/total.svg)](http://gra.caldis.me/?url=https://github.com/yliu7949/PrintCode)
<a title="Hits" target="_blank" href="https://github.com/yliu7949/PrintCode"><img src="https://hits.b3log.org/yliu7949/PrintCode.svg"></a>
[![Github Release Version](https://img.shields.io/github/v/release/yliu7949/PrintCode?color=green&include_prereleases)](https://github.com/yliu7949/PrintCode/releases/latest)

PrintCode 是一个使用 Rust 语言编写的、用于从代码文件生成为带有自定义页眉的 PDF 文档的命令行工具。它支持从指定目录中读取代码文件，并将代码按照每页固定代码行数以美观的格式输出为 PDF 文档。该工具特别适用于需要将代码整理为文档以供打印或发布的场景，例如在申请计算机软件著作权登记时使用的程序鉴别材料。

## 功能

- 支持自定义页眉，包含代码名称和版本号。
- 自动分页，适应不同代码文件的长度。
- 支持多种字体，用户可以选择自定义的字体文件。
- `--verbose` 可输出扫描统计、入选文件、字体、页设置和输出路径。

## 编译

1. 克隆项目到本地：

   ```bash
   git clone https://github.com/yliu7949/PrintCode.git
   cd PrintCode
   ```

2. 使用 Cargo 编译项目：

   ```bash
   cargo build --release
   ```
   
3. 生成的可执行文件将位于 `target/release/` 目录下。

## 示例

假设你的项目文件夹为 `src_folder`，你希望将其中的代码生成为 PDF 文档。具体步骤如下：

1. 准备字体文件，放在任意文件夹中。Windows 可使用 `C:/Windows/Fonts/simsun.ttc`；macOS 默认使用 `/System/Library/Fonts/SFNSMono.ttf`，通常无需额外指定。

2. 运行下面的命令。macOS 可以直接使用默认字体：

   ```bash
   ./target/release/printcode \
   --code-folder src_folder \
   --code-name "MyProject" \
   --code-version "V1.0.0" \
   --output-path MyOutput.pdf \
   --verbose
   ```

   Windows 可显式指定宋体：

   ```bash
   ./printcode \
   --font-dir C:/Windows/Fonts \
   --font-name simsun.ttc \
   --code-folder src_folder \
   --code-name "MyProject" \
   --code-version "V1.0.0" \
   --output-path MyOutput.pdf \
   --verbose
   ```

   其中 `--code-name` 和 `--code-version` 的值会用于生成 PDF 文档的页眉文字。

3. 该命令将生成一个名为 `MyOutput.pdf` 的文档。扫描时会先按照 `src_folder` 内各级 `.gitignore` 的作用域过滤文件，再收集剩余文件中的代码文件，并跳过 README、Markdown、PDF、Word、Excel、图片等文档或资源文件。`target`、`.git`、`.idea`、`.vscode`、`node_modules` 等构建、依赖和工具目录也会被跳过。

4. 如果仅需要生成连续前 30 页和连续后 30 页源代码为 PDF 文档，则可以使用 `--limit-pages` 或 `-l` 参数。

5. 如果需要调整每页代码行数，可以使用 `--lines-per-page` 参数，例如：

   ```bash
   ./target/release/printcode -d src_folder -n "MyProject" --lines-per-page 55
   ```

**Tips：** 如果你正在使用 Windows 平台，且操作系统的字体文件夹 `C:/Windows/Fonts` 中已内置了 `simsun.ttc` 字体，则上述命令可以简化为：

```bash
./printcode -d src_folder -n "PrintCode" -v "V1.0.0"
```

## 生成的 PDF 文档示例

生成的 PDF 文件包含以下内容：

- **页眉**：页眉显示代码名称和版本号，例如 "MyProject V1.0.0"。
- **分页**：默认每页包含 50 行代码，超过 50 行自动分页；正文使用较舒展的行距以减少页面底部空白。
- **文件过滤**：如果目录内存在一个或多个 `.gitignore`，会先按其所在目录的作用域过滤文件；随后只输出常见代码文件和必要的构建配置文件，非文本文件、文档、图片、依赖目录、构建输出目录和 IDE 配置目录会被跳过。
- **代码格式**：代码按原始格式显示，支持缩进，并会根据当前字体宽度自动换行以避免超出页面。但代码中所有的空白行均会被过滤不显示。

下面是生成的[示例 PDF 文档](https://github.com/yliu7949/PrintCode/blob/master/demo.pdf)的截图：

![demo](https://raw.githubusercontent.com/yliu7949/PrintCode/master/demo.svg)

## 常见问题

### 1. 如何更改每页显示的代码行数？

在生成 PDF 时，可以通过 `--lines-per-page` 参数控制每页行数。默认每页输出 50 行代码。

### 2. 是否支持其他字体格式？

支持多种字体格式，如 TTF 和 OTF 等。

### 3. 为什么生成的 PDF 文档中出现了乱码？

可能是字体文件不支持代码中的字符集，请确保所选字体支持所使用的字符。

## 开发说明

代码按职责拆分在 `src/` 目录下：

- `main.rs`：程序入口和整体流程编排。
- `cli.rs`：命令行参数解析及不同平台的默认字体配置。
- `source.rs`：按 `.gitignore` 递归读取代码文件、过滤文档/非文本文件、过滤空行、换行和页数裁剪。
- `pdf.rs`：PDF 页面、页眉、正文和元数据写入。

## 发布说明

GitHub Actions 会在提交和合并请求时运行格式检查、测试、Clippy 和 release 构建。
需要发布版本时，可在 Actions 页面手动触发 `Rust` workflow；它会按
`Cargo.toml` 中的版本创建 release，并上传 Linux、macOS、Windows 二进制包和源码压缩包。

## 贡献指南

欢迎提交 Issue 和 Pull Request 来改进本项目。请确保提交的代码符合 Rust 编码规范。

## 许可证

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fyliu7949%2FPrintCode.svg?type=large&issueType=license)](https://app.fossa.com/projects/git%2Bgithub.com%2Fyliu7949%2FPrintCode?ref=badge_large&issueType=license)

 
