use crate::{
    diagnostics::{diag, DocumentDiagnostic},
    document_ast::extract_quoted_attribute,
    escape_html,
};
use std::collections::{HashMap, HashSet};

pub(crate) fn render_mermaid_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let graph = parse_mermaid_flowchart(body);
    if graph.nodes.is_empty() || graph.edges.is_empty() {
        let diagnostic = diag(
            "warning",
            "Mermaid native preview only supports simple flowchart edges.",
            None,
            None,
            Some("Use flowchart or graph syntax with edges such as A[Start] --> B[End]."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-mermaid transform-error\">Unsupported Mermaid diagram</section>".to_string();
    }
    let columns = 3usize;
    let node_width = 170usize;
    let node_height = 54usize;
    let x_gap = 250usize;
    let y_gap = 120usize;
    let rows = graph.nodes.len().div_ceil(columns);
    let width = 120 + columns * x_gap;
    let height = 90 + rows * y_gap;
    let positions = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let x = 60 + (index % columns) * x_gap;
            let y = 55 + (index / columns) * y_gap;
            (node.id.clone(), (x, y))
        })
        .collect::<HashMap<_, _>>();
    let mut svg = format!(
        "<svg class=\"transform transform-mermaid\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" role=\"img\"><defs><marker id=\"arrow\" markerWidth=\"10\" markerHeight=\"10\" refX=\"8\" refY=\"3\" orient=\"auto\" markerUnits=\"strokeWidth\"><path d=\"M0,0 L0,6 L9,3 z\" fill=\"#275DA8\"/></marker></defs>"
    );
    for edge in &graph.edges {
        if let (Some((from_x, from_y)), Some((to_x, to_y))) =
            (positions.get(&edge.from), positions.get(&edge.to))
        {
            let x1 = from_x + node_width;
            let y1 = from_y + node_height / 2;
            let x2 = *to_x;
            let y2 = to_y + node_height / 2;
            svg.push_str(&format!(
                "<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"#275DA8\" stroke-width=\"3\" marker-end=\"url(#arrow)\"/>"
            ));
            render_edge_label(&mut svg, x1, y1, x2, y2, edge.label.as_deref());
        }
    }
    for node in &graph.nodes {
        if let Some((x, y)) = positions.get(&node.id) {
            svg.push_str(&format!(
                "<rect x=\"{x}\" y=\"{y}\" width=\"{node_width}\" height=\"{node_height}\" rx=\"8\" fill=\"#eff6ff\" stroke=\"#275DA8\" stroke-width=\"2\"/><text x=\"{}\" y=\"{}\" font-size=\"15\" text-anchor=\"middle\" fill=\"#1f2937\">{}</text>",
                x + node_width / 2,
                y + 33,
                escape_html(&node.label)
            ));
        }
    }
    svg.push_str("</svg>");
    svg
}

// ═══════════════════════════════════════════════════════════════════════
// Pikchr native fallback renderer — spec 9.19
//
// Covers: box / circle / ellipse / oval / cylinder / diamond / file shapes;
// arrow / line / spline connectors with right/left/up/down direction and
// optional "then" multi-segments; move; standalone text; fill / color /
// stroke colors (named + 0xRRGGBB / #RRGGBB); dashed / dotted / thick /
// thin line styles; above / below / ljust / rjust text placement; named
// object variables (A: box "…"); from ObjRef.compass to ObjRef.compass
// and at ObjRef.compass positioning; # and // comment stripping.
//
// Uses a 2-D cursor model: cursor starts at (0,0), shapes are placed with
// their entry edge at the cursor, and the cursor advances to the exit edge.
// ═══════════════════════════════════════════════════════════════════════

/// Screen pixels per Pikchr inch (Pikchr's default unit is inches).
const PK_INCH: f32 = 160.0;
/// Default arrow / line segment length (0.5 in).
const PK_ARROW_LEN: f32 = 0.5 * PK_INCH;
/// Default move distance (0.5 in).
const PK_MOVE_LEN: f32 = 0.5 * PK_INCH;
/// Canvas padding around the bounding box.
const PK_PAD: f32 = 20.0;

// ─── Shape kind ────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
enum PkShape {
    Box,
    Circle,
    Ellipse,
    Oval,
    Cylinder,
    Diamond,
    File,
}

impl PkShape {
    /// Default (half-width, half-height) in pixels.
    fn half_size(self) -> (f32, f32) {
        match self {
            PkShape::Box | PkShape::Diamond => (60.0, 38.0),
            PkShape::Circle => (38.0, 38.0),
            PkShape::Ellipse => (60.0, 32.0),
            PkShape::Oval => (60.0, 22.0),
            PkShape::Cylinder => (48.0, 44.0),
            PkShape::File => (55.0, 40.0),
        }
    }

    fn css_class(self) -> &'static str {
        match self {
            PkShape::Box => "pikchr-box",
            PkShape::Circle | PkShape::Ellipse | PkShape::Oval => "pikchr-circle",
            PkShape::Cylinder => "pikchr-cylinder",
            PkShape::Diamond => "pikchr-diamond",
            PkShape::File => "pikchr-file",
        }
    }
}

// ─── Direction ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum PkDir {
    #[default]
    Right,
    Left,
    Up,
    Down,
}

impl PkDir {
    fn vec(self) -> (f32, f32) {
        match self {
            PkDir::Right => (1.0, 0.0),
            PkDir::Left => (-1.0, 0.0),
            PkDir::Up => (0.0, -1.0),
            PkDir::Down => (0.0, 1.0),
        }
    }

    fn opposite(self) -> Self {
        match self {
            PkDir::Right => PkDir::Left,
            PkDir::Left => PkDir::Right,
            PkDir::Up => PkDir::Down,
            PkDir::Down => PkDir::Up,
        }
    }
}

fn parse_pk_dir(s: &str) -> Option<PkDir> {
    match s.to_ascii_lowercase().as_str() {
        "right" => Some(PkDir::Right),
        "left" => Some(PkDir::Left),
        "up" => Some(PkDir::Up),
        "down" => Some(PkDir::Down),
        _ => None,
    }
}

// ─── Text placement ────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum PkPlace {
    #[default]
    Center,
    Above,
    Below,
    Ljust,
    Rjust,
}

fn parse_pk_place(s: &str) -> Option<PkPlace> {
    match s.to_ascii_lowercase().as_str() {
        "above" => Some(PkPlace::Above),
        "below" => Some(PkPlace::Below),
        "ljust" => Some(PkPlace::Ljust),
        "rjust" => Some(PkPlace::Rjust),
        _ => None,
    }
}

