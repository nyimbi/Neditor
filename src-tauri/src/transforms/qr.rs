use crate::{diag, escape_html, DocumentDiagnostic};

#[derive(Clone, Copy)]
struct QrVersionSpec {
    version: usize,
    data_codewords: usize,
    ecc_codewords: usize,
}

const QR_VERSION_SPECS: [QrVersionSpec; 4] = [
    QrVersionSpec {
        version: 1,
        data_codewords: 19,
        ecc_codewords: 7,
    },
    QrVersionSpec {
        version: 2,
        data_codewords: 34,
        ecc_codewords: 10,
    },
    QrVersionSpec {
        version: 3,
        data_codewords: 55,
        ecc_codewords: 15,
    },
    QrVersionSpec {
        version: 4,
        data_codewords: 80,
        ecc_codewords: 20,
    },
];

pub(crate) fn render_qr_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let payload = body.trim();
    if payload.is_empty() {
        let diagnostic = diag(
            "warning",
            "QR transform requires text or a URL.".to_string(),
            None,
            None,
            Some("Add the content to encode inside the qr fenced block."),
        );
        diagnostics.push(diagnostic.clone());
        artifact_diags.push(diagnostic);
        return "<section class=\"transform transform-qr\"><p>QR content missing.</p></section>"
            .to_string();
    }
    match render_qr_matrix(payload.as_bytes()) {
        Ok(matrix) => render_qr_matrix_svg(&matrix, payload),
        Err(message) => {
            let diagnostic = diag(
                "error",
                message.clone(),
                None,
                None,
                Some("Use shorter QR content or link to a compact URL."),
            );
            diagnostics.push(diagnostic.clone());
            artifact_diags.push(diagnostic);
            format!(
                "<section class=\"transform transform-qr\"><p>{}</p></section>",
                escape_html(&message)
            )
        }
    }
}

pub(crate) fn render_qr_matrix(payload: &[u8]) -> Result<Vec<Vec<bool>>, String> {
    let spec = QR_VERSION_SPECS
        .iter()
        .copied()
        .find(|spec| qr_payload_fits(payload.len(), spec.data_codewords))
        .ok_or_else(|| {
            format!(
                "QR content is too long for the native renderer: {} bytes exceeds {} bytes.",
                payload.len(),
                QR_VERSION_SPECS
                    .last()
                    .map(|spec| spec.data_codewords.saturating_sub(2))
                    .unwrap_or(0)
            )
        })?;
    let data = qr_data_codewords(payload, spec.data_codewords);
    let ecc = qr_reed_solomon_remainder(&data, spec.ecc_codewords);
    let mut codewords = data;
    codewords.extend(ecc);
    Ok(qr_build_matrix(spec.version, &codewords))
}

fn qr_payload_fits(payload_len: usize, data_codewords: usize) -> bool {
    payload_len + 2 <= data_codewords
}

fn qr_data_codewords(payload: &[u8], data_codewords: usize) -> Vec<u8> {
    let mut bits = Vec::new();
    qr_append_bits(&mut bits, 0b0100, 4);
    qr_append_bits(&mut bits, payload.len() as u32, 8);
    for byte in payload {
        qr_append_bits(&mut bits, u32::from(*byte), 8);
    }
    let capacity_bits = data_codewords * 8;
    let terminator = 4.min(capacity_bits.saturating_sub(bits.len()));
    qr_append_bits(&mut bits, 0, terminator);
    while bits.len() % 8 != 0 {
        bits.push(false);
    }
    let mut output = bits
        .chunks(8)
        .map(|chunk| {
            chunk.iter().fold(0u8, |acc, bit| {
                let shifted = acc << 1;
                if *bit {
                    shifted | 1
                } else {
                    shifted
                }
            })
        })
        .collect::<Vec<_>>();
    let mut pad = true;
    while output.len() < data_codewords {
        output.push(if pad { 0xEC } else { 0x11 });
        pad = !pad;
    }
    output
}

fn qr_append_bits(bits: &mut Vec<bool>, value: u32, count: usize) {
    for index in (0..count).rev() {
        bits.push(((value >> index) & 1) != 0);
    }
}

fn qr_reed_solomon_remainder(data: &[u8], degree: usize) -> Vec<u8> {
    let divisor = qr_reed_solomon_divisor(degree);
    let mut result = vec![0u8; degree];
    for byte in data {
        let factor = byte ^ result[0];
        result.rotate_left(1);
        if let Some(last) = result.last_mut() {
            *last = 0;
        }
        for (value, coefficient) in result.iter_mut().zip(divisor.iter()) {
            *value ^= qr_gf_multiply(*coefficient, factor);
        }
    }
    result
}

fn qr_reed_solomon_divisor(degree: usize) -> Vec<u8> {
    let mut result = vec![0u8; degree];
    if let Some(last) = result.last_mut() {
        *last = 1;
    }
    let mut root = 1u8;
    for _ in 0..degree {
        for index in 0..degree {
            result[index] = qr_gf_multiply(result[index], root);
            if index + 1 < degree {
                result[index] ^= result[index + 1];
            }
        }
        root = qr_gf_multiply(root, 0x02);
    }
    result
}

fn qr_gf_multiply(mut left: u8, mut right: u8) -> u8 {
    let mut result = 0u8;
    while right != 0 {
        if right & 1 != 0 {
            result ^= left;
        }
        let carry = left & 0x80 != 0;
        left <<= 1;
        if carry {
            left ^= 0x1D;
        }
        right >>= 1;
    }
    result
}

