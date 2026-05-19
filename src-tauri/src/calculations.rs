use crate::diagnostics::DocumentDiagnostic;
use crate::{compiler_support::collect_fence_bodies, diagnostics::diag};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub(crate) struct FormulaValue {
    pub(crate) name: String,
    pub(crate) expression: String,
    pub(crate) value: Option<f64>,
    pub(crate) error: Option<String>,
    pub(crate) dependencies: Vec<String>,
    pub(crate) ast: Option<FormulaAstNode>,
}

#[derive(Debug, Serialize)]
pub(crate) struct FormulaDependencyEdge {
    pub(crate) from: String,
    pub(crate) to: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum FormulaAstNode {
    Number {
        value: f64,
    },
    Name {
        name: String,
    },
    Unary {
        op: String,
        expr: Box<FormulaAstNode>,
    },
    Binary {
        op: String,
        left: Box<FormulaAstNode>,
        right: Box<FormulaAstNode>,
    },
    Function {
        name: String,
        args: Vec<FormulaAstNode>,
    },
}

pub(crate) fn collect_calculations(
    text: &str,
    context: &mut HashMap<String, f64>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> Vec<FormulaValue> {
    let mut definitions = Vec::new();
    let mut definition_index = HashMap::new();
    for block in collect_fence_bodies(text, "calc") {
        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((name, expression)) = trimmed.split_once('=') {
                let name = name.trim().to_string();
                let expression = expression.trim().to_string();
                let dependencies = expression_dependencies(&expression);
                definition_index.insert(name.clone(), definitions.len());
                definitions.push(FormulaDefinition {
                    name,
                    expression,
                    dependencies,
                });
            }
        }
    }

    let mut cache = HashMap::new();
    let mut formulas = Vec::new();
    for index in 0..definitions.len() {
        let definition = &definitions[index];
        let mut stack = Vec::new();
        match evaluate_formula_definition(
            index,
            &definitions,
            &definition_index,
            context,
            &mut cache,
            &mut stack,
        ) {
            Ok(value) => formulas.push(FormulaValue {
                name: definition.name.clone(),
                expression: definition.expression.clone(),
                value: Some(value),
                error: None,
                dependencies: definition.dependencies.clone(),
                ast: parse_formula_ast(&definition.expression).ok(),
            }),
            Err(error) => {
                diagnostics.push(diag(
                    "error",
                    format!("Formula error for {}: {error}", definition.name),
                    None,
                    None,
                    Some("Use numeric expressions, supported functions, or acyclic named values."),
                ));
                formulas.push(FormulaValue {
                    name: definition.name.clone(),
                    expression: definition.expression.clone(),
                    value: None,
                    error: Some(error),
                    dependencies: definition.dependencies.clone(),
                    ast: parse_formula_ast(&definition.expression).ok(),
                });
            }
        }
    }
    formulas
}

pub(crate) fn formula_dependency_edges(formulas: &[FormulaValue]) -> Vec<FormulaDependencyEdge> {
    formulas
        .iter()
        .flat_map(|formula| {
            formula
                .dependencies
                .iter()
                .map(|dependency| FormulaDependencyEdge {
                    from: formula.name.clone(),
                    to: dependency.clone(),
                })
        })
        .collect()
}

struct FormulaDefinition {
    name: String,
    expression: String,
    dependencies: Vec<String>,
}

fn evaluate_formula_definition(
    index: usize,
    definitions: &[FormulaDefinition],
    definition_index: &HashMap<String, usize>,
    context: &mut HashMap<String, f64>,
    cache: &mut HashMap<usize, Result<f64, String>>,
    stack: &mut Vec<usize>,
) -> Result<f64, String> {
    if let Some(value) = cache.get(&index).cloned() {
        return value;
    }
    if let Some(cycle_start) = stack.iter().position(|candidate| *candidate == index) {
        return Err(formula_cycle_error(
            index,
            &stack[cycle_start..],
            definitions,
        ));
    }

    stack.push(index);
    let result = (|| {
        for dependency in &definitions[index].dependencies {
            if let Some(dependency_index) = definition_index.get(dependency).copied() {
                let value = evaluate_formula_definition(
                    dependency_index,
                    definitions,
                    definition_index,
                    context,
                    cache,
                    stack,
                )?;
                context.insert(dependency.clone(), value);
            }
        }
        eval_expression(&definitions[index].expression, context)
    })();
    stack.pop();
    if let Ok(value) = &result {
        context.insert(definitions[index].name.clone(), *value);
    }
    cache.insert(index, result.clone());
    result
}

fn formula_cycle_error(
    repeated_index: usize,
    cycle_stack: &[usize],
    definitions: &[FormulaDefinition],
) -> String {
    let mut names = cycle_stack
        .iter()
        .map(|index| definitions[*index].name.clone())
        .collect::<Vec<_>>();
    names.push(definitions[repeated_index].name.clone());
    format!("#CYCLE? {}", names.join(" -> "))
}

pub(crate) fn eval_expression(
    expression: &str,
    context: &HashMap<String, f64>,
) -> Result<f64, String> {
    let tokens = tokenize_expression(expression)?;
    let mut parser = FormulaParser {
        tokens,
        index: 0,
        context,
    };
    let value = parser.parse_expression()?;
    if parser.index != parser.tokens.len() {
        return Err("unexpected trailing input".to_string());
    }
    Ok(value)
}

pub(crate) fn parse_formula_ast(expression: &str) -> Result<FormulaAstNode, String> {
    let tokens = tokenize_expression(expression)?;
    let mut parser = FormulaAstParser { tokens, index: 0 };
    let ast = parser.parse_expression()?;
    if parser.index != parser.tokens.len() {
        return Err("unexpected trailing input".to_string());
    }
    Ok(ast)
}

#[derive(Clone, Debug, PartialEq)]
enum FormulaToken {
    Number(f64),
    Name(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    NotEqual,
    LParen,
    RParen,
    Comma,
}

struct FormulaAstParser {
    tokens: Vec<FormulaToken>,
    index: usize,
}

impl FormulaAstParser {
    fn parse_expression(&mut self) -> Result<FormulaAstNode, String> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<FormulaAstNode, String> {
        let mut node = self.parse_additive()?;
        loop {
            let Some(token) = self.peek().cloned() else {
                return Ok(node);
            };
            let op = match token {
                FormulaToken::Greater => ">",
                FormulaToken::GreaterEqual => ">=",
                FormulaToken::Less => "<",
                FormulaToken::LessEqual => "<=",
                FormulaToken::Equal => "=",
                FormulaToken::NotEqual => "!=",
                _ => return Ok(node),
            };
            self.index += 1;
            let right = self.parse_additive()?;
            node = FormulaAstNode::Binary {
                op: op.to_string(),
                left: Box::new(node),
                right: Box::new(right),
            };
        }
    }

