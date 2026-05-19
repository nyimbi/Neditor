use super::*;

pub(crate) fn render_pdf_bytes(response: &CompileResponse, options: &Value) -> Vec<u8> {
    let pages = build_pdf_pages(response, options);
    let (page_width, page_height, margin_top, margin_left) = pdf_page_layout(response, options);
    let total_pages = pages.len().max(1);
    let mut objects = vec![
        String::new(),
        String::new(),
        "3 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj\n".to_string(),
    ];
    let mut page_ids = Vec::new();

    for (page_index, page) in pages.iter().enumerate() {
        let page_id = objects.len() + 1;
        let content_id = page_id + 1;
        page_ids.push(page_id);
        let (mut header, mut footer) =
            export_header_footer_for_page(response, options, page_index + 1, total_pages);
        if let Some(section_header) = &page.header {
            header = render_section_template(response, section_header, page_index + 1, total_pages);
        }
        if let Some(section_footer) = &page.footer {
            footer = render_section_template(response, section_footer, page_index + 1, total_pages);
        }
        let mut stream = String::new();
        let mut y = page_height.saturating_sub(margin_top) as i32;
        if !header.trim().is_empty() {
            stream.push_str(&pdf_text_line(
                9,
                margin_left,
                page_height.saturating_sub((margin_top / 2).max(12)) as i32,
                &header,
            ));
        }
        for item in page.items.iter().take(60) {
            match item {
                PdfPageItem::Text(line) => {
                    stream.push_str(&pdf_text_line(10, margin_left, y, line));
                    y -= 12;
                }
                PdfPageItem::Table(table) => {
                    let (table_stream, consumed_height) =
                        pdf_table_stream(table, margin_left, y, page_width - margin_left * 2);
                    stream.push_str(&table_stream);
                    y -= consumed_height;
                }
                PdfPageItem::Figure(figure) => {
                    let (figure_stream, consumed_height) =
                        pdf_figure_stream(figure, margin_left, y, page_width - margin_left * 2);
                    stream.push_str(&figure_stream);
                    y -= consumed_height;
                }
            }
        }
        if !footer.trim().is_empty() {
            stream.push_str(&pdf_text_line(
                9,
                margin_left,
                (margin_top / 2).max(12) as i32,
                &footer,
            ));
        }
        objects.push(format!(
            "{page_id} 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 {page_width} {page_height}] /Resources << /Font << /F1 3 0 R >> >> /Contents {content_id} 0 R >> endobj\n"
        ));
        objects.push(format!(
            "{content_id} 0 obj << /Length {} >> stream\n{}endstream endobj\n",
            stream.len(),
            stream
        ));
    }

    let kids = page_ids
        .iter()
        .map(|id| format!("{id} 0 R"))
        .collect::<Vec<_>>()
        .join(" ");
    objects[0] = "1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n".to_string();
    objects[1] = format!(
        "2 0 obj << /Type /Pages /Kids [{kids}] /Count {} >> endobj\n",
        page_ids.len()
    );
    let info_id = objects.len() + 1;
    objects.push(render_pdf_info_object(info_id, response));

    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for object in &objects {
        offsets.push(pdf.len());
        pdf.extend_from_slice(object.as_bytes());
    }
    let xref = pdf.len();
    pdf.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer << /Size {} /Root 1 0 R /Info {info_id} 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref
        )
        .as_bytes(),
    );
    pdf
}

fn pdf_text_line(font_size: u8, x: u32, y: i32, text: &str) -> String {
    format!(
        "BT /F1 {font_size} Tf {x} {y} Td ({}) Tj ET\n",
        escape_pdf(text)
    )
}

