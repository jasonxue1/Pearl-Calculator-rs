use std::{
    error::Error,
    fs,
    io::{self, Error as IoError, ErrorKind, IsTerminal},
    path::PathBuf,
};

use pearl_calculator::{CodeItem, CodeRule, Config, PearlError, RB, Root, TNTNumCode, TNTNumRB};

pub(crate) fn parse_max_tnt_num(values: Option<Vec<u64>>) -> Result<Option<TNTNumRB>, PearlError> {
    match values.as_deref() {
        None | Some([]) => Ok(None),
        Some([value]) => Ok(Some(TNTNumRB {
            red: *value,
            blue: *value,
        })),
        Some([red, blue]) => Ok(Some(TNTNumRB {
            red: *red,
            blue: *blue,
        })),
        Some(values) => Err(PearlError::InvalidMaxTntArgCount(values.len())),
    }
}

pub(crate) fn load_config(path: PathBuf) -> Result<Config, Box<dyn Error>> {
    let config_file = fs::read_to_string(path)?;
    let root: Root = serde_json::from_str(&config_file)?;
    Ok(Config::try_from(root)?)
}

pub(crate) fn parse_code_input(input: &str) -> Result<TNTNumCode, Box<dyn Error>> {
    let trimmed: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    if trimmed.is_empty() {
        return Err(IoError::new(ErrorKind::InvalidInput, "code cannot be empty").into());
    }

    let mut bits = Vec::with_capacity(trimmed.len());
    for (idx, ch) in trimmed.chars().enumerate() {
        match ch {
            '0' => bits.push(false),
            '1' => bits.push(true),
            _ => {
                return Err(IoError::new(
                    ErrorKind::InvalidInput,
                    format!("invalid code char at position {}: '{ch}'", idx + 1),
                )
                .into());
            }
        }
    }

    Ok(TNTNumCode(bits))
}

pub(crate) fn format_code_with_rule(
    rule: &CodeRule,
    code: TNTNumCode,
) -> Result<String, Box<dyn Error>> {
    let bits = code.0;
    let mut bit_idx = 0usize;
    let mut out = String::new();
    let use_color = io::stdout().is_terminal();
    let reset = "\x1b[0m";
    let red = "\x1b[1;31m";
    let blue = "\x1b[1;34m";
    let green = "\x1b[1;32m";

    for item in &rule.default {
        match item {
            CodeItem::Space => out.push(' '),
            CodeItem::Red { .. } | CodeItem::Blue { .. } | CodeItem::Direction { .. } => {
                let bit = bits.get(bit_idx).ok_or_else(|| {
                    IoError::new(
                        ErrorKind::InvalidData,
                        "rb-to-code produced fewer bits than code rule requires",
                    )
                })?;
                let ch = if *bit { '1' } else { '0' };
                if use_color {
                    let color = match item {
                        CodeItem::Red { .. } => red,
                        CodeItem::Blue { .. } => blue,
                        CodeItem::Direction { .. } => green,
                        CodeItem::Space => "",
                    };
                    out.push_str(color);
                    out.push(ch);
                    out.push_str(reset);
                } else {
                    out.push(ch);
                }
                bit_idx += 1;
            }
        }
    }

    if bit_idx != bits.len() {
        return Err(IoError::new(
            ErrorKind::InvalidData,
            "rb-to-code produced more bits than code rule requires",
        )
        .into());
    }

    Ok(out)
}

pub(crate) fn format_code_to_rb_output(rb: RB) -> String {
    if !io::stdout().is_terminal() {
        return format!(
            "direction={} red={} blue={}",
            rb.direction, rb.num.red, rb.num.blue
        );
    }

    let reset = "\x1b[0m";
    let magenta = "\x1b[1;35m";
    let yellow = "\x1b[1;33m";

    format!(
        "{}direction={}{} {}red={}{} {}blue={}{}",
        magenta, rb.direction, reset, yellow, rb.num.red, reset, yellow, rb.num.blue, reset
    )
}