// ─── Colors ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq)]
enum PkColor {
    #[default]
    Default,
    None,
    Rgb(u8, u8, u8),
}

impl PkColor {
    fn to_svg(&self, default: &str) -> String {
        match self {
            PkColor::Default => default.to_string(),
            PkColor::None => "none".to_string(),
            PkColor::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        }
    }
}

fn parse_pk_color(s: &str) -> PkColor {
    match s.to_ascii_lowercase().as_str() {
        "none" | "transparent" => PkColor::None,
        "red" => PkColor::Rgb(220, 38, 38),
        "green" => PkColor::Rgb(22, 163, 74),
        "blue" => PkColor::Rgb(37, 99, 235),
        "white" => PkColor::Rgb(255, 255, 255),
        "black" => PkColor::Rgb(0, 0, 0),
        "gray" | "grey" => PkColor::Rgb(107, 114, 128),
        "lightgray" | "lightgrey" => PkColor::Rgb(209, 213, 219),
        "darkgray" | "darkgrey" => PkColor::Rgb(75, 85, 99),
        "yellow" => PkColor::Rgb(234, 179, 8),
        "orange" => PkColor::Rgb(249, 115, 22),
        "purple" => PkColor::Rgb(147, 51, 234),
        "pink" => PkColor::Rgb(236, 72, 153),
        "cyan" | "aqua" => PkColor::Rgb(6, 182, 212),
        "brown" => PkColor::Rgb(146, 64, 14),
        _ => {
            let hex = s.strip_prefix("0x").or_else(|| s.strip_prefix('#'));
            if let Some(h) = hex {
                if h.len() == 6 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&h[0..2], 16),
                        u8::from_str_radix(&h[2..4], 16),
                        u8::from_str_radix(&h[4..6], 16),
                    ) {
                        return PkColor::Rgb(r, g, b);
                    }
                }
            }
            PkColor::Default
        }
    }
}

// ─── Style ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
struct PkStyle {
    fill: PkColor,
    stroke: PkColor,
    dashed: bool,
    dotted: bool,
    thickness: Option<f32>,
}

impl PkStyle {
    fn sw(&self, default: f32) -> f32 {
        self.thickness.unwrap_or(default)
    }

    fn dash_attr(&self) -> String {
        if self.dotted {
            " stroke-dasharray=\"1 6\" stroke-linecap=\"round\"".to_string()
        } else if self.dashed {
            " stroke-dasharray=\"8 4\"".to_string()
        } else {
            String::new()
        }
    }
}

// ─── Canvas items ──────────────────────────────────────────────────────

struct PkShapeItem {
    shape: PkShape,
    cx: f32,
    cy: f32,
    hw: f32,
    hh: f32,
    labels: Vec<(String, PkPlace)>,
    style: PkStyle,
}

struct PkLineItem {
    /// Two or more (x,y) waypoints; arrowhead at last point when arrow_end.
    points: Vec<(f32, f32)>,
    arrow_end: bool,
    arrow_start: bool,
    label: Option<(String, PkPlace)>,
    style: PkStyle,
}

struct PkTextItem {
    x: f32,
    y: f32,
    content: String,
    placement: PkPlace,
}

enum PkItem {
    Shape(PkShapeItem),
    Line(PkLineItem),
    Text(PkTextItem),
}

// ─── Named-object registry (for from/to/at) ────────────────────────────

struct PkObj {
    cx: f32,
    cy: f32,
    hw: f32,
    hh: f32,
}

impl PkObj {
    fn compass(&self, pt: &str) -> (f32, f32) {
        let (cx, cy, hw, hh) = (self.cx, self.cy, self.hw, self.hh);
        match pt {
            "n" | "north" | "top" => (cx, cy - hh),
            "s" | "south" | "bottom" => (cx, cy + hh),
            "e" | "east" | "right" => (cx + hw, cy),
            "w" | "west" | "left" => (cx - hw, cy),
            "ne" => (cx + hw, cy - hh),
            "nw" => (cx - hw, cy - hh),
            "se" => (cx + hw, cy + hh),
            "sw" => (cx - hw, cy + hh),
            _ => (cx, cy), // "center", "c", or unknown
        }
    }

    fn exit(&self, dir: PkDir) -> (f32, f32) {
        match dir {
            PkDir::Right => self.compass("e"),
            PkDir::Left => self.compass("w"),
            PkDir::Up => self.compass("n"),
            PkDir::Down => self.compass("s"),
        }
    }
}

// ─── Layout state ──────────────────────────────────────────────────────

struct PkState {
    lx: f32,
    ly: f32,
    dir: PkDir,
    names: HashMap<String, PkObj>,
    items: Vec<PkItem>,
}

impl PkState {
    fn new() -> Self {
        Self {
            lx: 0.0,
            ly: 0.0,
            dir: PkDir::Right,
            names: HashMap::new(),
            items: Vec::new(),
        }
    }

    fn resolve(&self, r: &str) -> Option<(f32, f32)> {
        let (obj, compass) = r.split_once('.').unwrap_or((r, "center"));
        self.names.get(obj).map(|o| o.compass(compass))
    }

    fn place_shape(
        &mut self,
        shape: PkShape,
        hw: f32,
        hh: f32,
        labels: Vec<(String, PkPlace)>,
        style: PkStyle,
        name: Option<&str>,
        at: Option<(f32, f32)>,
    ) {
        let (dx, dy) = self.dir.vec();
        let (cx, cy) = at.unwrap_or((self.lx + dx * hw, self.ly + dy * hh));
        let obj = PkObj { cx, cy, hw, hh };
        let (ex, ey) = obj.exit(self.dir);
        self.lx = ex;
        self.ly = ey;
        if let Some(n) = name {
            self.names.insert(n.to_string(), PkObj { cx, cy, hw, hh });
        }
        self.items.push(PkItem::Shape(PkShapeItem {
            shape,
            cx,
            cy,
            hw,
            hh,
            labels,
            style,
        }));
    }

