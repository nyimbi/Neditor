use crate::{document_ast::AstSourceRange, sha256_hex};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ExportImageDimensions {
    pub(crate) width_px: f64,
    pub(crate) height_px: f64,
}

pub(crate) struct ParsedExportImage {
    pub(crate) extension: String,
    pub(crate) content_type: String,
    pub(crate) bytes: Vec<u8>,
    pub(crate) dimensions: Option<ExportImageDimensions>,
}

pub(crate) fn safe_bundle_path(path: &str) -> String {
    let digest = sha256_hex(path.as_bytes());
    let filename = PathBuf::from(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("include.md")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("{}-{filename}", &digest[..12])
}

pub(crate) fn parse_export_image(
    src: &str,
    source: Option<&AstSourceRange>,
) -> Option<ParsedExportImage> {
    parse_image_data_uri(src).or_else(|| read_local_export_image(src, source))
}

fn parse_image_data_uri(src: &str) -> Option<ParsedExportImage> {
    let data = src.strip_prefix("data:")?;
    let (metadata, payload) = data.split_once(',')?;
    let mut parts = metadata.split(';');
    let content_type = parts.next()?.to_ascii_lowercase();
    if !parts.any(|part| part.eq_ignore_ascii_case("base64")) {
        return None;
    }
    let extension = image_extension_for_content_type(&content_type)?;
    let bytes = decode_base64(payload)?;
    let dimensions = export_image_dimensions(&content_type, &bytes);
    Some(ParsedExportImage {
        extension: extension.to_string(),
        content_type,
        bytes,
        dimensions,
    })
}

fn read_local_export_image(
    src: &str,
    source: Option<&AstSourceRange>,
) -> Option<ParsedExportImage> {
    if src.starts_with("data:") || src.contains("://") || src.starts_with('#') {
        return None;
    }
    let path = local_export_image_path(src, source)?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())?;
    let content_type = image_content_type_for_extension(&extension)?;
    let bytes = fs::read(path).ok()?;
    let dimensions = export_image_dimensions(content_type, &bytes);
    Some(ParsedExportImage {
        extension,
        content_type: content_type.to_string(),
        bytes,
        dimensions,
    })
}

fn local_export_image_path(src: &str, source: Option<&AstSourceRange>) -> Option<PathBuf> {
    let path = PathBuf::from(src);
    if path.is_absolute() {
        return Some(path);
    }
    let source_file = source?.source_file.as_str();
    Some(Path::new(source_file).parent()?.join(path))
}

fn image_extension_for_content_type(content_type: &str) -> Option<&'static str> {
    match content_type {
        "image/svg+xml" => Some("svg"),
        "image/png" => Some("png"),
        "image/jpeg" | "image/jpg" => Some("jpg"),
        _ => None,
    }
}

fn image_content_type_for_extension(extension: &str) -> Option<&'static str> {
    match extension {
        "svg" => Some("image/svg+xml"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        _ => None,
    }
}

fn export_image_dimensions(content_type: &str, bytes: &[u8]) -> Option<ExportImageDimensions> {
    match content_type {
        "image/svg+xml" => svg_image_dimensions(bytes),
        "image/png" => png_image_dimensions(bytes),
        "image/jpeg" => jpeg_image_dimensions(bytes),
        _ => None,
    }
}

fn svg_image_dimensions(bytes: &[u8]) -> Option<ExportImageDimensions> {
    let text = std::str::from_utf8(bytes).ok()?;
    let svg_start = text.find("<svg")?;
    let svg_tag = &text[svg_start..];
    let tag_end = svg_tag.find('>')?;
    let tag = &svg_tag[..=tag_end];
    let width = xml_attr_value(tag, "width").and_then(parse_svg_length_px);
    let height = xml_attr_value(tag, "height").and_then(parse_svg_length_px);
    let view_box = xml_attr_value(tag, "viewBox")
        .or_else(|| xml_attr_value(tag, "viewbox"))
        .and_then(parse_svg_view_box);
    let (width_px, height_px) = match (width, height, view_box) {
        (Some(width_px), Some(height_px), _) => (width_px, height_px),
        (Some(width_px), None, Some((_, _, view_width, view_height))) if view_width > 0.0 => {
            (width_px, width_px * view_height / view_width)
        }
        (None, Some(height_px), Some((_, _, view_width, view_height))) if view_height > 0.0 => {
            (height_px * view_width / view_height, height_px)
        }
        (None, None, Some((_, _, view_width, view_height))) => (view_width, view_height),
        _ => return None,
    };
    if width_px <= 0.0 || height_px <= 0.0 {
        return None;
    }
    Some(ExportImageDimensions {
        width_px,
        height_px,
    })
}

