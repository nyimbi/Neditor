use crate::{
    markdown_tables::{
        escape_markdown_table_cell, is_markdown_table_row, is_markdown_table_separator,
        markdown_table_row, split_markdown_table_row,
    },
    path_to_string,
    tables::parse_delimited_rows,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

#[derive(Debug, Deserialize)]
pub(crate) struct ImportSpreadsheetTableRequest {
    pub(crate) path: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ExportMarkdownTablesRequest {
    pub(crate) markdown: String,
    pub(crate) output_path: String,
    pub(crate) format: String,
    pub(crate) table_index: Option<usize>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ImportSpreadsheetTableResponse {
    pub(crate) source_path: String,
    pub(crate) source_format: String,
    pub(crate) sheet_name: String,
    pub(crate) rows: usize,
    pub(crate) columns: usize,
    pub(crate) markdown: String,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExportMarkdownTablesResponse {
    pub(crate) output_path: String,
    pub(crate) format: String,
    pub(crate) table_count: usize,
    pub(crate) exported_tables: usize,
    pub(crate) rows: usize,
    pub(crate) columns: usize,
}

#[derive(Debug, Clone)]
struct DataTable {
    caption: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

pub(crate) fn import_xlsx_data_source_markdown(
    path: &Path,
    caption: &str,
) -> Result<(String, Vec<String>), String> {
    let (mut table, warnings) = read_xlsx_first_sheet(path)?;
    table.caption = caption.trim().to_string();
    Ok((table_to_markdown(&table), warnings))
}

#[tauri::command]
pub(crate) fn import_spreadsheet_table(
    request: ImportSpreadsheetTableRequest,
) -> Result<ImportSpreadsheetTableResponse, String> {
    let path = PathBuf::from(request.path.trim());
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let (table, source_format, sheet_name, warnings) = match extension.as_str() {
        "csv" => {
            let text = fs::read_to_string(&path)
                .map_err(|err| format!("Could not read CSV file {}: {err}", path.display()))?;
            (
                table_from_rows(parse_delimited_rows(&text, ','), "Imported CSV"),
                "csv",
                "CSV",
                Vec::new(),
            )
        }
        "tsv" => {
            let text = fs::read_to_string(&path)
                .map_err(|err| format!("Could not read TSV file {}: {err}", path.display()))?;
            (
                table_from_rows(parse_delimited_rows(&text, '\t'), "Imported TSV"),
                "tsv",
                "TSV",
                Vec::new(),
            )
        }
        "xlsx" => {
            let (table, warnings) = read_xlsx_first_sheet(&path)?;
            (table, "xlsx", "Sheet1", warnings)
        }
        _ => {
            return Err("Import supports .csv, .tsv, and .xlsx files.".to_string());
        }
    };
    Ok(ImportSpreadsheetTableResponse {
        source_path: path_to_string(&path),
        source_format: source_format.to_string(),
        sheet_name: sheet_name.to_string(),
        rows: table.rows.len(),
        columns: table.headers.len(),
        markdown: table_to_markdown(&table),
        warnings,
    })
}

#[tauri::command]
pub(crate) fn export_markdown_tables(
    request: ExportMarkdownTablesRequest,
) -> Result<ExportMarkdownTablesResponse, String> {
    let tables = extract_markdown_tables(&request.markdown);
    if tables.is_empty() {
        return Err("No Markdown tables were found to export.".to_string());
    }
    let selected = if let Some(index) = request.table_index {
        vec![tables
            .get(index)
            .cloned()
            .ok_or_else(|| format!("Table index {index} does not exist."))?]
    } else {
        tables.clone()
    };
    let output_path = PathBuf::from(request.output_path.trim());
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let format = request.format.trim().to_ascii_lowercase();
    match format.as_str() {
        "csv" => {
            let table = selected
                .first()
                .ok_or_else(|| "No selected table is available.".to_string())?;
            fs::write(&output_path, table_to_csv(table)).map_err(|err| err.to_string())?;
        }
        "xlsx" => {
            fs::write(&output_path, tables_to_xlsx_bytes(&selected)?)
                .map_err(|err| err.to_string())?;
        }
        _ => return Err("Export format must be csv or xlsx.".to_string()),
    }
    let first = selected
        .first()
        .ok_or_else(|| "No selected table is available.".to_string())?;
    Ok(ExportMarkdownTablesResponse {
        output_path: path_to_string(&output_path),
        format,
        table_count: tables.len(),
        exported_tables: selected.len(),
        rows: first.rows.len(),
        columns: first.headers.len(),
    })
}

fn table_from_rows(mut rows: Vec<Vec<String>>, caption: &str) -> DataTable {
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    for row in &mut rows {
        row.resize(width, String::new());
    }
    let headers = rows
        .first()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                format!("Column {}", index + 1)
            } else {
                trimmed.to_string()
            }
        })
        .collect::<Vec<_>>();
    DataTable {
        caption: caption.to_string(),
        headers,
        rows: rows.into_iter().skip(1).collect(),
    }
}

fn extract_markdown_tables(markdown: &str) -> Vec<DataTable> {
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut tables = Vec::new();
    let mut index = 0usize;
    while index + 1 < lines.len() {
        if !is_markdown_table_row(lines[index].trim())
            || !is_markdown_table_separator(lines[index + 1].trim())
        {
            index += 1;
            continue;
        }
        let caption = if index > 0 {
            parse_table_caption(lines[index - 1])
        } else {
            String::new()
        };
        let headers = split_markdown_table_row(lines[index].trim());
        index += 2;
        let mut rows = Vec::new();
        while index < lines.len() && is_markdown_table_row(lines[index].trim()) {
            let mut row = split_markdown_table_row(lines[index].trim());
            row.resize(headers.len(), String::new());
            rows.push(row);
            index += 1;
        }
        tables.push(DataTable {
            caption,
            headers,
            rows,
        });
    }
    tables
}

fn parse_table_caption(line: &str) -> String {
    line.trim()
        .strip_prefix("Table:")
        .or_else(|| line.trim().strip_prefix("table:"))
        .unwrap_or_default()
        .replace(|ch| matches!(ch, '{' | '}'), "")
        .trim()
        .to_string()
}

fn table_to_markdown(table: &DataTable) -> String {
    let mut lines = Vec::new();
    if !table.caption.trim().is_empty() {
        lines.push(format!("Table: {}", table.caption.trim()));
    }
    lines.push(markdown_table_row(&table.headers));
    lines.push(markdown_table_row(
        &table
            .headers
            .iter()
            .map(|_| "---".to_string())
            .collect::<Vec<_>>(),
    ));
    lines.extend(table.rows.iter().map(|row| {
        let mut padded = row.clone();
        padded.resize(table.headers.len(), String::new());
        markdown_table_row(&padded)
    }));
    lines.join("\n")
}

fn table_to_csv(table: &DataTable) -> String {
    let mut lines = Vec::new();
    lines.push(csv_row(&table.headers));
    lines.extend(table.rows.iter().map(|row| csv_row(row)));
    format!("{}\n", lines.join("\n"))
}

fn csv_row(row: &[String]) -> String {
    row.iter()
        .map(|cell| csv_cell(cell))
        .collect::<Vec<_>>()
        .join(",")
}

fn csv_cell(cell: &str) -> String {
    if cell.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", cell.replace('"', "\"\""))
    } else {
        cell.to_string()
    }
}

fn read_xlsx_first_sheet(path: &Path) -> Result<(DataTable, Vec<String>), String> {
    let file = fs::File::open(path)
        .map_err(|err| format!("Could not open XLSX file {}: {err}", path.display()))?;
    let mut archive =
        ZipArchive::new(file).map_err(|err| format!("Invalid XLSX archive: {err}"))?;
    let shared_strings = read_shared_strings(&mut archive)?;
    let mut sheet_xml = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .map_err(|_| "Only the first XLSX worksheet is supported for import.".to_string())?
        .read_to_string(&mut sheet_xml)
        .map_err(|err| format!("Could not read XLSX worksheet XML: {err}"))?;
    let rows = rows_from_sheet_xml(&sheet_xml, &shared_strings);
    if rows.is_empty() {
        return Err("The first XLSX worksheet did not contain rows.".to_string());
    }
    let mut warnings = Vec::new();
    warnings
        .push("Imported formulas are read from cached worksheet values when present.".to_string());
    Ok((table_from_rows(rows, "Imported XLSX table"), warnings))
}

fn read_shared_strings<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<Vec<String>, String> {
    let Ok(mut file) = archive.by_name("xl/sharedStrings.xml") else {
        return Ok(Vec::new());
    };
    let mut xml = String::new();
    file.read_to_string(&mut xml)
        .map_err(|err| format!("Could not read XLSX shared strings: {err}"))?;
    Ok(extract_xml_text_tags(&xml, "t"))
}

fn rows_from_sheet_xml(xml: &str, shared_strings: &[String]) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    for row_xml in extract_xml_elements(xml, "row") {
        let mut row = Vec::new();
        for cell_xml in extract_xml_elements(&row_xml, "c") {
            let column = cell_reference_column(&cell_xml).unwrap_or(row.len());
            if row.len() <= column {
                row.resize(column + 1, String::new());
            }
            row[column] = xlsx_cell_value(&cell_xml, shared_strings);
        }
        if row.iter().any(|cell| !cell.trim().is_empty()) {
            rows.push(row);
        }
    }
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    for row in &mut rows {
        row.resize(width, String::new());
    }
    rows
}