    fn place_line(
        &mut self,
        dir: PkDir,
        len: f32,
        arrow_end: bool,
        arrow_start: bool,
        label: Option<(String, PkPlace)>,
        style: PkStyle,
        from: Option<(f32, f32)>,
        to: Option<(f32, f32)>,
        then_segs: Vec<(PkDir, f32)>,
        name: Option<&str>,
    ) {
        let (x1, y1) = from.unwrap_or((self.lx, self.ly));
        let (dx, dy) = dir.vec();
        let end = to.unwrap_or_else(|| (x1 + dx * len, y1 + dy * len));
        let mut pts = vec![(x1, y1), end];
        for (sdir, slen) in &then_segs {
            let &(px, py) = pts.last().unwrap();
            let (sdx, sdy) = sdir.vec();
            pts.push((px + sdx * slen, py + sdy * slen));
        }
        let &(ex, ey) = pts.last().unwrap();
        self.lx = ex;
        self.ly = ey;
        if let Some(n) = name {
            let cx = (x1 + ex) / 2.0;
            let cy = (y1 + ey) / 2.0;
            self.names.insert(
                n.to_string(),
                PkObj {
                    cx,
                    cy,
                    hw: (ex - x1).abs() / 2.0,
                    hh: (ey - y1).abs() / 2.0,
                },
            );
        }
        self.items.push(PkItem::Line(PkLineItem {
            points: pts,
            arrow_end,
            arrow_start,
            label,
            style,
        }));
    }
}

// ─── Tokeniser ─────────────────────────────────────────────────────────

fn pk_tokens(s: &str) -> Vec<&str> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if b[i] == b'"' {
            let start = i;
            i += 1;
            while i < b.len() && b[i] != b'"' {
                i += 1;
            }
            if i < b.len() {
                i += 1;
            }
            out.push(&s[start..i]);
        } else {
            let start = i;
            while i < b.len() && !b[i].is_ascii_whitespace() && b[i] != b'"' {
                i += 1;
            }
            out.push(&s[start..i]);
        }
    }
    out
}

fn pk_unquote(s: &str) -> &str {
    s.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(s)
}

/// Parse a number that Pikchr treats as inches; returns pixels.
fn pk_parse_num(s: &str) -> Option<f32> {
    let s = s.trim_end_matches(|c: char| c.is_alphabetic());
    s.parse::<f32>().ok().map(|n| n * PK_INCH)
}

// ─── Attribute block parser ────────────────────────────────────────────

struct PkAttrs {
    dir: Option<PkDir>,
    len: Option<f32>,
    width: Option<f32>,
    height: Option<f32>,
    style: PkStyle,
    labels: Vec<(String, PkPlace)>,
    at: Option<String>,
    from: Option<String>,
    to: Option<String>,
    then_segs: Vec<(PkDir, f32)>,
    arrow_start: bool, // bidirectional (<->)
}

impl Default for PkAttrs {
    fn default() -> Self {
        Self {
            dir: None,
            len: None,
            width: None,
            height: None,
            style: PkStyle::default(),
            labels: Vec::new(),
            at: None,
            from: None,
            to: None,
            then_segs: Vec::new(),
            arrow_start: false,
        }
    }
}

fn parse_pk_attrs(tokens: &[&str]) -> PkAttrs {
    let mut a = PkAttrs::default();
    let mut i = 0;
    let mut after_then = false;

    while i < tokens.len() {
        let t = tokens[i];
        let tl = t.to_ascii_lowercase();

        if t.starts_with('"') {
            // Quoted label; check if the next token is a placement modifier.
            let content = pk_unquote(t).to_string();
            let place = if i + 1 < tokens.len() {
                if let Some(p) = parse_pk_place(tokens[i + 1]) {
                    i += 1;
                    p
                } else {
                    PkPlace::Center
                }
            } else {
                PkPlace::Center
            };
            a.labels.push((content, place));
            i += 1;
            continue;
        }

        if tl == "then" {
            after_then = true;
            i += 1;
            continue;
        }

        if let Some(dir) = parse_pk_dir(&tl) {
            // Peek for optional length after the direction.
            let seg_len = if i + 1 < tokens.len() {
                pk_parse_num(tokens[i + 1]).map(|n| {
                    i += 1;
                    n
                })
            } else {
                None
            };
            if after_then {
                a.then_segs.push((dir, seg_len.unwrap_or(PK_ARROW_LEN)));
                after_then = false;
            } else {
                a.dir = Some(dir);
                if let Some(n) = seg_len {
                    a.len = Some(n);
                }
            }
            i += 1;
            continue;
        }
        after_then = false;

        match tl.as_str() {
            "dashed" => a.style.dashed = true,
            "dotted" => a.style.dotted = true,
            "thick" => a.style.thickness = Some(4.0),
            "thin" => a.style.thickness = Some(1.0),
            "fill" => {
                i += 1;
                if i < tokens.len() {
                    a.style.fill = parse_pk_color(tokens[i]);
                }
            }
            "color" | "stroke" => {
                i += 1;
                if i < tokens.len() {
                    a.style.stroke = parse_pk_color(tokens[i]);
                }
            }
            "width" | "wd" => {
                i += 1;
                if i < tokens.len() {
                    a.width = pk_parse_num(tokens[i]);
                }
            }
            "height" | "ht" => {
                i += 1;
                if i < tokens.len() {
                    a.height = pk_parse_num(tokens[i]);
                }
            }
            "radius" | "rad" => {
                i += 1;
                if let Some(r) = tokens.get(i).and_then(|t| pk_parse_num(t)) {
                    a.width = Some(r * 2.0);
                    a.height = Some(r * 2.0);
                }
            }
            "len" | "length" => {
                i += 1;
                if i < tokens.len() {
                    a.len = pk_parse_num(tokens[i]);
                }
            }
            "at" => {
                i += 1;
                if i < tokens.len() {
                    a.at = Some(tokens[i].to_string());
                }
            }
            "from" => {
                i += 1;
                if i < tokens.len() {
                    a.from = Some(tokens[i].to_string());
                }
            }
            "to" => {
                i += 1;
                if i < tokens.len() {
                    a.to = Some(tokens[i].to_string());
                }
            }
            "<->" => a.arrow_start = true,
            _ => {
                // Standalone placement modifier (applies to last label pushed so far).
                if let Some(p) = parse_pk_place(&tl) {
                    if let Some((_, pl)) = a.labels.last_mut() {
                        *pl = p;
                    }
                } else if let Some(n) = pk_parse_num(t) {
                    if a.len.is_none() {
                        a.len = Some(n);
                    }
                }
            }
        }
        i += 1;
    }
    a
}

// ─── Statement parser ──────────────────────────────────────────────────

fn pk_strip_comment(line: &str) -> &str {
    // Strip # comments (not inside quotes) and // comments.
    let mut in_q = false;
    let b = line.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'"' {
            in_q = !in_q;
        }
        if !in_q {
            if b[i] == b'#' {
                return &line[..i];
            }
            if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'/' {
                return &line[..i];
            }
        }
        i += 1;
    }
    line
}