    fn parse_additive(&mut self) -> Result<FormulaAstNode, String> {
        let mut node = self.parse_term()?;
        loop {
            let op = match self.peek() {
                Some(FormulaToken::Plus) => "+",
                Some(FormulaToken::Minus) => "-",
                _ => return Ok(node),
            };
            self.index += 1;
            let right = self.parse_term()?;
            node = FormulaAstNode::Binary {
                op: op.to_string(),
                left: Box::new(node),
                right: Box::new(right),
            };
        }
    }

    fn parse_term(&mut self) -> Result<FormulaAstNode, String> {
        let mut node = self.parse_factor()?;
        loop {
            let op = match self.peek() {
                Some(FormulaToken::Star) => "*",
                Some(FormulaToken::Slash) => "/",
                _ => return Ok(node),
            };
            self.index += 1;
            let right = self.parse_factor()?;
            node = FormulaAstNode::Binary {
                op: op.to_string(),
                left: Box::new(node),
                right: Box::new(right),
            };
        }
    }

    fn parse_factor(&mut self) -> Result<FormulaAstNode, String> {
        let mut node = match self.next() {
            Some(FormulaToken::Number(value)) => Ok(FormulaAstNode::Number { value }),
            Some(FormulaToken::Minus) => Ok(FormulaAstNode::Unary {
                op: "-".to_string(),
                expr: Box::new(self.parse_factor()?),
            }),
            Some(FormulaToken::LParen) => {
                let node = self.parse_expression()?;
                self.expect(FormulaToken::RParen)?;
                Ok(node)
            }
            Some(FormulaToken::Name(name)) => {
                if matches!(self.peek(), Some(FormulaToken::LParen)) {
                    self.index += 1;
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Some(FormulaToken::RParen)) {
                        loop {
                            args.push(self.parse_expression()?);
                            if matches!(self.peek(), Some(FormulaToken::Comma)) {
                                self.index += 1;
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(FormulaToken::RParen)?;
                    Ok(FormulaAstNode::Function { name, args })
                } else {
                    Ok(FormulaAstNode::Name { name })
                }
            }
            other => Err(format!("unexpected token {other:?}")),
        }?;
        while matches!(self.peek(), Some(FormulaToken::Percent)) {
            self.index += 1;
            node = FormulaAstNode::Unary {
                op: "%".to_string(),
                expr: Box::new(node),
            };
        }
        Ok(node)
    }

    fn expect(&mut self, token: FormulaToken) -> Result<(), String> {
        match self.next() {
            Some(found) if found == token => Ok(()),
            other => Err(format!("expected {token:?}, found {other:?}")),
        }
    }

    fn peek(&self) -> Option<&FormulaToken> {
        self.tokens.get(self.index)
    }

    fn next(&mut self) -> Option<FormulaToken> {
        let token = self.tokens.get(self.index).cloned();
        self.index += usize::from(token.is_some());
        token
    }
}

struct FormulaParser<'a> {
    tokens: Vec<FormulaToken>,
    index: usize,
    context: &'a HashMap<String, f64>,
}

impl FormulaParser<'_> {
    fn parse_expression(&mut self) -> Result<f64, String> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<f64, String> {
        let mut value = self.parse_additive()?;
        loop {
            let Some(token) = self.peek().cloned() else {
                return Ok(value);
            };
            let comparison: fn(f64, f64) -> bool = match token {
                FormulaToken::Greater => |left: f64, right: f64| left > right,
                FormulaToken::GreaterEqual => |left: f64, right: f64| left >= right,
                FormulaToken::Less => |left: f64, right: f64| left < right,
                FormulaToken::LessEqual => |left: f64, right: f64| left <= right,
                FormulaToken::Equal => |left: f64, right: f64| (left - right).abs() < f64::EPSILON,
                FormulaToken::NotEqual => {
                    |left: f64, right: f64| (left - right).abs() >= f64::EPSILON
                }
                _ => return Ok(value),
            };
            self.index += 1;
            let right = self.parse_additive()?;
            value = if comparison(value, right) { 1.0 } else { 0.0 };
        }
    }