fn pdf_table_stream(table: &PdfTable, x: u32, top_y: i32, width: u32) -> (String, i32) {
    let mut stream = String::new();
    let caption = table_export_line(&table.id, &table.caption, &table.headers);
    stream.push_str(&pdf_text_line(10, x, top_y, &caption));
    let mut current_y = top_y - 18;
    let column_count = table.headers.len().max(1);
    let column_width = (width / column_count as u32).max(48);
    let row_height = 18i32;

    stream.push_str(&pdf_table_row_stream(
        &table.headers,
        &table.alignments,
        x,
        current_y,
        column_width,
        row_height,
    ));
    current_y -= row_height;
    for row in &table.rows {
        stream.push_str(&pdf_table_row_stream(
            row,
            &table.alignments,
            x,
            current_y,
            column_width,
            row_height,
        ));
        current_y -= row_height;
    }

    let consumed = (top_y - current_y + 10).max(28);
    (stream, consumed)
}

fn pdf_figure_stream(figure: &PdfFigure, x: u32, top_y: i32, max_width: u32) -> (String, i32) {
    let (width, height) =
        pdf_figure_box_size(figure.dimensions, max_width, 180, figure.fit.as_deref());
    let x = pdf_aligned_x(x, max_width, width, figure.float.as_deref());
    let bottom_y = top_y - height;
    let label = figure
        .alt
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Figure");
    let mut stream = format!("{x} {bottom_y} {width} {height} re S\n");
    stream.push_str(&pdf_text_line(
        8,
        (x + 8).max(0) as u32,
        bottom_y + (height / 2),
        label,
    ));
    stream.push_str(&pdf_text_line(
        10,
        x.max(0) as u32,
        bottom_y - 14,
        &figure.caption_line,
    ));
    (stream, height + pdf_figure_caption_height())
}

fn pdf_aligned_x(base_x: u32, max_width: u32, width: i32, float: Option<&str>) -> i32 {
    let base_x = base_x as i32;
    let remaining = max_width as i32 - width;
    match normalized_float(float) {
        Some("right") => base_x + remaining.max(0),
        Some("center") => base_x + (remaining.max(0) / 2),
        _ => base_x,
    }
}

fn pdf_figure_box_size(
    dimensions: Option<ExportImageDimensions>,
    max_width: u32,
    max_height: i32,
    fit: Option<&str>,
) -> (i32, i32) {
    let fallback = (240, 135);
    if normalized_fit(fit) == Some("cover") {
        return fallback;
    }
    let Some(dimensions) = dimensions else {
        return fallback;
    };
    let width = dimensions.width_px * 72.0 / 96.0;
    let height = dimensions.height_px * 72.0 / 96.0;
    if width <= 0.0 || height <= 0.0 {
        return fallback;
    }
    let scale = (max_width as f64 / width)
        .min(max_height as f64 / height)
        .min(1.0);
    let scaled_width = (width * scale).round() as i32;
    let scaled_height = (height * scale).round() as i32;
    if scaled_width <= 0 || scaled_height <= 0 {
        fallback
    } else {
        (scaled_width, scaled_height)
    }
}

fn pdf_table_row_stream(
    cells: &[String],
    alignments: &[String],
    x: u32,
    y: i32,
    column_width: u32,
    row_height: i32,
) -> String {
    let mut stream = String::new();
    let column_count = cells.len().max(1);
    for index in 0..column_count {
        let cell_x = x + (index as u32 * column_width);
        let cell = cells.get(index).map(String::as_str).unwrap_or("");
        stream.push_str(&format!("{cell_x} {y} {column_width} {row_height} re S\n"));
        let text_x = pdf_table_cell_text_x(
            cell_x,
            column_width,
            cell,
            alignments.get(index).map(String::as_str),
        );
        stream.push_str(&pdf_text_line(8, text_x, y + 6, cell));
    }
    stream
}

fn pdf_table_cell_text_x(
    cell_x: u32,
    column_width: u32,
    text: &str,
    alignment: Option<&str>,
) -> u32 {
    match alignment {
        Some("center") => {
            let text_width = approximate_pdf_text_width(text, 8);
            cell_x + column_width.saturating_sub(text_width) / 2
        }
        Some("right") => {
            let text_width = approximate_pdf_text_width(text, 8);
            cell_x + column_width.saturating_sub(text_width + 4)
        }
        _ => cell_x + 4,
    }
}