fn parse_pk_statement(state: &mut PkState, stmt: &str) {
    let stmt = stmt.trim();
    if stmt.is_empty() {
        return;
    }
    // Skip macro / control structures (require expression evaluator).
    let lower = stmt.to_ascii_lowercase();
    if lower.starts_with("if ")
        || lower.starts_with("for ")
        || lower.starts_with("define ")
        || lower.starts_with("print ")
        || lower.starts_with("assert ")
    {
        return;
    }

    let toks = pk_tokens(stmt);
    if toks.is_empty() {
        return;
    }

    let mut idx = 0;
    // Optional "Name:" label.
    let name: Option<&str> = if !toks[0].starts_with('"') && toks[0].ends_with(':') {
        let n = toks[0].trim_end_matches(':');
        idx += 1;
        if n.is_empty() {
            None
        } else {
            Some(n)
        }
    } else {
        None
    };

    if idx >= toks.len() {
        return;
    }
    let cmd = toks[idx].to_ascii_lowercase();
    idx += 1;
    let rest = &toks[idx..];

    match cmd.as_str() {
        "box" | "circle" | "ellipse" | "oval" | "cylinder" | "diamond" | "file" => {
            let shape = match cmd.as_str() {
                "box" => PkShape::Box,
                "circle" => PkShape::Circle,
                "ellipse" => PkShape::Ellipse,
                "oval" => PkShape::Oval,
                "cylinder" => PkShape::Cylinder,
                "diamond" => PkShape::Diamond,
                _ => PkShape::File,
            };
            let a = parse_pk_attrs(rest);
            let (def_hw, def_hh) = shape.half_size();
            let hw = a.width.map(|w| w / 2.0).unwrap_or(def_hw);
            let hh = a.height.map(|h| h / 2.0).unwrap_or(def_hh);
            if let Some(d) = a.dir {
                state.dir = d;
            }
            let at = a.at.as_deref().and_then(|r| state.resolve(r));
            state.place_shape(shape, hw, hh, a.labels, a.style, name, at);
        }
        "arrow" => {
            let a = parse_pk_attrs(rest);
            let dir = a.dir.unwrap_or(state.dir);
            state.dir = dir;
            let len = a.len.unwrap_or(PK_ARROW_LEN);
            let from = a.from.as_deref().and_then(|r| state.resolve(r));
            let to = a.to.as_deref().and_then(|r| state.resolve(r));
            let lbl = a.labels.into_iter().next();
            state.place_line(
                dir,
                len,
                true,
                a.arrow_start,
                lbl,
                a.style,
                from,
                to,
                a.then_segs,
                name,
            );
        }
        "line" | "spline" => {
            let a = parse_pk_attrs(rest);
            let dir = a.dir.unwrap_or(state.dir);
            state.dir = dir;
            let len = a.len.unwrap_or(PK_ARROW_LEN);
            let from = a.from.as_deref().and_then(|r| state.resolve(r));
            let to = a.to.as_deref().and_then(|r| state.resolve(r));
            let lbl = a.labels.into_iter().next();
            // "line" has no arrowhead by default; bidirectional (<->) adds start.
            state.place_line(
                dir,
                len,
                a.arrow_start,
                false,
                lbl,
                a.style,
                from,
                to,
                a.then_segs,
                name,
            );
        }
        "move" => {
            let a = parse_pk_attrs(rest);
            let dir = a.dir.unwrap_or(state.dir);
            state.dir = dir;
            let len = a.len.unwrap_or(PK_MOVE_LEN);
            let (dx, dy) = dir.vec();
            state.lx += dx * len;
            state.ly += dy * len;
        }
        "text" => {
            let a = parse_pk_attrs(rest);
            for (content, placement) in a.labels {
                state.items.push(PkItem::Text(PkTextItem {
                    x: state.lx,
                    y: state.ly,
                    content,
                    placement,
                }));
            }
        }
        _ => {}
    }
}

fn parse_pk_diagram(body: &str) -> PkState {
    let mut state = PkState::new();
    for line in body.lines() {
        let line = pk_strip_comment(line);
        for stmt in line.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            parse_pk_statement(&mut state, stmt);
        }
    }
    state
}

// ─── Bounding-box helper ───────────────────────────────────────────────

fn pk_bbox(items: &[PkItem]) -> (f32, f32, f32, f32) {
    let mut x0 = f32::INFINITY;
    let mut y0 = f32::INFINITY;
    let mut x1 = f32::NEG_INFINITY;
    let mut y1 = f32::NEG_INFINITY;
    for item in items {
        match item {
            PkItem::Shape(s) => {
                x0 = x0.min(s.cx - s.hw);
                y0 = y0.min(s.cy - s.hh);
                x1 = x1.max(s.cx + s.hw);
                y1 = y1.max(s.cy + s.hh);
                // Account for labels placed outside the shape.
                for (_, p) in &s.labels {
                    if *p == PkPlace::Above {
                        y0 = y0.min(s.cy - s.hh - 20.0);
                    }
                    if *p == PkPlace::Below {
                        y1 = y1.max(s.cy + s.hh + 20.0);
                    }
                }
            }
            PkItem::Line(l) => {
                for &(px, py) in &l.points {
                    x0 = x0.min(px);
                    y0 = y0.min(py);
                    x1 = x1.max(px);
                    y1 = y1.max(py);
                }
            }
            PkItem::Text(t) => {
                x0 = x0.min(t.x - 50.0);
                y0 = y0.min(t.y - 18.0);
                x1 = x1.max(t.x + 50.0);
                y1 = y1.max(t.y + 18.0);
            }
        }
    }
    if x0 == f32::INFINITY {
        (0.0, 0.0, 200.0, 100.0)
    } else {
        (x0, y0, x1, y1)
    }
}

// ─── SVG emitter ───────────────────────────────────────────────────────