fn xml_attr_value<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
    let mut rest = tag;
    while let Some(offset) = rest.find(attr) {
        let candidate_start = tag.len() - rest.len() + offset;
        let candidate_end = candidate_start + attr.len();
        let before = tag[..candidate_start].chars().next_back();
        let is_name_boundary = before
            .map(|ch| !matches!(ch, ':' | '-' | '_' | '.') && !ch.is_ascii_alphanumeric())
            .unwrap_or(true);
        let after = tag[candidate_end..].trim_start();
        if is_name_boundary && after.starts_with('=') {
            let value = after[1..].trim_start();
            let quote = value.chars().next()?;
            if quote == '"' || quote == '\'' {
                let quoted_start = tag.len() - value.len() + quote.len_utf8();
                let quoted_end = tag[quoted_start..].find(quote)? + quoted_start;
                return Some(&tag[quoted_start..quoted_end]);
            }
        }
        rest = &tag[candidate_end..];
    }
    None
}

fn parse_svg_length_px(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    let numeric_end = trimmed
        .char_indices()
        .find_map(|(index, ch)| {
            if ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+') {
                None
            } else {
                Some(index)
            }
        })
        .unwrap_or(trimmed.len());
    let number = trimmed[..numeric_end].trim().parse::<f64>().ok()?;
    let unit = trimmed[numeric_end..].trim().to_ascii_lowercase();
    let pixels = match unit.as_str() {
        "" | "px" => number,
        "pt" => number * 96.0 / 72.0,
        "in" => number * 96.0,
        "cm" => number * 96.0 / 2.54,
        "mm" => number * 96.0 / 25.4,
        _ => return None,
    };
    (pixels > 0.0).then_some(pixels)
}

fn parse_svg_view_box(value: &str) -> Option<(f64, f64, f64, f64)> {
    let values = value
        .split(|ch: char| ch == ',' || ch.is_ascii_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| part.parse::<f64>().ok())
        .collect::<Option<Vec<_>>>()?;
    if values.len() != 4 || values[2] <= 0.0 || values[3] <= 0.0 {
        return None;
    }
    Some((values[0], values[1], values[2], values[3]))
}

fn png_image_dimensions(bytes: &[u8]) -> Option<ExportImageDimensions> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || &bytes[..8] != PNG_SIGNATURE || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    image_dimensions_from_u32(width, height)
}

fn jpeg_image_dimensions(bytes: &[u8]) -> Option<ExportImageDimensions> {
    if bytes.len() < 4 || bytes[0] != 0xff || bytes[1] != 0xd8 {
        return None;
    }
    let mut index = 2;
    while index + 3 < bytes.len() {
        if bytes[index] != 0xff {
            index += 1;
            continue;
        }
        while index < bytes.len() && bytes[index] == 0xff {
            index += 1;
        }
        if index >= bytes.len() {
            return None;
        }
        let marker = bytes[index];
        index += 1;
        if marker == 0xda || marker == 0xd9 {
            return None;
        }
        if marker == 0x01 || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        if index + 1 >= bytes.len() {
            return None;
        }
        let segment_length = u16::from_be_bytes([bytes[index], bytes[index + 1]]) as usize;
        if segment_length < 2 || index + segment_length > bytes.len() {
            return None;
        }
        if is_jpeg_start_of_frame(marker) {
            if segment_length < 7 {
                return None;
            }
            let height = u16::from_be_bytes([bytes[index + 3], bytes[index + 4]]) as u32;
            let width = u16::from_be_bytes([bytes[index + 5], bytes[index + 6]]) as u32;
            return image_dimensions_from_u32(width, height);
        }
        index += segment_length;
    }
    None
}

fn is_jpeg_start_of_frame(marker: u8) -> bool {
    matches!(
        marker,
        0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc5 | 0xc6 | 0xc7 | 0xc9 | 0xca | 0xcb | 0xcd | 0xce | 0xcf
    )
}

fn image_dimensions_from_u32(width: u32, height: u32) -> Option<ExportImageDimensions> {
    if width == 0 || height == 0 {
        return None;
    }
    Some(ExportImageDimensions {
        width_px: width as f64,
        height_px: height as f64,
    })
}

fn decode_base64(input: &str) -> Option<Vec<u8>> {
    let mut bits = 0u32;
    let mut bit_count = 0u8;
    let mut output = Vec::new();
    for byte in input.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        if byte == b'=' {
            break;
        }
        let value = base64_value(byte)? as u32;
        bits = (bits << 6) | value;
        bit_count += 6;
        while bit_count >= 8 {
            bit_count -= 8;
            output.push(((bits >> bit_count) & 0xff) as u8);
        }
    }
    Some(output)
}

fn base64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