fn approximate_pdf_text_width(text: &str, font_size: u32) -> u32 {
    (text.chars().count() as u32)
        .saturating_mul(font_size)
        .saturating_div(2)
        .max(4)
}

fn render_pdf_info_object(object_id: usize, response: &CompileResponse) -> String {
    let author = metadata_string(&response.metadata, "author")
        .or_else(|| metadata_string(&response.metadata, "approvedBy"))
        .unwrap_or_else(|| "NEditor".to_string());
    let version = metadata_string(&response.metadata, "version").unwrap_or_default();
    let classification = metadata_string(&response.metadata, "classification").unwrap_or_default();
    let keywords = [
        response.semantic.status.as_str(),
        version.as_str(),
        classification.as_str(),
    ]
    .into_iter()
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join("; ");
    format!(
        "{object_id} 0 obj << /Title ({}) /Author ({}) /Subject ({}) /Keywords ({}) /Producer (NEditor) >> endobj\n",
        escape_pdf(&response.semantic.title),
        escape_pdf(&author),
        escape_pdf(&format!("Status: {}", response.semantic.status)),
        escape_pdf(&keywords)
    )
}

#[derive(Clone, Debug)]
enum PdfPageItem {
    Text(String),
    Table(PdfTable),
    Figure(PdfFigure),
}

#[derive(Clone, Debug)]
struct PdfPage {
    items: Vec<PdfPageItem>,
    header: Option<String>,
    footer: Option<String>,
}

#[derive(Clone, Debug)]
struct PdfTable {
    id: Option<String>,
    caption: Option<String>,
    headers: Vec<String>,
    alignments: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug)]
struct PdfFigure {
    caption_line: String,
    alt: Option<String>,
    float: Option<String>,
    fit: Option<String>,
    dimensions: Option<ExportImageDimensions>,
}

fn build_pdf_pages(response: &CompileResponse, options: &Value) -> Vec<PdfPage> {
    let (_, page_height, margin_top, _) = pdf_page_layout(response, options);
    let footer_reserved = (margin_top / 2).max(12) + 24;
    let available_height =
        (page_height as i32 - margin_top as i32 - footer_reserved as i32).max(120);
    let mut paginator = PdfPaginator::new(available_height);
    paginator.extend_text(export_metadata_lines(response, options));
    paginator.finish_page();

    for block in &response.document_ast.blocks {
        match block {
            DocumentBlock::Layout { directive, .. } if directive == "page-break" => {
                paginator.finish_page();
            }
            DocumentBlock::Layout {
                directive,
                options,
                settings,
                ..
            } if directive == "section-break" => {
                paginator.finish_page();
                paginator.apply_section_options(settings);
                paginator.extend_text(layout_export_lines(directive, options, settings));
                if matches_layout_break(settings.break_after.as_deref()) {
                    paginator.finish_page();
                }
            }
            DocumentBlock::Layout {
                directive,
                options,
                settings,
                ..
            } if directive == "layout" => {
                if matches_layout_break(settings.break_before.as_deref()) {
                    paginator.finish_page();
                }
                paginator.apply_section_options(settings);
                paginator.extend_text(layout_export_lines(directive, options, settings));
                if matches_layout_break(settings.break_after.as_deref()) {
                    paginator.finish_page();
                }
            }
            _ => paginator.extend_items(block_pdf_items(block)),
        }
    }
    paginator.finish_page();
    for appendix in appendix_pages(response, options) {
        paginator.extend_text(appendix);
        paginator.finish_page();
    }
    paginator.into_pages()
}