fn pk_emit_shape(svg: &mut String, s: &PkShapeItem, ox: f32, oy: f32) {
    let cx = s.cx + ox;
    let cy = s.cy + oy;
    let hw = s.hw;
    let hh = s.hh;
    let x = cx - hw;
    let y = cy - hh;
    let w = hw * 2.0;
    let h = hh * 2.0;
    let fill = s.style.fill.to_svg("#eff6ff");
    let stroke = s.style.stroke.to_svg("#275DA8");
    let sw = s.style.sw(2.0);
    let dash = s.style.dash_attr();
    let cls = s.shape.css_class();

    match s.shape {
        PkShape::Box => {
            svg.push_str(&format!(
                "<rect class=\"pikchr-node {cls}\" x=\"{x:.1}\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" rx=\"6\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw}\"{dash}/>"
            ));
        }
        PkShape::Circle => {
            let r = hw.min(hh);
            svg.push_str(&format!(
                "<circle class=\"pikchr-node {cls}\" cx=\"{cx:.1}\" cy=\"{cy:.1}\" r=\"{r:.1}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw}\"{dash}/>"
            ));
        }
        PkShape::Ellipse => {
            svg.push_str(&format!(
                "<ellipse class=\"pikchr-node {cls}\" cx=\"{cx:.1}\" cy=\"{cy:.1}\" rx=\"{hw:.1}\" ry=\"{hh:.1}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw}\"{dash}/>"
            ));
        }
        PkShape::Oval => {
            // Oval = rectangle with fully-rounded short ends.
            svg.push_str(&format!(
                "<rect class=\"pikchr-node {cls}\" x=\"{x:.1}\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" rx=\"{hh:.1}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw}\"{dash}/>"
            ));
        }
        PkShape::Cylinder => {
            let cap = hh * 0.22; // cap ellipse half-height
            let body_y = y + cap;
            let body_h = h - cap;
            svg.push_str(&format!(
                "<rect class=\"pikchr-node {cls}\" x=\"{x:.1}\" y=\"{body_y:.1}\" width=\"{w:.1}\" height=\"{body_h:.1}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw}\"{dash}/>"
            ));
            svg.push_str(&format!(
                "<ellipse cx=\"{cx:.1}\" cy=\"{body_y:.1}\" rx=\"{hw:.1}\" ry=\"{cap:.1}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw}\"{dash}/>"
            ));
            // Bottom arc: cubic Bézier curving downward.
            svg.push_str(&format!(
                "<path d=\"M{x:.1},{:.1} C{x:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{sw}\"{dash}/>",
                y + h - cap,
                y + h + cap,
                x + w, y + h + cap,
                x + w, y + h - cap
            ));
        }
        PkShape::Diamond => {
            svg.push_str(&format!(
                "<polygon class=\"pikchr-node {cls}\" points=\"{cx:.1},{y:.1} {:.1},{cy:.1} {cx:.1},{:.1} {x:.1},{cy:.1}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw}\"{dash}/>",
                x + w, y + h
            ));
        }
        PkShape::File => {
            let notch = hw * 0.28;
            svg.push_str(&format!(
                "<path class=\"pikchr-node {cls}\" d=\"M{x:.1},{y:.1} H{:.1} L{:.1},{:.1} V{:.1} H{x:.1} Z\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw}\"{dash}/>",
                x + w - notch, x + w, y + notch, y + h
            ));
            svg.push_str(&format!(
                "<path d=\"M{:.1},{y:.1} V{:.1} H{:.1}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{sw}\"{dash}/>",
                x + w - notch, y + notch, x + w
            ));
        }
    }

    for (label, place) in &s.labels {
        if label.is_empty() {
            continue;
        }
        let (tx, ty, anchor) = match place {
            PkPlace::Center => (cx, cy + 5.0, "middle"),
            PkPlace::Above => (cx, y - 5.0, "middle"),
            PkPlace::Below => (cx, y + h + 15.0, "middle"),
            PkPlace::Ljust => (x + 5.0, cy + 5.0, "start"),
            PkPlace::Rjust => (x + w - 5.0, cy + 5.0, "end"),
        };
        svg.push_str(&format!(
            "<text x=\"{tx:.1}\" y=\"{ty:.1}\" font-size=\"14\" text-anchor=\"{anchor}\" fill=\"#111827\">{}</text>",
            escape_html(label)
        ));
    }
}

fn pk_emit_line(svg: &mut String, l: &PkLineItem, ox: f32, oy: f32) {
    if l.points.len() < 2 {
        return;
    }
    let stroke = l.style.stroke.to_svg("#275DA8");
    let sw = l.style.sw(2.0);
    let dash = l.style.dash_attr();
    let end_marker = if l.arrow_end {
        " marker-end=\"url(#pikchr-arrow)\""
    } else {
        ""
    };
    let start_marker = if l.arrow_start {
        " marker-start=\"url(#pikchr-arrow-start)\""
    } else {
        ""
    };

    if l.points.len() == 2 {
        let (x1, y1) = l.points[0];
        let (x2, y2) = l.points[1];
        svg.push_str(&format!(
            "<line class=\"pikchr-arrow\" x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{stroke}\" stroke-width=\"{sw}\"{dash}{end_marker}{start_marker}/>",
            x1 + ox, y1 + oy, x2 + ox, y2 + oy
        ));
    } else {
        let pts: String = l
            .points
            .iter()
            .map(|(px, py)| format!("{:.1},{:.1}", px + ox, py + oy))
            .collect::<Vec<_>>()
            .join(" ");
        svg.push_str(&format!(
            "<polyline class=\"pikchr-arrow\" points=\"{pts}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"{sw}\"{dash}{end_marker}{start_marker}/>"
        ));
    }

    if let Some((label, place)) = &l.label {
        if !label.is_empty() {
            let n = l.points.len();
            let mid = if n % 2 == 1 {
                l.points[n / 2]
            } else {
                let (ax, ay) = l.points[n / 2 - 1];
                let (bx, by) = l.points[n / 2];
                ((ax + bx) / 2.0, (ay + by) / 2.0)
            };
            let (tx, ty, anchor) = match place {
                PkPlace::Below => (mid.0 + ox, mid.1 + oy + 15.0, "middle"),
                PkPlace::Ljust => (mid.0 + ox - 4.0, mid.1 + oy + 5.0, "end"),
                PkPlace::Rjust => (mid.0 + ox + 4.0, mid.1 + oy + 5.0, "start"),
                _ => (mid.0 + ox, mid.1 + oy - 6.0, "middle"),
            };
            svg.push_str(&format!(
                "<text x=\"{tx:.1}\" y=\"{ty:.1}\" font-size=\"12\" text-anchor=\"{anchor}\" fill=\"#475569\">{}</text>",
                escape_html(label)
            ));
        }
    }
}

fn pk_emit_text(svg: &mut String, t: &PkTextItem, ox: f32, oy: f32) {
    let (tx, ty, anchor) = match t.placement {
        PkPlace::Above => (t.x + ox, t.y + oy - 6.0, "middle"),
        PkPlace::Below => (t.x + ox, t.y + oy + 15.0, "middle"),
        PkPlace::Ljust => (t.x + ox, t.y + oy, "start"),
        PkPlace::Rjust => (t.x + ox, t.y + oy, "end"),
        PkPlace::Center => (t.x + ox, t.y + oy, "middle"),
    };
    svg.push_str(&format!(
        "<text x=\"{tx:.1}\" y=\"{ty:.1}\" font-size=\"14\" text-anchor=\"{anchor}\" fill=\"#111827\">{}</text>",
        escape_html(&t.content)
    ));
}