fn xlsx_cell_value(cell_xml: &str, shared_strings: &[String]) -> String {
    let value = extract_xml_text_tags(cell_xml, "v")
        .first()
        .cloned()
        .unwrap_or_default();
    if cell_xml.contains(r#"t="s""#) {
        return value
            .parse::<usize>()
            .ok()
            .and_then(|index| shared_strings.get(index).cloned())
            .unwrap_or_default();
    }
    if cell_xml.contains(r#"t="inlineStr""#) {
        return extract_xml_text_tags(cell_xml, "t")
            .first()
            .cloned()
            .unwrap_or_default();
    }
    value
}

fn cell_reference_column(cell_xml: &str) -> Option<usize> {
    let marker = r#" r=""#;
    let start = cell_xml.find(marker)? + marker.len();
    let value = cell_xml.get(start..)?;
    let end = value.find('"')?;
    let letters = value[..end]
        .chars()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .collect::<String>();
    if letters.is_empty() {
        return None;
    }
    let mut column = 0usize;
    for ch in letters.bytes() {
        column = column * 26 + usize::from(ch.to_ascii_uppercase() - b'A' + 1);
    }
    column.checked_sub(1)
}

fn tables_to_xlsx_bytes(tables: &[DataTable]) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file("[Content_Types].xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(content_types_xml(tables.len()).as_bytes())
        .map_err(|err| err.to_string())?;
    zip.start_file("_rels/.rels", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(root_rels_xml().as_bytes())
        .map_err(|err| err.to_string())?;
    zip.start_file("xl/workbook.xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(workbook_xml(tables).as_bytes())
        .map_err(|err| err.to_string())?;
    zip.start_file("xl/_rels/workbook.xml.rels", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(workbook_rels_xml(tables.len()).as_bytes())
        .map_err(|err| err.to_string())?;
    for (index, table) in tables.iter().enumerate() {
        zip.start_file(format!("xl/worksheets/sheet{}.xml", index + 1), options)
            .map_err(|err| err.to_string())?;
        zip.write_all(worksheet_xml(table).as_bytes())
            .map_err(|err| err.to_string())?;
    }
    Ok(zip.finish().map_err(|err| err.to_string())?.into_inner())
}

fn content_types_xml(sheet_count: usize) -> String {
    let mut overrides = String::new();
    for index in 1..=sheet_count {
        overrides.push_str(&format!(
            r#"<Override PartName="/xl/worksheets/sheet{index}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#
        ));
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>{overrides}</Types>"#
    )
}

fn root_rels_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#
}

fn workbook_xml(tables: &[DataTable]) -> String {
    let sheets = tables
        .iter()
        .enumerate()
        .map(|(index, table)| {
            let name = sheet_name(table, index);
            format!(
                r#"<sheet name="{}" sheetId="{}" r:id="rId{}"/>"#,
                xml_escape(&name),
                index + 1,
                index + 1
            )
        })
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets>{sheets}</sheets></workbook>"#
    )
}

fn workbook_rels_xml(sheet_count: usize) -> String {
    let relationships = (1..=sheet_count)
        .map(|index| {
            format!(
                r#"<Relationship Id="rId{index}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet{index}.xml"/>"#
            )
        })
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{relationships}</Relationships>"#
    )
}

