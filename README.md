<p align="center">
  <img width="190" src="https://raw.githubusercontent.com/yliu7949/PrintCode/master/logo.svg" style="text-align: center;" alt="PrintCode logo">
</p>


# PrintCode

[![License](https://img.shields.io/github/license/yliu7949/PrintCode)](https://github.com/yliu7949/PrintCode/blob/master/LICENSE)
[![Github Downloads](https://img.shields.io/github/downloads/yliu7949/PrintCode/total.svg)](http://gra.caldis.me/?url=https://github.com/yliu7949/PrintCode)
<a title="Hits" target="_blank" href="https://github.com/yliu7949/PrintCode"><img src="https://hits.b3log.org/yliu7949/PrintCode.svg"></a>
[![Github Release Version](https://img.shields.io/github/v/release/yliu7949/PrintCode?color=green&include_prereleases)](https://github.com/yliu7949/PrintCode/releases/latest)

PrintCode 是一个使用 Rust 语言编写的、用于从代码文件生成为带有自定义页眉的 PDF 文档的命令行工具。它支持从指定目录中读取代码文件，并将代码按照每页固定代码行数以美观的格式输出为 PDF 文档。该工具特别适用于需要将代码整理为文档以供打印或发布的场景，例如在申请计算机软件著作权登记时使用的程序鉴别材料。

## 功能

- 支持自定义页眉，包含代码名称和版本号。
- 自动分页，适应不同代码文件的长度。
- 支持多种字体，用户可以选择自定义的字体文件。

## 编译

1. 克隆项目到本地：
   ```bash
   git clone https://github.com/yliu7949/PrintCode.git
   cd PrintCode

2. 使用 Cargo 编译项目：

   ```
   cargo build --release
   ```
   
3. 生成的可执行文件将位于 `target/release/` 目录下。

## 示例

假设你的项目文件夹为 `src_folder`，你希望将其中的代码生成为 PDF 文档。具体步骤如下：

1. 准备字体文件（如 `simsun.ttc`），放在任意文件夹中（例如 `C:/Windows/Fonts` ）。

2. 运行下面的命令：

   ```
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

3. 该命令将生成一个名为 `MyOutput.pdf` 的文档，文档里包含了 `src_folder` 文件夹中所有文本文件的文本内容。

**Tips：** 如果你正在使用 Windows 平台，且操作系统的字体文件夹 `C:/Windows/Fonts` 中已内置了 `simsun.ttc` 字体，则上述命令可以简化为：

```bash
./printcode -d src_folder -n "PrintCode" -v "V1.0.0"
```

## 生成的 PDF 文档示例

生成的 PDF 文件包含以下内容：

- **页眉**：页眉显示代码名称和版本号，例如 "MyProject V1.0.0"。
- **分页**：每页包含 50 行代码，超过 50 行自动分页。
- **代码格式**：代码按原始格式显示，支持缩进和自动换行。但代码中所有的空白行均会被过滤不显示。

下面是生成的[示例 PDF 文档](https://github.com/yliu7949/PrintCode/blob/master/demo.pdf)的截图：

![demo](https://raw.githubusercontent.com/yliu7949/PrintCode/master/demo.svg)

## 常见问题

### 1. 如何更改每页显示的代码行数？

在生成 PDF 时，可以通过修改 `main.rs` 源代码文件中创建 `PdfWriter` 结构体时的 `lines_per_page` 参数来控制每页的行数。默认每页输出 50 行代码。

### 2. 是否支持其他字体格式？

支持多种字体格式，如 TTF 和 OTF 等。

### 3. 为什么生成的 PDF 文档中出现了乱码？

可能是字体文件不支持代码中的字符集，请确保所选字体支持所使用的字符。

## 贡献指南

欢迎提交 Issue 和 Pull Request 来改进本项目。请确保提交的代码符合 Rust 编码规范。

## 许可证

本项目采用 MIT 许可证，详情请参阅 LICENSE 文件。

 
