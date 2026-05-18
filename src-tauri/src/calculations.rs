use crate::diagnostics::DocumentDiagnostic;
use crate::{collect_fence_bodies, diagnostics::diag};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub(crate) struct FormulaValue {
    pub(crate) name: String,
    pub(crate) expression: String,
    pub(crate) value: Option<f64>,
    pub(crate) error: Option<String>,
    pub(crate) dependencies: Vec<String>,
}

pub(crate) fn collect_calculations(
    text: &str,
    context: &mut HashMap<String, f64>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> Vec<FormulaValue> {
    let mut formulas = Vec::new();
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
                match eval_expression(&expression, context) {
                    Ok(value) => {
                        context.insert(name.clone(), value);
                        formulas.push(FormulaValue {
                            name,
                            expression,
                            value: Some(value),
                            error: None,
                            dependencies,
                        });
                    }
                    Err(error) => {
                        diagnostics.push(diag(
                            "error",
                            format!("Formula error for {name}: {error}"),
                            None,
                            None,
                            Some("Use numeric expressions, supported functions, or previously defined names."),
                        ));
                        formulas.push(FormulaValue {
                            name,
                            expression,
                            value: None,
                            error: Some(error),
                            dependencies,
                        });
                    }
                }
            }
        }
    }
    formulas
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
        match self.next() {
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
        }
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