    fn parse_additive(&mut self) -> Result<f64, String> {
        let mut value = self.parse_term()?;
        loop {
            match self.peek() {
                Some(FormulaToken::Plus) => {
                    self.index += 1;
                    value += self.parse_term()?;
                }
                Some(FormulaToken::Minus) => {
                    self.index += 1;
                    value -= self.parse_term()?;
                }
                _ => return Ok(value),
            }
        }
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut value = self.parse_factor()?;
        loop {
            match self.peek() {
                Some(FormulaToken::Star) => {
                    self.index += 1;
                    value *= self.parse_factor()?;
                }
                Some(FormulaToken::Slash) => {
                    self.index += 1;
                    let divisor = self.parse_factor()?;
                    if divisor == 0.0 {
                        return Err("#DIV/0!".to_string());
                    }
                    value /= divisor;
                }
                _ => return Ok(value),
            }
        }
    }

    fn parse_factor(&mut self) -> Result<f64, String> {
        let mut value = match self.next() {
            Some(FormulaToken::Number(value)) => Ok(value),
            Some(FormulaToken::Minus) => Ok(-self.parse_factor()?),
            Some(FormulaToken::LParen) => {
                let value = self.parse_expression()?;
                self.expect(FormulaToken::RParen)?;
                Ok(value)
            }
            Some(FormulaToken::Name(name)) => {
                if matches!(self.peek(), Some(FormulaToken::LParen)) {
                    self.index += 1;
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Some(FormulaToken::RParen)) {
                        loop {
                            args.push(self.parse_expression()?);
                            if matches!(self.peek(), Some(FormulaToken::Comma)) {
                                self.index += 1;
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(FormulaToken::RParen)?;
                    eval_function(&name, &args)
                } else {
                    self.context
                        .get(&name)
                        .copied()
                        .ok_or_else(|| format!("#NAME? {name}"))
                }
            }
            other => Err(format!("unexpected token {other:?}")),
        }?;
        while matches!(self.peek(), Some(FormulaToken::Percent)) {
            self.index += 1;
            value /= 100.0;
        }
        Ok(value)
    }

    fn expect(&mut self, token: FormulaToken) -> Result<(), String> {
        match self.next() {
            Some(found) if found == token => Ok(()),
            other => Err(format!("expected {token:?}, found {other:?}")),
        }
    }

    fn peek(&self) -> Option<&FormulaToken> {
        self.tokens.get(self.index)
    }

    fn next(&mut self) -> Option<FormulaToken> {
        let token = self.tokens.get(self.index).cloned();
        self.index += usize::from(token.is_some());
        token
    }
}

fn tokenize_expression(expression: &str) -> Result<Vec<FormulaToken>, String> {
    let mut tokens = Vec::new();
    let chars = expression.chars().collect::<Vec<_>>();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if ch.is_whitespace() {
            index += 1;
        } else if ch.is_ascii_digit() || ch == '.' {
            let start = index;
            index += 1;
            while index < chars.len() && (chars[index].is_ascii_digit() || chars[index] == '.') {
                index += 1;
            }
            let value = chars[start..index]
                .iter()
                .collect::<String>()
                .parse::<f64>()
                .map_err(|_| "#VALUE?".to_string())?;
            tokens.push(FormulaToken::Number(value));
        } else if ch.is_ascii_alphabetic() || ch == '_' {
            let start = index;
            index += 1;
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric() || chars[index] == '_')
            {
                index += 1;
            }
            tokens.push(FormulaToken::Name(chars[start..index].iter().collect()));
        } else {
            let token = match ch {
                '+' => FormulaToken::Plus,
                '-' => FormulaToken::Minus,
                '*' => FormulaToken::Star,
                '/' => FormulaToken::Slash,
                '%' => FormulaToken::Percent,
                '(' => FormulaToken::LParen,
                ')' => FormulaToken::RParen,
                ',' => FormulaToken::Comma,
                '>' if chars.get(index + 1) == Some(&'=') => {
                    index += 1;
                    FormulaToken::GreaterEqual
                }
                '>' => FormulaToken::Greater,
                '<' if chars.get(index + 1) == Some(&'=') => {
                    index += 1;
                    FormulaToken::LessEqual
                }
                '<' => FormulaToken::Less,
                '=' if chars.get(index + 1) == Some(&'=') => {
                    index += 1;
                    FormulaToken::Equal
                }
                '=' => FormulaToken::Equal,
                '!' if chars.get(index + 1) == Some(&'=') => {
                    index += 1;
                    FormulaToken::NotEqual
                }
                '!' => return Err("unsupported formula character '!'".to_string()),
                _ => return Err(format!("unsupported formula character '{ch}'")),
            };
            tokens.push(token);
            index += 1;
        }
    }
    Ok(tokens)
}

fn eval_function(name: &str, args: &[f64]) -> Result<f64, String> {
    match name.to_ascii_uppercase().as_str() {
        "SUM" => Ok(args.iter().sum()),
        "AVG" => {
            if args.is_empty() {
                Err("#DIV/0!".to_string())
            } else {
                Ok(args.iter().sum::<f64>() / args.len() as f64)
            }
        }
        "MIN" => args
            .iter()
            .copied()
            .reduce(f64::min)
            .ok_or_else(|| "#VALUE?".to_string()),
        "MAX" => args
            .iter()
            .copied()
            .reduce(f64::max)
            .ok_or_else(|| "#VALUE?".to_string()),
        "COUNT" => Ok(args.len() as f64),
        "ROUND" => {
            let value = *args.first().ok_or_else(|| "#VALUE?".to_string())?;
            let places = args.get(1).copied().unwrap_or(0.0);
            let factor = 10f64.powf(places);
            Ok((value * factor).round() / factor)
        }
        "ABS" => args
            .first()
            .copied()
            .map(f64::abs)
            .ok_or_else(|| "#VALUE?".to_string()),
        "IF" => {
            if args.len() < 3 {
                Err("#VALUE?".to_string())
            } else if args[0] != 0.0 {
                Ok(args[1])
            } else {
                Ok(args[2])
            }
        }
        "PERCENT" => args
            .first()
            .copied()
            .map(|value| value * 100.0)
            .ok_or_else(|| "#VALUE?".to_string()),
        "CURRENCY" => args.first().copied().ok_or_else(|| "#VALUE?".to_string()),
        _ => Err(format!("#NAME? {name}")),
    }
}

fn expression_dependencies(expression: &str) -> Vec<String> {
    expression
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .filter(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .next()
                    .is_some_and(|ch| ch.is_ascii_alphabetic())
        })
        .filter(|part| {
            !matches!(
                part.to_ascii_uppercase().as_str(),
                "SUM"
                    | "AVG"
                    | "MIN"
                    | "MAX"
                    | "COUNT"
                    | "ROUND"
                    | "ABS"
                    | "IF"
                    | "PERCENT"
                    | "CURRENCY"
            )
        })
        .map(ToString::to_string)
        .collect()
}