fn worksheet_xml(table: &DataTable) -> String {
    let mut rows = Vec::new();
    rows.push(table.headers.clone());
    rows.extend(table.rows.clone());
    let body = rows
        .iter()
        .enumerate()
        .map(|(row_index, row)| {
            let cells = row
                .iter()
                .enumerate()
                .map(|(column_index, cell)| {
                    let reference = format!("{}{}", column_name(column_index), row_index + 1);
                    format!(
                        r#"<c r="{reference}" t="inlineStr"><is><t>{}</t></is></c>"#,
                        xml_escape(cell)
                    )
                })
                .collect::<String>();
            format!(r#"<row r="{}">{cells}</row>"#, row_index + 1)
        })
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>{body}</sheetData></worksheet>"#
    )
}

fn sheet_name(table: &DataTable, index: usize) -> String {
    let cleaned = table
        .caption
        .chars()
        .filter(|ch| !matches!(ch, '[' | ']' | ':' | '*' | '?' | '/' | '\\'))
        .collect::<String>();
    let name = cleaned.trim();
    if name.is_empty() {
        format!("Table {}", index + 1)
    } else {
        name.chars().take(31).collect()
    }
}

fn column_name(mut index: usize) -> String {
    let mut name = String::new();
    loop {
        let remainder = index % 26;
        name.insert(0, char::from(b'A' + remainder as u8));
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    name
}

fn extract_xml_blocks(xml: &str, tag: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut start_at = 0usize;
    let open_prefix = format!("<{tag}");
    let close = format!("</{tag}>");
    while let Some(open_offset) = xml[start_at..].find(&open_prefix) {
        let open = start_at + open_offset;
        let Some(open_end_offset) = xml[open..].find('>') else {
            break;
        };
        let content_start = open + open_end_offset + 1;
        let Some(close_offset) = xml[content_start..].find(&close) else {
            break;
        };
        let content_end = content_start + close_offset;
        blocks.push(xml[content_start..content_end].to_string());
        start_at = content_end + close.len();
    }
    blocks
}

fn extract_xml_elements(xml: &str, tag: &str) -> Vec<String> {
    let mut elements = Vec::new();
    let mut start_at = 0usize;
    let open_prefix = format!("<{tag}");
    let close = format!("</{tag}>");
    while let Some(open_offset) = xml[start_at..].find(&open_prefix) {
        let open = start_at + open_offset;
        let Some(open_end_offset) = xml[open..].find('>') else {
            break;
        };
        let content_start = open + open_end_offset + 1;
        let Some(close_offset) = xml[content_start..].find(&close) else {
            break;
        };
        let end = content_start + close_offset + close.len();
        elements.push(xml[open..end].to_string());
        start_at = end;
    }
    elements
}

fn extract_xml_text_tags(xml: &str, tag: &str) -> Vec<String> {
    extract_xml_blocks(xml, tag)
        .into_iter()
        .map(|value| decode_xml_text(value.trim()))
        .collect()
}

fn decode_xml_text(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[allow(dead_code)]
fn markdown_safe_cell(cell: &str) -> String {
    escape_markdown_table_cell(cell)
}
