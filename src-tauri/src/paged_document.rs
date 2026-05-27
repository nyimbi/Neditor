use crate::{
    document_ast::{AstSourceRange, DocumentAst, DocumentBlock},
    layout::LayoutSettings,
};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PagedDocument {
    pub(crate) sections: Vec<PagedSection>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PagedSection {
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) start_line: usize,
    pub(crate) end_line: usize,
    pub(crate) layout: LayoutSettings,
    pub(crate) blocks: Vec<PagedBlockRef>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PagedBlockRef {
    pub(crate) kind: String,
    pub(crate) line: usize,
    pub(crate) end_line: usize,
    pub(crate) source: Option<AstSourceRange>,
}

pub(crate) fn build_paged_document(ast: &DocumentAst) -> PagedDocument {
    let mut builder = PagedDocumentBuilder::new();
    for block in &ast.blocks {
        if let DocumentBlock::Layout {
            directive,
            settings,
            line,
            ..
        } = block
        {
            if directive == "section-break" {
                builder.start_section(settings.clone(), *line);
            } else if directive == "layout" {
                builder.apply_layout(settings);
            }
        }
        if let DocumentBlock::Heading { text, .. } = block {
            builder.set_title_if_empty(text);
        }
        builder.push_block(block);
    }
    builder.finish()
}

struct PagedDocumentBuilder {
    sections: Vec<PagedSection>,
    current: PagedSection,
    next_id: usize,
}

impl PagedDocumentBuilder {
    fn new() -> Self {
        Self {
            sections: Vec::new(),
            current: PagedSection {
                id: "section-1".to_string(),
                title: None,
                start_line: 1,
                end_line: 1,
                layout: LayoutSettings::default(),
                blocks: Vec::new(),
            },
            next_id: 2,
        }
    }

    fn start_section(&mut self, layout: LayoutSettings, line: usize) {
        if !self.current.blocks.is_empty() {
            self.sections.push(self.current.clone());
        }
        self.current = PagedSection {
            id: format!("section-{}", self.next_id),
            title: layout.title.clone(),
            start_line: line,
            end_line: line,
            layout,
            blocks: Vec::new(),
        };
        self.next_id += 1;
    }

    fn apply_layout(&mut self, settings: &LayoutSettings) {
        merge_layout_settings(&mut self.current.layout, settings);
        if self.current.title.is_none() {
            self.current.title = settings.title.clone();
        }
    }

    fn set_title_if_empty(&mut self, title: &str) {
        if self.current.title.is_none() {
            self.current.title = Some(title.to_string());
        }
    }

    fn push_block(&mut self, block: &DocumentBlock) {
        let reference = paged_block_ref(block);
        if self.current.blocks.is_empty() {
            self.current.start_line = reference.line;
        }
        self.current.end_line = self.current.end_line.max(reference.end_line);
        self.current.blocks.push(reference);
    }

    fn finish(mut self) -> PagedDocument {
        if !self.current.blocks.is_empty() || self.sections.is_empty() {
            self.sections.push(self.current);
        }
        PagedDocument {
            sections: self.sections,
        }
    }
}

fn merge_layout_settings(target: &mut LayoutSettings, update: &LayoutSettings) {
    if update.columns.is_some() {
        target.columns = update.columns;
    }
    if update.column_gap.is_some() {
        target.column_gap = update.column_gap.clone();
    }
    if update.page_size.is_some() {
        target.page_size = update.page_size.clone();
    }
    if update.orientation.is_some() {
        target.orientation = update.orientation.clone();
    }
    if update.margins.is_some() {
        target.margins = update.margins.clone();
    }
    if update.break_before.is_some() {
        target.break_before = update.break_before.clone();
    }
    if update.break_after.is_some() {
        target.break_after = update.break_after.clone();
    }
    target.keep_with_next |= update.keep_with_next;
    target.keep_together |= update.keep_together;
    if update.header.is_some() {
        target.header = update.header.clone();
    }
    if update.footer.is_some() {
        target.footer = update.footer.clone();
    }
    if update.title.is_some() {
        target.title = update.title.clone();
    }
    if update.layout.is_some() {
        target.layout = update.layout.clone();
    }
    if update.notes.is_some() {
        target.notes = update.notes.clone();
    }
}

fn paged_block_ref(block: &DocumentBlock) -> PagedBlockRef {
    let (kind, line, end_line, source) = match block {
        DocumentBlock::Heading {
            line,
            end_line,
            source,
            ..
        } => ("heading", *line, *end_line, source.clone()),
        DocumentBlock::Paragraph {
            line,
            end_line,
            source,
            ..
        } => ("paragraph", *line, *end_line, source.clone()),
        DocumentBlock::List {
            line,
            end_line,
            source,
            ..
        } => ("list", *line, *end_line, source.clone()),
        DocumentBlock::TaskList {
            line,
            end_line,
            source,
            ..
        } => ("task_list", *line, *end_line, source.clone()),
        DocumentBlock::BlockQuote {
            line,
            end_line,
            source,
            ..
        } => ("block_quote", *line, *end_line, source.clone()),
        DocumentBlock::CodeBlock {
            line,
            end_line,
            source,
            ..
        } => ("code_block", *line, *end_line, source.clone()),
        DocumentBlock::Table {
            line,
            end_line,
            source,
            ..
        } => ("table", *line, *end_line, source.clone()),
        DocumentBlock::Figure {
            line,
            end_line,
            source,
            ..
        } => ("figure", *line, *end_line, source.clone()),
        DocumentBlock::Equation {
            line,
            end_line,
            source,
            ..
        } => ("equation", *line, *end_line, source.clone()),
        DocumentBlock::Layout {
            line,
            end_line,
            source,
            ..
        } => ("layout", *line, *end_line, source.clone()),
        DocumentBlock::Callout {
            line,
            end_line,
            source,
            ..
        } => ("callout", *line, *end_line, source.clone()),
        DocumentBlock::Footnotes {
            line,
            end_line,
            source,
            ..
        } => ("footnotes", *line, *end_line, source.clone()),
        DocumentBlock::ReviewComment {
            line,
            end_line,
            source,
            ..
        } => ("review_comment", *line, *end_line, source.clone()),
        DocumentBlock::ChangeNote {
            line,
            end_line,
            source,
            ..
        } => ("change_note", *line, *end_line, source.clone()),
        DocumentBlock::AiSource {
            line,
            end_line,
            source,
            ..
        } => ("ai_source", *line, *end_line, source.clone()),
        DocumentBlock::Transform {
            line,
            end_line,
            source,
            ..
        } => ("transform", *line, *end_line, source.clone()),
        DocumentBlock::RawHtml {
            line,
            end_line,
            source,
            ..
        } => ("raw_html", *line, *end_line, source.clone()),
    };
    PagedBlockRef {
        kind: kind.to_string(),
        line,
        end_line,
        source,
    }
}
