use std::{fs, path::Path};

use std::fmt::Write;

use orbit_core::{
    ast::{AstNode, ValueNode},
    error::CoreError,
};

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn format_source(source: &str) -> Result<String, FormatError> {
    let ast = orbit_core::parse(source)?;
    let mut formatter = Formatter::new();
    formatter.write_document(&ast);
    let mut output = formatter.finish();
    if !output.ends_with('\n') {
        output.push('\n');
    }
    Ok(output)
}

pub fn format_file(path: impl AsRef<Path>) -> Result<String, FormatError> {
    let source = fs::read_to_string(path.as_ref())?;
    format_source(&source)
}

pub fn format_file_in_place(path: impl AsRef<Path>) -> Result<(), FormatError> {
    let path_ref = path.as_ref();
    let formatted = format_file(path_ref)?;
    fs::write(path_ref, formatted)?;
    Ok(())
}

struct Formatter {
    output: String,
    indent: usize,
}

impl Formatter {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    fn finish(self) -> String {
        self.output
    }

    fn write_document(&mut self, node: &AstNode) {
        match node {
            AstNode::Document { body, .. } => {
                for entry in body {
                    self.write_node(entry);
                }
            }
            _ => self.write_node(node),
        }
    }

    fn write_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Entry { key, value, .. } => {
                self.write_indent();
                let _ = write!(self.output, "{}: ", key);
                self.write_value(value);
                self.output.push('\n');
            }
            AstNode::Block { name, body, .. } => {
                self.write_indent();
                let _ = writeln!(self.output, "{} {{", name);
                self.indent += 1;
                for child in body {
                    self.write_node(child);
                }
                self.indent -= 1;
                self.write_indent();
                self.output.push_str("}\n");
            }
            AstNode::Document { body, .. } => {
                for entry in body {
                    self.write_node(entry);
                }
            }
        }
    }

    fn write_value(&mut self, value: &ValueNode) {
        match value {
            ValueNode::String { value, .. } => {
                let _ = write!(self.output, "\"{}\"", escape_string(value));
            }
            ValueNode::Number { value, .. } => {
                let _ = write!(self.output, "{}", value);
            }
            ValueNode::Bool { value, .. } => {
                let _ = write!(self.output, "{}", if *value { "true" } else { "false" });
            }
            ValueNode::List { items, .. } => {
                if items.is_empty() {
                    self.output.push_str("[]");
                } else {
                    self.output.push_str("[\n");
                    self.indent += 1;
                    for (index, item) in items.iter().enumerate() {
                        self.write_indent();
                        self.write_value(item);
                        if index + 1 != items.len() {
                            self.output.push(',');
                        }
                        self.output.push('\n');
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.output.push(']');
                }
            }
            ValueNode::Object { entries, .. } => {
                if entries.is_empty() {
                    self.output.push_str("{}");
                } else {
                    self.output.push_str("{\n");
                    self.indent += 1;
                    let mut items: Vec<_> = entries.iter().collect();
                    items.sort_by(|a, b| a.key.cmp(&b.key));
                    let total = items.len();
                    for (index, entry) in items.into_iter().enumerate() {
                        self.write_indent();
                        let _ = write!(self.output, "{}: ", entry.key);
                        self.write_value(&entry.value);
                        if index + 1 != total {
                            self.output.push(',');
                        }
                        self.output.push('\n');
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.output.push('}');
                }
            }
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }
}

fn escape_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other => escaped.push(other),
        }
    }
    escaped
}