fn block_pdf_items(block: &DocumentBlock) -> Vec<PdfPageItem> {
    if let DocumentBlock::RawHtml { html, .. } = block {
        if let Some(table) = export_table_from_transform_html(html) {
            return vec![PdfPageItem::Table(PdfTable {
                id: None,
                caption: None,
                headers: table.headers,
                alignments: table.alignments,
                rows: table.rows,
            })];
        }
    }
    if let DocumentBlock::CodeBlock { language, code, .. } = block {
        if let Some(table) = export_table_from_delimited_code(language.as_deref(), code) {
            return vec![PdfPageItem::Table(PdfTable {
                id: None,
                caption: None,
                headers: table.headers,
                alignments: table.alignments,
                rows: table.rows,
            })];
        }
    }
    if let DocumentBlock::Table {
        id,
        caption,
        headers,
        alignments,
        rows,
        ..
    } = block
    {
        return vec![PdfPageItem::Table(PdfTable {
            id: id.clone(),
            caption: caption.clone(),
            headers: headers.clone(),
            alignments: alignments.clone(),
            rows: rows.clone(),
        })];
    }
    if let DocumentBlock::Figure {
        id,
        src,
        alt,
        caption,
        float,
        fit,
        position,
        source,
        ..
    } = block
    {
        let dimensions = src.as_deref().and_then(|src| {
            parse_export_image(src, source.as_ref()).and_then(|image| image.dimensions)
        });
        return vec![PdfPageItem::Figure(PdfFigure {
            caption_line: figure_export_line(id, src, alt, caption, float, fit, position),
            alt: alt.clone(),
            float: normalized_float(float.as_deref()).map(str::to_string),
            fit: normalized_fit(fit.as_deref()).map(str::to_string),
            dimensions,
        })];
    }
    pdf_text_items(block_export_lines(block))
}

fn pdf_text_items(lines: Vec<String>) -> Vec<PdfPageItem> {
    lines.into_iter().map(PdfPageItem::Text).collect()
}

struct PdfPaginator {
    pages: Vec<PdfPage>,
    current: Vec<PdfPageItem>,
    current_header: Option<String>,
    current_footer: Option<String>,
    used_height: i32,
    available_height: i32,
}

impl PdfPaginator {
    fn new(available_height: i32) -> Self {
        Self {
            pages: Vec::new(),
            current: Vec::new(),
            current_header: None,
            current_footer: None,
            used_height: 0,
            available_height,
        }
    }

    fn apply_section_options(&mut self, settings: &LayoutSettings) {
        if let Some(header) = &settings.header {
            self.current_header = Some(header.clone());
        }
        if let Some(footer) = &settings.footer {
            self.current_footer = Some(footer.clone());
        }
    }

    fn extend_text(&mut self, lines: Vec<String>) {
        self.extend_items(pdf_text_items(lines));
    }

    fn extend_items(&mut self, items: Vec<PdfPageItem>) {
        for item in items {
            match item {
                PdfPageItem::Text(line) => self.push_text(line),
                PdfPageItem::Table(table) => self.push_table(table),
                PdfPageItem::Figure(figure) => self.push_figure(figure),
            }
        }
    }

    fn push_text(&mut self, line: String) {
        let height = pdf_text_item_height();
        if self.used_height + height > self.available_height {
            self.finish_page();
        }
        self.used_height += height;
        self.current.push(PdfPageItem::Text(line));
    }

    fn push_table(&mut self, table: PdfTable) {
        let mut remaining_rows = table.rows.as_slice();
        let mut continued = false;
        while !remaining_rows.is_empty() {
            let available_rows = self.available_table_rows(continued);
            if available_rows == 0 {
                self.finish_page();
                continue;
            }
            let take_count = remaining_rows.len().min(available_rows);
            let chunk = pdf_table_chunk(&table, remaining_rows[..take_count].to_vec(), continued);
            let height = pdf_table_height(&chunk);
            if self.used_height + height > self.available_height && !self.current.is_empty() {
                self.finish_page();
                continue;
            }
            self.used_height += height;
            self.current.push(PdfPageItem::Table(chunk));
            remaining_rows = &remaining_rows[take_count..];
            continued = true;
        }

        if table.rows.is_empty() {
            let height = pdf_table_height(&table);
            if self.used_height + height > self.available_height {
                self.finish_page();
            }
            self.used_height += height;
            self.current.push(PdfPageItem::Table(table));
        }
    }

