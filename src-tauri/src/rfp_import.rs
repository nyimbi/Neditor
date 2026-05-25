use crate::path_to_string;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use zip::ZipArchive;

#[derive(Debug, Deserialize)]
pub(crate) struct ImportRfpSourceRequest {
    pub(crate) source_type: String,
    pub(crate) path: Option<String>,
    pub(crate) url: Option<String>,
    pub(crate) text: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ImportRfpSourceResponse {
    pub(crate) source_type: String,
    pub(crate) title: String,
    pub(crate) path: Option<String>,
    pub(crate) url: Option<String>,
    pub(crate) text: String,
    pub(crate) extraction_method: String,
    pub(crate) warnings: Vec<String>,
}

#[tauri::command]
pub(crate) fn import_rfp_source(
    request: ImportRfpSourceRequest,
) -> Result<ImportRfpSourceResponse, String> {
    let source_type = normalize_source_type(&request.source_type);
    let mut warnings = Vec::new();
    let (text, title, extraction_method) = match source_type.as_str() {
        "markdown" => import_markdown(&request)?,
        "docx" => import_docx(&request)?,
        "pdf" => import_pdf(&request, &mut warnings)?,
        "url" => import_url(&request, &mut warnings)?,
        _ => {
            return Err("Unsupported RFP source type. Use markdown, pdf, docx, or url.".to_string())
        }
    };
    let cleaned = normalize_text(&text);
    if cleaned.len() < 400 {
        warnings.push("Imported source is short; confirm the full RFP, attachments, and tables were captured.".to_string());
    }
    Ok(ImportRfpSourceResponse {
        source_type,
        title,
        path: request
            .path
            .map(|path| path_to_string(&PathBuf::from(path))),
        url: request
            .url
            .map(|url| url.trim().to_string())
            .filter(|url| !url.is_empty()),
        text: cleaned,
        extraction_method,
        warnings,
    })
}

fn normalize_source_type(value: &str) -> String {
    match value.trim().to_lowercase().as_str() {
        "md" | "markdown" | "txt" | "text" => "markdown".to_string(),
        "pdf" => "pdf".to_string(),
        "docx" | "word" => "docx".to_string(),
        "url" | "uri" | "link" => "url".to_string(),
        other => other.to_string(),
    }
}

fn import_markdown(request: &ImportRfpSourceRequest) -> Result<(String, String, String), String> {
    if let Some(text) = request
        .text
        .as_deref()
        .filter(|text| !text.trim().is_empty())
    {
        return Ok((
            text.to_string(),
            request_title(request),
            "pasted-markdown-text".to_string(),
        ));
    }
    let path = request_path(request)?;
    let text =
        fs::read_to_string(&path).map_err(|err| format!("Could not read Markdown RFP: {err}"))?;
    Ok((
        text,
        title_from_path(&path),
        "native-markdown-file".to_string(),
    ))
}

fn import_docx(request: &ImportRfpSourceRequest) -> Result<(String, String, String), String> {
    let path = request_path(request)?;
    let file = fs::File::open(&path).map_err(|err| format!("Could not open DOCX RFP: {err}"))?;
    let mut archive =
        ZipArchive::new(file).map_err(|err| format!("Could not inspect DOCX package: {err}"))?;
    let mut parts = Vec::new();
    for name in [
        "word/document.xml",
        "word/header1.xml",
        "word/header2.xml",
        "word/footer1.xml",
        "word/footer2.xml",
    ] {
        if let Ok(mut file) = archive.by_name(name) {
            let mut xml = String::new();
            file.read_to_string(&mut xml)
                .map_err(|err| format!("Could not read {name} from DOCX: {err}"))?;
            let text = docx_xml_to_text(&xml);
            if !text.trim().is_empty() {
                parts.push(text);
            }
        }
    }
    if parts.is_empty() {
        return Err("DOCX did not contain readable Word document XML.".to_string());
    }
    Ok((
        parts.join("\n\n"),
        title_from_path(&path),
        "native-docx-package-text".to_string(),
    ))
}

fn import_pdf(
    request: &ImportRfpSourceRequest,
    warnings: &mut Vec<String>,
) -> Result<(String, String, String), String> {
    let path = request_path(request)?;
    let output = Command::new("pdftotext")
        .arg("-layout")
        .arg(&path)
        .arg("-")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    match output {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            warnings.push("PDF text extraction depends on the local pdftotext utility; verify tables and scanned pages manually.".to_string());
            Ok((text, title_from_path(&path), "pdftotext-layout".to_string()))
        }
        Ok(output) => Err(format!(
            "Could not extract PDF text with pdftotext: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(err) => Err(format!(
            "PDF import needs the pdftotext utility on this machine, or paste extracted RFP text manually: {err}"
        )),
    }
}

fn import_url(
    request: &ImportRfpSourceRequest,
    warnings: &mut Vec<String>,
) -> Result<(String, String, String), String> {
    let url = request
        .url
        .as_deref()
        .map(str::trim)
        .filter(|url| url.starts_with("https://") || url.starts_with("http://"))
        .ok_or_else(|| "Enter an http:// or https:// RFP URL.".to_string())?;
    let output = Command::new("curl")
        .args([
            "--location",
            "--max-time",
            "25",
            "--fail",
            "--silent",
            "--show-error",
            url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| format!("Could not fetch RFP URL with curl: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "Could not fetch RFP URL: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let raw = String::from_utf8_lossy(&output.stdout).to_string();
    warnings.push("URL import captures the public page response; verify linked attachments and addenda manually.".to_string());
    Ok((
        html_to_text(&raw),
        url.to_string(),
        "curl-url-text".to_string(),
    ))
}

fn request_path(request: &ImportRfpSourceRequest) -> Result<PathBuf, String> {
    request
        .path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "Choose an RFP source file first.".to_string())
}

fn request_title(request: &ImportRfpSourceRequest) -> String {
    request
        .url
        .as_deref()
        .or(request.path.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Pasted RFP")
        .to_string()
}

fn title_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Imported RFP")
        .to_string()
}

fn docx_xml_to_text(xml: &str) -> String {
    xml.replace("</w:p>", "\n")
        .replace("</w:tr>", "\n")
        .replace("</w:tc>", "\t")
        .replace("<w:tab/>", "\t")
        .replace("<w:br/>", "\n")
        .split('<')
        .map(|chunk| chunk.split_once('>').map(|(_, text)| text).unwrap_or(""))
        .collect::<Vec<_>>()
        .join("")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn html_to_text(html: &str) -> String {
    html.replace("</p>", "\n")
        .replace("</li>", "\n")
        .replace("</tr>", "\n")
        .replace("</h1>", "\n")
        .replace("</h2>", "\n")
        .replace("</h3>", "\n")
        .split('<')
        .map(|chunk| chunk.split_once('>').map(|(_, text)| text).unwrap_or(""))
        .collect::<Vec<_>>()
        .join(" ")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}

fn normalize_text(text: &str) -> String {
    text.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("\n\n\n", "\n\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docx_xml_text_preserves_paragraphs_and_table_cells() {
        let text = docx_xml_to_text(
            r#"<w:p><w:r><w:t>Requirement A</w:t></w:r></w:p><w:tr><w:tc><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr>"#,
        );
        assert!(text.contains("Requirement A"));
        assert!(text.contains("Cell"));
    }

    #[test]
    fn url_html_text_strips_markup() {
        let text = html_to_text("<h1>RFP</h1><p>Vendor must provide support.</p>");
        assert!(text.contains("RFP"));
        assert!(text.contains("Vendor must provide support."));
        assert!(!text.contains("<p>"));
    }
}
