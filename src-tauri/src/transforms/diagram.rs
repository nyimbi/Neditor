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

pub(crate) fn render_pikchr_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let nodes = parse_pikchr_nodes(body);
    if nodes.is_empty() {
        let diagnostic = diag(
            "warning",
            "Pikchr native preview did not find any box or circle nodes.",
            None,
            None,
            Some("Use simple lines such as box \"Start\"; arrow; box \"Done\", or configure an external Pikchr engine."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-pikchr transform-error\">No Pikchr nodes found</section>".to_string();
    }
    let has_arrows = body
        .lines()
        .any(|line| line.trim_start().starts_with("arrow"))
        || nodes.len() > 1;
    let width = nodes.len().max(1) * 190 + 60;
    let mut svg = format!(
        "<svg class=\"transform transform-pikchr\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} 180\" role=\"img\"><defs><marker id=\"pikchr-arrow\" markerWidth=\"10\" markerHeight=\"10\" refX=\"8\" refY=\"3\" orient=\"auto\" markerUnits=\"strokeWidth\"><path d=\"M0,0 L0,6 L9,3 z\" fill=\"#275DA8\"/></marker></defs>"
    );
    for (index, node) in nodes.iter().enumerate() {
        let x = 40 + index * 190;
        let y = 62;
        if has_arrows && index + 1 < nodes.len() {
            svg.push_str(&format!(
                "<line x1=\"{}\" y1=\"90\" x2=\"{}\" y2=\"90\" stroke=\"#275DA8\" stroke-width=\"3\" marker-end=\"url(#pikchr-arrow)\"/>",
                x + 120,
                x + 180
            ));
        }
        match node.shape {
            PikchrShape::Circle => {
                svg.push_str(&format!(
                    "<ellipse cx=\"{}\" cy=\"90\" rx=\"60\" ry=\"34\" fill=\"#eff6ff\" stroke=\"#275DA8\" stroke-width=\"2\"/>",
                    x + 60
                ));
            }
            PikchrShape::Box => {
                svg.push_str(&format!(
                    "<rect x=\"{x}\" y=\"{y}\" width=\"120\" height=\"56\" rx=\"6\" fill=\"#eff6ff\" stroke=\"#275DA8\" stroke-width=\"2\"/>"
                ));
            }
        }
        svg.push_str(&format!(
            "<text x=\"{}\" y=\"95\" text-anchor=\"middle\" font-size=\"14\" fill=\"#111827\">{}</text>",
            x + 60,
            escape_html(&node.label)
        ));
    }
    svg.push_str("</svg>");
    svg
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

#[derive(Clone, Copy)]
enum PikchrShape {
    Box,
    Circle,
}

struct PikchrNode {
    shape: PikchrShape,
    label: String,
}

fn parse_pikchr_nodes(body: &str) -> Vec<PikchrNode> {
    body.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("box") {
                Some(PikchrNode {
                    shape: PikchrShape::Box,
                    label: pikchr_label(trimmed, "box"),
                })
            } else if trimmed.starts_with("circle") || trimmed.starts_with("ellipse") {
                let command = if trimmed.starts_with("circle") {
                    "circle"
                } else {
                    "ellipse"
                };
                Some(PikchrNode {
                    shape: PikchrShape::Circle,
                    label: pikchr_label(trimmed, command),
                })
            } else {
                None
            }
        })
        .collect()
}

fn pikchr_label(line: &str, command: &str) -> String {
    extract_first_quoted(line)
        .or_else(|| {
            let rest = line.trim_start_matches(command).trim();
            (!rest.is_empty()).then(|| rest.to_string())
        })
        .unwrap_or_else(|| command.to_string())
}

fn extract_first_quoted(text: &str) -> Option<String> {
    let start = text.find('"')?;
    let after_start = &text[start + 1..];
    let end = after_start.find('"')?;
    Some(after_start[..end].to_string())
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
        let to = parse_mermaid_node(strip_mermaid_edge_label(right));
        add_mermaid_node(&mut nodes, &mut seen, &from);
        add_mermaid_node(&mut nodes, &mut seen, &to);
        edges.push(MermaidEdge {
            from: from.id,
            to: to.id,
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

fn strip_mermaid_edge_label(text: &str) -> &str {
    let text = text.trim();
    if let Some(rest) = text.strip_prefix('|') {
        if let Some((_, after_label)) = rest.split_once('|') {
            return after_label.trim();
        }
    }
    text
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
            add_mermaid_node(&mut nodes, &mut seen, &from);
            add_mermaid_node(&mut nodes, &mut seen, &to);
            edges.push(MermaidEdge {
                from: from.id,
                to: to.id,
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
    for line in body.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        if let Some((left, right)) = split_first_operator(line, &["<->", "->", "--"]) {
            let from = parse_plain_graph_node(left);
            let to = parse_plain_graph_node(right.split_once(':').map_or(right, |(id, _)| id));
            add_mermaid_node(&mut nodes, &mut seen, &from);
            add_mermaid_node(&mut nodes, &mut seen, &to);
            edges.push(MermaidEdge {
                from: from.id,
                to: to.id,
            });
        } else if let Some((id, label)) = line.split_once(':') {
            let node = MermaidNode {
                id: normalize_plain_node_id(id),
                label: label.trim().trim_matches('"').to_string(),
            };
            add_mermaid_node(&mut nodes, &mut seen, &node);
        }
    }
    MermaidGraph { nodes, edges }
}

fn parse_plantuml_graph(body: &str) -> MermaidGraph {
    let mut nodes = Vec::new();
    let mut seen = HashSet::new();
    let mut edges = Vec::new();
    for line in body.lines().map(str::trim) {
        if line.is_empty()
            || line.starts_with('\'')
            || line.starts_with("@start")
            || line.starts_with("@end")
        {
            continue;
        }
        if let Some((keyword, rest)) = line.split_once(' ') {
            if matches!(
                keyword,
                "actor" | "participant" | "component" | "database" | "queue" | "boundary"
            ) {
                let node = parse_plain_graph_node(rest);
                add_mermaid_node(&mut nodes, &mut seen, &node);
                continue;
            }
        }
        if let Some((left, right)) = split_first_operator(line, &["-->", "->", "<--", "<-"]) {
            let from = parse_plain_graph_node(left);
            let to = parse_plain_graph_node(right.split_once(':').map_or(right, |(id, _)| id));
            add_mermaid_node(&mut nodes, &mut seen, &from);
            add_mermaid_node(&mut nodes, &mut seen, &to);
            edges.push(MermaidEdge {
                from: from.id,
                to: to.id,
            });
        }
    }
    MermaidGraph { nodes, edges }
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