fn emit_pk_svg(state: &PkState) -> String {
    let (bx0, by0, bx1, by1) = pk_bbox(&state.items);
    let ox = PK_PAD - bx0;
    let oy = PK_PAD - by0;
    let vw = ((bx1 - bx0) + 2.0 * PK_PAD).ceil() as i32;
    let vh = ((by1 - by0) + 2.0 * PK_PAD).ceil() as i32;

    let mut svg = format!(
        "<svg class=\"transform transform-pikchr\" xmlns=\"http://www.w3.org/2000/svg\" \
         viewBox=\"0 0 {vw} {vh}\" role=\"img\">\
         <defs>\
         <marker id=\"pikchr-arrow\" markerWidth=\"10\" markerHeight=\"7\" refX=\"9\" refY=\"3.5\" \
           orient=\"auto\" markerUnits=\"strokeWidth\">\
           <path d=\"M0,0 L0,7 L10,3.5 z\" fill=\"#275DA8\"/>\
         </marker>\
         <marker id=\"pikchr-arrow-start\" markerWidth=\"10\" markerHeight=\"7\" refX=\"1\" refY=\"3.5\" \
           orient=\"auto-start-reverse\" markerUnits=\"strokeWidth\">\
           <path d=\"M0,0 L0,7 L10,3.5 z\" fill=\"#275DA8\"/>\
         </marker>\
         </defs>"
    );

    // Lines drawn before shapes so shapes occlude line endpoints cleanly.
    for item in &state.items {
        if let PkItem::Line(l) = item {
            pk_emit_line(&mut svg, l, ox, oy);
        }
    }
    for item in &state.items {
        if let PkItem::Shape(s) = item {
            pk_emit_shape(&mut svg, s, ox, oy);
        }
    }
    for item in &state.items {
        if let PkItem::Text(t) = item {
            pk_emit_text(&mut svg, t, ox, oy);
        }
    }

    svg.push_str("</svg>");
    svg
}

pub(crate) fn render_pikchr_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let state = parse_pk_diagram(body);
    if state.items.is_empty() {
        let diagnostic = diag(
            "warning",
            "Pikchr native preview did not find any supported shape nodes.",
            None,
            None,
            Some("Use statements such as box \"Start\"; arrow right; diamond \"Decision\". Configure an external Pikchr engine for full grammar support."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-pikchr transform-error\">No Pikchr nodes found</section>".to_string();
    }
    emit_pk_svg(&state)
}

pub(crate) fn render_dot_svg(
    name: &str,
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let graph = parse_dot_graph(body);
    if graph.nodes.is_empty() || graph.edges.is_empty() {
        return unsupported_native_diagram(
            name,
            "DOT native preview only supports simple edge statements.",
            "Use edges such as a -> b, or configure Graphviz as an external transform engine.",
            artifact_diags,
            diagnostics,
        );
    }
    render_simple_graph_svg(name, &graph)
}

pub(crate) fn render_d2_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let graph = parse_d2_graph(body);
    if graph.nodes.is_empty() || graph.edges.is_empty() {
        return unsupported_native_diagram(
            "d2",
            "D2 native preview only supports simple edge statements.",
            "Use edges such as source -> target: label, or configure D2 as an external transform engine.",
            artifact_diags,
            diagnostics,
        );
    }
    render_simple_graph_svg("d2", &graph)
}

pub(crate) fn render_plantuml_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let graph = parse_plantuml_graph(body);
    if graph.nodes.is_empty() || graph.edges.is_empty() {
        return unsupported_native_diagram(
            "plantuml",
            "PlantUML native preview only supports simple sequence or component arrows.",
            "Use arrows such as Alice -> Bob: message, or configure PlantUML as an external transform engine.",
            artifact_diags,
            diagnostics,
        );
    }
    render_simple_graph_svg("plantuml", &graph)
}

fn unsupported_native_diagram(
    name: &str,
    message: &str,
    suggestion: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let diagnostic = diag("warning", message.to_string(), None, None, Some(suggestion));
    artifact_diags.push(diagnostic.clone());
    diagnostics.push(diagnostic);
    format!(
        "<section class=\"transform transform-{} transform-error\">Unsupported {} diagram</section>",
        escape_html(name),
        escape_html(name)
    )
}

fn render_simple_graph_svg(name: &str, graph: &MermaidGraph) -> String {
    let columns = 3usize;
    let node_width = 170usize;
    let node_height = 54usize;
    let x_gap = 250usize;
    let y_gap = 120usize;
    let rows = graph.nodes.len().div_ceil(columns);
    let width = 120 + columns * x_gap;
    let height = 90 + rows * y_gap;
    let marker_id = format!("{name}-arrow");
    let positions = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let x = 60 + (index % columns) * x_gap;
            let y = 55 + (index / columns) * y_gap;
            (node.id.clone(), (x, y))
        })
        .collect::<HashMap<_, _>>();
    let mut svg = format!(
        "<svg class=\"transform transform-{}\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" role=\"img\"><defs><marker id=\"{}\" markerWidth=\"10\" markerHeight=\"10\" refX=\"8\" refY=\"3\" orient=\"auto\" markerUnits=\"strokeWidth\"><path d=\"M0,0 L0,6 L9,3 z\" fill=\"#275DA8\"/></marker></defs>",
        escape_html(name),
        escape_html(&marker_id)
    );
    for edge in &graph.edges {
        if let (Some((from_x, from_y)), Some((to_x, to_y))) =
            (positions.get(&edge.from), positions.get(&edge.to))
        {
            let x1 = from_x + node_width;
            let y1 = from_y + node_height / 2;
            let x2 = *to_x;
            let y2 = to_y + node_height / 2;
            svg.push_str(&format!(
                "<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"#275DA8\" stroke-width=\"3\" marker-end=\"url(#{})\"/>",
                escape_html(&marker_id)
            ));
            render_edge_label(&mut svg, x1, y1, x2, y2, edge.label.as_deref());
        }
    }
    for node in &graph.nodes {
        if let Some((x, y)) = positions.get(&node.id) {
            svg.push_str(&format!(
                "<rect x=\"{x}\" y=\"{y}\" width=\"{node_width}\" height=\"{node_height}\" rx=\"8\" fill=\"#eff6ff\" stroke=\"#275DA8\" stroke-width=\"2\"/><text x=\"{}\" y=\"{}\" font-size=\"15\" text-anchor=\"middle\" fill=\"#1f2937\">{}</text>",
                x + node_width / 2,
                y + 33,
                escape_html(&node.label)
            ));
        }
    }
    svg.push_str("</svg>");
    svg
}