    fn push_figure(&mut self, figure: PdfFigure) {
        let height = pdf_figure_height(figure.dimensions, figure.fit.as_deref());
        if self.used_height + height > self.available_height && !self.current.is_empty() {
            self.finish_page();
        }
        self.used_height += height.min(self.available_height);
        self.current.push(PdfPageItem::Figure(figure));
    }

    fn available_table_rows(&self, continued: bool) -> usize {
        let remaining = self.available_height - self.used_height;
        let caption_height = if continued {
            pdf_table_continued_caption_height()
        } else {
            pdf_table_caption_height()
        };
        let available_for_rows = remaining - caption_height - pdf_table_header_height() - 10;
        if available_for_rows < pdf_table_row_height() {
            return 0;
        }
        (available_for_rows / pdf_table_row_height()) as usize
    }

    fn finish_page(&mut self) {
        if self.current.is_empty() {
            return;
        }
        self.pages.push(PdfPage {
            items: std::mem::take(&mut self.current),
            header: self.current_header.clone(),
            footer: self.current_footer.clone(),
        });
        self.used_height = 0;
    }

    fn into_pages(mut self) -> Vec<PdfPage> {
        self.finish_page();
        if self.pages.is_empty() {
            self.pages.push(PdfPage {
                items: Vec::new(),
                header: self.current_header.clone(),
                footer: self.current_footer.clone(),
            });
        }
        self.pages
    }
}

fn pdf_table_chunk(table: &PdfTable, rows: Vec<Vec<String>>, continued: bool) -> PdfTable {
    let caption = if continued {
        Some(table.caption.clone().unwrap_or_else(|| "Table".to_string()) + " (continued)")
    } else {
        table.caption.clone()
    };
    PdfTable {
        id: table.id.clone(),
        caption,
        headers: table.headers.clone(),
        alignments: table.alignments.clone(),
        rows,
    }
}

fn pdf_text_item_height() -> i32 {
    12
}

fn pdf_table_caption_height() -> i32 {
    18
}

fn pdf_table_continued_caption_height() -> i32 {
    18
}

fn pdf_figure_caption_height() -> i32 {
    26
}

fn pdf_table_header_height() -> i32 {
    18
}

fn pdf_table_row_height() -> i32 {
    18
}

fn pdf_table_height(table: &PdfTable) -> i32 {
    pdf_table_caption_height()
        + pdf_table_header_height()
        + (table.rows.len() as i32 * pdf_table_row_height())
        + 10
}

fn pdf_figure_height(dimensions: Option<ExportImageDimensions>, fit: Option<&str>) -> i32 {
    let (_, height) = pdf_figure_box_size(dimensions, 468, 180, fit);
    height + pdf_figure_caption_height()
}

fn pdf_page_layout(response: &CompileResponse, options: &Value) -> (u32, u32, u32, u32) {
    let (mut width, mut height) = match layout_page_size(&response.metadata).as_str() {
        "letter" => (612, 792),
        "legal" => (612, 1008),
        _ => (595, 842),
    };
    if layout_orientation(&response.metadata) == "landscape" {
        std::mem::swap(&mut width, &mut height);
    }
    let margin = match explicit_layout_margins(&response.metadata).as_deref() {
        Some("narrow") | Some("compact") => 34,
        Some("wide") => 91,
        Some("normal") => 68,
        _ => match layout_preset(options) {
            "compact" => 51,
            "presentation" => 57,
            _ => 68,
        },
    };
    (width, height, margin, margin)
}