fn qr_build_matrix(version: usize, codewords: &[u8]) -> Vec<Vec<bool>> {
    let size = 21 + (version - 1) * 4;
    let mut modules = vec![vec![false; size]; size];
    let mut reserved = vec![vec![false; size]; size];
    qr_add_finder(&mut modules, &mut reserved, 0, 0);
    qr_add_finder(&mut modules, &mut reserved, size - 7, 0);
    qr_add_finder(&mut modules, &mut reserved, 0, size - 7);
    qr_add_timing(&mut modules, &mut reserved);
    qr_reserve_format_areas(&mut reserved);
    let dark_row = 4 * version + 9;
    qr_set_function(&mut modules, &mut reserved, 8, dark_row, true);
    qr_place_codewords(&mut modules, &reserved, codewords);
    qr_add_format_bits(&mut modules, &mut reserved, size);
    modules
}

fn qr_add_finder(modules: &mut [Vec<bool>], reserved: &mut [Vec<bool>], left: usize, top: usize) {
    let size = modules.len();
    let start_x = left.saturating_sub(1);
    let start_y = top.saturating_sub(1);
    let end_x = (left + 7).min(size - 1);
    let end_y = (top + 7).min(size - 1);
    for y in start_y..=end_y {
        for x in start_x..=end_x {
            reserved[y][x] = true;
            modules[y][x] = false;
        }
    }
    for dy in 0..7 {
        for dx in 0..7 {
            let x = left + dx;
            let y = top + dy;
            let black = dx == 0
                || dx == 6
                || dy == 0
                || dy == 6
                || ((2..=4).contains(&dx) && (2..=4).contains(&dy));
            qr_set_function(modules, reserved, x, y, black);
        }
    }
}

fn qr_add_timing(modules: &mut [Vec<bool>], reserved: &mut [Vec<bool>]) {
    let size = modules.len();
    for index in 8..(size - 8) {
        let black = index % 2 == 0;
        qr_set_function(modules, reserved, index, 6, black);
        qr_set_function(modules, reserved, 6, index, black);
    }
}

fn qr_reserve_format_areas(reserved: &mut [Vec<bool>]) {
    let size = reserved.len();
    for index in 0..9 {
        reserved[8][index] = true;
        reserved[index][8] = true;
    }
    for index in 0..8 {
        reserved[8][size - 1 - index] = true;
        reserved[size - 1 - index][8] = true;
    }
}

fn qr_set_function(
    modules: &mut [Vec<bool>],
    reserved: &mut [Vec<bool>],
    x: usize,
    y: usize,
    black: bool,
) {
    modules[y][x] = black;
    reserved[y][x] = true;
}

fn qr_place_codewords(modules: &mut [Vec<bool>], reserved: &[Vec<bool>], codewords: &[u8]) {
    let size = modules.len();
    let bits = codewords
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |index| ((byte >> index) & 1) != 0))
        .collect::<Vec<_>>();
    let mut bit_index = 0usize;
    let mut upward = true;
    let mut right = size - 1;
    while right > 0 {
        if right == 6 {
            right -= 1;
        }
        for vertical in 0..size {
            let y = if upward {
                size - 1 - vertical
            } else {
                vertical
            };
            for x in [right, right - 1] {
                if reserved[y][x] {
                    continue;
                }
                let mut bit = bits.get(bit_index).copied().unwrap_or(false);
                if (x + y) % 2 == 0 {
                    bit = !bit;
                }
                modules[y][x] = bit;
                bit_index += 1;
            }
        }
        upward = !upward;
        right = right.saturating_sub(2);
    }
}

fn qr_add_format_bits(modules: &mut [Vec<bool>], reserved: &mut [Vec<bool>], size: usize) {
    let format_bits = 0x77C4u16;
    for index in 0..=5 {
        qr_set_function(
            modules,
            reserved,
            8,
            index,
            qr_format_bit(format_bits, index),
        );
    }
    qr_set_function(modules, reserved, 8, 7, qr_format_bit(format_bits, 6));
    qr_set_function(modules, reserved, 8, 8, qr_format_bit(format_bits, 7));
    qr_set_function(modules, reserved, 7, 8, qr_format_bit(format_bits, 8));
    for index in 9..15 {
        qr_set_function(
            modules,
            reserved,
            14 - index,
            8,
            qr_format_bit(format_bits, index),
        );
    }
    for index in 0..8 {
        qr_set_function(
            modules,
            reserved,
            size - 1 - index,
            8,
            qr_format_bit(format_bits, index),
        );
    }
    for index in 8..15 {
        qr_set_function(
            modules,
            reserved,
            8,
            size - 15 + index,
            qr_format_bit(format_bits, index),
        );
    }
}

fn qr_format_bit(format_bits: u16, index: usize) -> bool {
    ((format_bits >> index) & 1) != 0
}

fn render_qr_matrix_svg(matrix: &[Vec<bool>], payload: &str) -> String {
    let quiet = 4usize;
    let module_size = 8usize;
    let modules = matrix.len() + quiet * 2;
    let size = modules * module_size;
    let mut rects = String::new();
    for (y, row) in matrix.iter().enumerate() {
        for (x, black) in row.iter().enumerate() {
            if *black {
                rects.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{module_size}\" height=\"{module_size}\"/>",
                    (x + quiet) * module_size,
                    (y + quiet) * module_size
                ));
            }
        }
    }
    format!(
        "<svg class=\"transform transform-qr\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {size} {size}\" role=\"img\"><title>QR code for {}</title><rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/><g fill=\"#111827\" shape-rendering=\"crispEdges\">{rects}</g></svg>",
        escape_html(payload)
    )
}