#[derive(Debug)]
struct MermaidGraph {
    nodes: Vec<MermaidNode>,
    edges: Vec<MermaidEdge>,
}

#[derive(Debug)]
struct MermaidNode {
    id: String,
    label: String,
}

#[derive(Debug)]
struct MermaidEdge {
    from: String,
    to: String,
    label: Option<String>,
}

fn parse_mermaid_flowchart(body: &str) -> MermaidGraph {
    let mut nodes = Vec::new();
    let mut seen = HashSet::new();
    let mut edges = Vec::new();
    for line in body.lines() {
        let line = line.trim().trim_end_matches(';').trim();
        if line.is_empty()
            || line.starts_with("%%")
            || line.starts_with("graph ")
            || line.starts_with("flowchart ")
        {
            continue;
        }
        let Some((left, right)) = split_mermaid_edge(line) else {
            continue;
        };
        let from = parse_mermaid_node(left);
        let (right, label) = split_mermaid_edge_label(right);
        let to = parse_mermaid_node(right);
        add_mermaid_node(&mut nodes, &mut seen, &from);
        add_mermaid_node(&mut nodes, &mut seen, &to);
        edges.push(MermaidEdge {
            from: from.id,
            to: to.id,
            label,
        });
    }
    MermaidGraph { nodes, edges }
}

fn split_mermaid_edge(line: &str) -> Option<(&str, &str)> {
    for operator in ["-->", "==>", "-.->", "---"] {
        if let Some((left, right)) = line.split_once(operator) {
            return Some((left.trim(), right.trim()));
        }
    }
    None
}

fn split_mermaid_edge_label(text: &str) -> (&str, Option<String>) {
    let text = text.trim();
    if let Some(rest) = text.strip_prefix('|') {
        if let Some((_, after_label)) = rest.split_once('|') {
            return (
                after_label.trim(),
                rest.split_once('|')
                    .map(|(label, _)| label.trim().to_string())
                    .filter(|label| !label.is_empty()),
            );
        }
    }
    (text, None)
}

fn parse_mermaid_node(text: &str) -> MermaidNode {
    let text = text.trim();
    for (open, close) in [('[', ']'), ('(', ')'), ('{', '}')] {
        if let Some(start) = text.find(open) {
            if let Some(end) = text.rfind(close) {
                let id = text[..start].trim();
                let label = text[start + 1..end].trim().trim_matches('"');
                return MermaidNode {
                    id: id.to_string(),
                    label: label.to_string(),
                };
            }
        }
    }
    let id = text
        .split_whitespace()
        .next()
        .unwrap_or(text)
        .trim_matches('"')
        .to_string();
    MermaidNode {
        label: id.clone(),
        id,
    }
}

fn add_mermaid_node(nodes: &mut Vec<MermaidNode>, seen: &mut HashSet<String>, node: &MermaidNode) {
    if seen.insert(node.id.clone()) {
        nodes.push(MermaidNode {
            id: node.id.clone(),
            label: node.label.clone(),
        });
    } else if let Some(existing) = nodes.iter_mut().find(|existing| existing.id == node.id) {
        if (existing.label == existing.id || existing.label == d2_node_leaf(&existing.id))
            && node.label != node.id
        {
            existing.label.clone_from(&node.label);
        }
    }
}

fn parse_dot_graph(body: &str) -> MermaidGraph {
    let mut nodes = Vec::new();
    let mut seen = HashSet::new();
    let mut edges = Vec::new();
    for statement in body.replace(['{', '}', ';'], "\n").lines().map(str::trim) {
        if statement.is_empty()
            || statement.starts_with("//")
            || statement.starts_with('#')
            || statement.starts_with("digraph")
            || statement.starts_with("graph")
            || statement.starts_with("node ")
            || statement.starts_with("edge ")
        {
            continue;
        }
        if let Some((left, right)) = split_first_operator(statement, &["->", "--"]) {
            let from = parse_plain_graph_node(left);
            let to = parse_plain_graph_node(strip_bracket_attributes(right));
            let label = extract_quoted_attribute(statement, "label");
            add_mermaid_node(&mut nodes, &mut seen, &from);
            add_mermaid_node(&mut nodes, &mut seen, &to);
            edges.push(MermaidEdge {
                from: from.id,
                to: to.id,
                label,
            });
        } else if statement.contains("[label=") {
            let node = parse_plain_graph_node(statement);
            add_mermaid_node(&mut nodes, &mut seen, &node);
        }
    }
    MermaidGraph { nodes, edges }
}

fn parse_d2_graph(body: &str) -> MermaidGraph {
    let mut nodes = Vec::new();
    let mut seen = HashSet::new();
    let mut edges = Vec::new();
    let mut scope: Vec<String> = Vec::new();
    for raw_line in body.lines().flat_map(|line| line.split(';')) {
        let mut line = raw_line.trim();
        while let Some(rest) = line.strip_prefix('}') {
            scope.pop();
            line = rest.trim();
        }
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        let opens_scope = line.ends_with('{');
        line = line.trim_end_matches('{').trim();
        if line.is_empty() {
            continue;
        }
        if let Some((left, right)) = split_first_operator(line, &["<->", "->", "--"]) {
            let from = parse_d2_graph_node(left, &scope);
            let (target, label) = split_d2_edge_label(right);
            let to = parse_d2_graph_node(target, &scope);
            add_mermaid_node(&mut nodes, &mut seen, &from);
            add_mermaid_node(&mut nodes, &mut seen, &to);
            edges.push(MermaidEdge {
                from: from.id,
                to: to.id,
                label,
            });
        } else if let Some((id, label)) = line.split_once(':') {
            if is_d2_attribute_statement(id) {
                continue;
            }
            let full_id = qualify_d2_node_id(&normalize_plain_node_id(id), &scope);
            let node = MermaidNode {
                id: full_id,
                label: clean_d2_label(label),
            };
            add_mermaid_node(&mut nodes, &mut seen, &node);
        } else if !is_d2_attribute_statement(line) {
            let node = parse_d2_graph_node(line, &scope);
            if !node.id.is_empty() {
                add_mermaid_node(&mut nodes, &mut seen, &node);
            }
        }
        if opens_scope {
            if let Some(scope_id) = d2_scope_id(line, &scope) {
                scope.push(scope_id);
            }
        }
    }
    MermaidGraph { nodes, edges }
}

fn parse_d2_graph_node(text: &str, scope: &[String]) -> MermaidNode {
    let label = extract_quoted_attribute(text, "label");
    let raw_id = normalize_plain_node_id(strip_bracket_attributes(text));
    let id = qualify_d2_node_id(&raw_id, scope);
    MermaidNode {
        label: label.unwrap_or_else(|| d2_node_leaf(&id).to_string()),
        id,
    }
}

fn d2_scope_id(statement: &str, scope: &[String]) -> Option<String> {
    if split_first_operator(statement, &["<->", "->", "--"]).is_some() {
        return None;
    }
    let id = statement
        .split_once(':')
        .map(|(id, _)| id)
        .unwrap_or(statement)
        .trim();
    if id.is_empty() || is_d2_attribute_statement(id) {
        return None;
    }
    Some(qualify_d2_node_id(&normalize_plain_node_id(id), scope))
}

fn qualify_d2_node_id(id: &str, scope: &[String]) -> String {
    let id = id.trim();
    if id.is_empty() || id.contains('.') || scope.is_empty() {
        return id.to_string();
    }
    format!("{}.{}", scope.last().expect("checked non-empty scope"), id)
}

fn d2_node_leaf(id: &str) -> &str {
    id.rsplit('.').next().unwrap_or(id)
}

fn split_d2_edge_label(text: &str) -> (&str, Option<String>) {
    text.split_once(':')
        .map(|(id, label)| (id.trim(), Some(clean_d2_label(label))))
        .unwrap_or((text.trim(), None))
}

fn clean_d2_label(text: &str) -> String {
    text.trim()
        .trim_end_matches('{')
        .trim()
        .trim_matches('"')
        .to_string()
}

fn is_d2_attribute_statement(id: &str) -> bool {
    let key = id.trim().to_ascii_lowercase();
    matches!(
        key.as_str(),
        "direction"
            | "shape"
            | "label"
            | "tooltip"
            | "link"
            | "icon"
            | "width"
            | "height"
            | "near"
            | "class"
            | "classes"
    ) || key.starts_with("style.")
        || key.ends_with(".shape")
        || key.ends_with(".style")
        || key.contains(".style.")
}

fn parse_plantuml_graph(body: &str) -> MermaidGraph {
    let mut nodes = Vec::new();
    let mut seen = HashSet::new();
    let mut edges = Vec::new();
    for raw_line in body.lines().flat_map(|line| line.split(';')) {
        let line = raw_line.trim().trim_end_matches('{').trim();
        if line.is_empty()
            || line.starts_with('\'')
            || line.starts_with("@start")
            || line.starts_with("@end")
            || line.starts_with("skinparam")
            || line.starts_with("left to right direction")
        {
            continue;
        }
        if let Some((keyword, rest)) = line.split_once(' ') {
            if is_plantuml_node_keyword(keyword) {
                let node = parse_plantuml_declaration_node(rest);
                add_mermaid_node(&mut nodes, &mut seen, &node);
                continue;
            }
        }
        if let Some((left, right)) =
            split_first_operator(line, &["-->", "->", "<--", "<-", "..>", ".>"])
        {
            let from = parse_plain_graph_node(left);
            let (target, label) = split_d2_edge_label(right);
            let to = parse_plain_graph_node(target);
            add_mermaid_node(&mut nodes, &mut seen, &from);
            add_mermaid_node(&mut nodes, &mut seen, &to);
            edges.push(MermaidEdge {
                from: from.id,
                to: to.id,
                label,
            });
        }
    }
    MermaidGraph { nodes, edges }
}

fn is_plantuml_node_keyword(keyword: &str) -> bool {
    matches!(
        keyword.to_ascii_lowercase().as_str(),
        "actor"
            | "participant"
            | "component"
            | "database"
            | "queue"
            | "boundary"
            | "control"
            | "entity"
            | "interface"
            | "collections"
            | "storage"
            | "folder"
            | "artifact"
            | "node"
            | "cloud"
            | "rectangle"
            | "package"
            | "frame"
    )
}

fn parse_plantuml_declaration_node(text: &str) -> MermaidNode {
    let text = text
        .trim()
        .trim_matches('{')
        .trim()
        .trim_matches(['[', ']'])
        .trim();
    if let Some((label, alias)) = split_plantuml_alias(text) {
        let clean_alias = normalize_plain_node_id(alias);
        let clean_label = clean_plantuml_label(label);
        if !clean_alias.is_empty() {
            return MermaidNode {
                id: clean_alias,
                label: clean_label,
            };
        }
    }
    if let Some(label) = first_quoted(text) {
        let id = normalize_plain_node_id(&label);
        return MermaidNode {
            id,
            label: clean_plantuml_label(&label),
        };
    }
    parse_plain_graph_node(text)
}

fn first_quoted(text: &str) -> Option<String> {
    let start = text.find('"')? + 1;
    let end = text[start..].find('"')?;
    Some(text[start..start + end].to_string())
}

fn split_plantuml_alias(text: &str) -> Option<(&str, &str)> {
    for marker in [" as ", " AS ", " As ", " aS "] {
        if let Some((label, alias)) = text.rsplit_once(marker) {
            return Some((label.trim(), alias.trim()));
        }
    }
    None
}

fn clean_plantuml_label(text: &str) -> String {
    text.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim_matches(['[', ']'])
        .trim()
        .to_string()
}

fn render_edge_label(
    svg: &mut String,
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    label: Option<&str>,
) {
    let Some(label) = label.filter(|label| !label.trim().is_empty()) else {
        return;
    };
    let x = (x1 + x2) / 2;
    let y = (y1 + y2) / 2;
    svg.push_str(&format!(
        "<text class=\"diagram-edge-label\" x=\"{x}\" y=\"{}\" font-size=\"12\" text-anchor=\"middle\" fill=\"#475569\">{}</text>",
        y.saturating_sub(8),
        escape_html(label)
    ));
}

fn split_first_operator<'a>(line: &'a str, operators: &[&str]) -> Option<(&'a str, &'a str)> {
    operators
        .iter()
        .filter_map(|operator| line.find(operator).map(|index| (index, *operator)))
        .min_by_key(|(index, _)| *index)
        .map(|(index, operator)| {
            let after_operator = index + operator.len();
            (line[..index].trim(), line[after_operator..].trim())
        })
}

fn parse_plain_graph_node(text: &str) -> MermaidNode {
    let label = extract_quoted_attribute(text, "label");
    let id = normalize_plain_node_id(strip_bracket_attributes(text));
    MermaidNode {
        label: label.unwrap_or_else(|| id.clone()),
        id,
    }
}

fn strip_bracket_attributes(text: &str) -> &str {
    text.split('[').next().unwrap_or(text).trim()
}

fn normalize_plain_node_id(text: &str) -> String {
    text.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}
