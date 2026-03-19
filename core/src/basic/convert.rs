use std::fmt;

use crate::{CodeItem, CodeRule, RB, TNTNumCode, TNTNumRB};

#[derive(Debug, Clone)]
pub enum ConvertError {
    CodeLengthMismatch { expected: usize, actual: usize },
    InvalidCapBit { bit: usize, max: usize },
    DuplicateCapBit { bit: usize },
    DirectionOutOfRange { value: u64 },
    ValueOverflow,
    NoExactEncoding { rb: RB },
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CodeLengthMismatch { expected, actual } => {
                write!(
                    f,
                    "code length mismatch: expected {expected} bits from rule, got {actual}"
                )
            }
            Self::InvalidCapBit { bit, max } => {
                write!(f, "cap bit index out of range: {bit} (must be 1..={max})")
            }
            Self::DuplicateCapBit { bit } => {
                write!(f, "duplicate cap bit index in one cap group: {bit}")
            }
            Self::DirectionOutOfRange { value } => {
                write!(f, "direction value out of range: {value} (must be 0..=3)")
            }
            Self::ValueOverflow => write!(f, "numeric overflow while accumulating code counts"),
            Self::NoExactEncoding { rb } => {
                write!(
                    f,
                    "cannot encode exact RB value: direction={}, red={}, blue={}",
                    rb.direction, rb.num.red, rb.num.blue
                )
            }
        }
    }
}

impl std::error::Error for ConvertError {}

#[derive(Debug, Clone, Copy)]
enum SlotKind {
    Red,
    Blue,
    Direction,
}

#[derive(Debug, Clone, Copy)]
struct Slot {
    kind: SlotKind,
    count: u64,
}

#[derive(Debug, Clone)]
struct CapGroup {
    bits: Vec<usize>,
    cap: u64,
}

#[derive(Debug, Clone)]
struct CompiledRule {
    slots: Vec<Slot>,
    caps: Vec<CapGroup>,
    slot_to_caps: Vec<Vec<usize>>,
}

fn compile_rule(rule: &CodeRule) -> Result<CompiledRule, ConvertError> {
    let mut slots = Vec::new();
    for item in &rule.default {
        match item {
            CodeItem::Red { count } => slots.push(Slot {
                kind: SlotKind::Red,
                count: *count,
            }),
            CodeItem::Blue { count } => slots.push(Slot {
                kind: SlotKind::Blue,
                count: *count,
            }),
            CodeItem::Direction { count } => slots.push(Slot {
                kind: SlotKind::Direction,
                count: *count,
            }),
            CodeItem::Space => {}
        }
    }

    let slot_len = slots.len();
    let mut caps = Vec::with_capacity(rule.extra.caps.len());
    for cap in &rule.extra.caps {
        let mut bits = Vec::with_capacity(cap.bits.len());
        let mut seen = vec![false; slot_len];
        for &bit in &cap.bits {
            if bit == 0 || bit > slot_len {
                return Err(ConvertError::InvalidCapBit { bit, max: slot_len });
            }
            let idx = bit - 1;
            if seen[idx] {
                return Err(ConvertError::DuplicateCapBit { bit });
            }
            seen[idx] = true;
            bits.push(idx);
        }
        caps.push(CapGroup { bits, cap: cap.cap });
    }

    let mut slot_to_caps = vec![Vec::new(); slot_len];
    for (cap_index, cap) in caps.iter().enumerate() {
        for &bit in &cap.bits {
            slot_to_caps[bit].push(cap_index);
        }
    }

    Ok(CompiledRule {
        slots,
        caps,
        slot_to_caps,
    })
}

fn normalize_bits(compiled: &CompiledRule, bits: &mut [bool]) {
    loop {
        let mut changed = false;

        for cap in &compiled.caps {
            let mut enabled: Vec<usize> =
                cap.bits.iter().copied().filter(|&idx| bits[idx]).collect();
            if enabled.is_empty() {
                continue;
            }

            let sum: u64 = enabled.iter().map(|&idx| compiled.slots[idx].count).sum();
            if sum <= cap.cap {
                continue;
            }

            // Keep larger count bits first, then smaller index for deterministic behavior.
            enabled.sort_unstable_by(|&a, &b| {
                compiled.slots[b]
                    .count
                    .cmp(&compiled.slots[a].count)
                    .then_with(|| a.cmp(&b))
            });

            let mut kept_sum = 0_u64;
            for idx in enabled {
                let count = compiled.slots[idx].count;
                if kept_sum.saturating_add(count) <= cap.cap {
                    kept_sum += count;
                } else if bits[idx] {
                    bits[idx] = false;
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }
}

fn ensure_code_len(compiled: &CompiledRule, code: &TNTNumCode) -> Result<(), ConvertError> {
    let expected = compiled.slots.len();
    let actual = code.0.len();
    if expected != actual {
        return Err(ConvertError::CodeLengthMismatch { expected, actual });
    }
    Ok(())
}

pub fn normalize_code(rule: &CodeRule, code: TNTNumCode) -> Result<TNTNumCode, ConvertError> {
    let compiled = compile_rule(rule)?;
    ensure_code_len(&compiled, &code)?;

    let mut bits = code.0;
    normalize_bits(&compiled, &mut bits);
    Ok(TNTNumCode(bits))
}

pub fn code_to_rb(rule: &CodeRule, code: TNTNumCode) -> Result<RB, ConvertError> {
    let compiled = compile_rule(rule)?;
    ensure_code_len(&compiled, &code)?;

    let mut bits = code.0;
    normalize_bits(&compiled, &mut bits);

    let mut red = 0_u64;
    let mut blue = 0_u64;
    let mut direction = 0_u64;

    for (idx, slot) in compiled.slots.iter().enumerate() {
        if !bits[idx] {
            continue;
        }

        match slot.kind {
            SlotKind::Red => {
                red = red
                    .checked_add(slot.count)
                    .ok_or(ConvertError::ValueOverflow)?
            }
            SlotKind::Blue => {
                blue = blue
                    .checked_add(slot.count)
                    .ok_or(ConvertError::ValueOverflow)?
            }
            SlotKind::Direction => {
                direction = direction
                    .checked_add(slot.count)
                    .ok_or(ConvertError::ValueOverflow)?
            }
        }
    }

    if direction > 3 {
        return Err(ConvertError::DirectionOutOfRange { value: direction });
    }

    Ok(RB {
        num: TNTNumRB { red, blue },
        direction: direction as usize,
    })
}

fn build_suffix_sums(slots: &[Slot]) -> (Vec<u64>, Vec<u64>, Vec<u64>) {
    let len = slots.len();
    let mut red = vec![0_u64; len + 1];
    let mut blue = vec![0_u64; len + 1];
    let mut direction = vec![0_u64; len + 1];

    for idx in (0..len).rev() {
        red[idx] = red[idx + 1];
        blue[idx] = blue[idx + 1];
        direction[idx] = direction[idx + 1];
        match slots[idx].kind {
            SlotKind::Red => red[idx] += slots[idx].count,
            SlotKind::Blue => blue[idx] += slots[idx].count,
            SlotKind::Direction => direction[idx] += slots[idx].count,
        }
    }

    (red, blue, direction)
}

#[derive(Debug, Clone, Copy)]
struct Remaining {
    red: u64,
    blue: u64,
    direction: u64,
}

fn dfs_exact(
    idx: usize,
    compiled: &CompiledRule,
    suffix_red: &[u64],
    suffix_blue: &[u64],
    suffix_direction: &[u64],
    remaining: Remaining,
    cap_used: &mut [u64],
    bits: &mut [bool],
) -> bool {
    if remaining.red > suffix_red[idx]
        || remaining.blue > suffix_blue[idx]
        || remaining.direction > suffix_direction[idx]
    {
        return false;
    }

    if idx == compiled.slots.len() {
        return remaining.red == 0 && remaining.blue == 0 && remaining.direction == 0;
    }

    let slot = compiled.slots[idx];
    let next_remaining = match slot.kind {
        SlotKind::Red => Remaining {
            red: match remaining.red.checked_sub(slot.count) {
                Some(v) => v,
                None => {
                    return dfs_exact(
                        idx + 1,
                        compiled,
                        suffix_red,
                        suffix_blue,
                        suffix_direction,
                        remaining,
                        cap_used,
                        bits,
                    );
                }
            },
            blue: remaining.blue,
            direction: remaining.direction,
        },
        SlotKind::Blue => Remaining {
            red: remaining.red,
            blue: match remaining.blue.checked_sub(slot.count) {
                Some(v) => v,
                None => {
                    return dfs_exact(
                        idx + 1,
                        compiled,
                        suffix_red,
                        suffix_blue,
                        suffix_direction,
                        remaining,
                        cap_used,
                        bits,
                    );
                }
            },
            direction: remaining.direction,
        },
        SlotKind::Direction => Remaining {
            red: remaining.red,
            blue: remaining.blue,
            direction: match remaining.direction.checked_sub(slot.count) {
                Some(v) => v,
                None => {
                    return dfs_exact(
                        idx + 1,
                        compiled,
                        suffix_red,
                        suffix_blue,
                        suffix_direction,
                        remaining,
                        cap_used,
                        bits,
                    );
                }
            },
        },
    };

    let mut caps_ok = true;
    for &cap_idx in &compiled.slot_to_caps[idx] {
        if cap_used[cap_idx].saturating_add(slot.count) > compiled.caps[cap_idx].cap {
            caps_ok = false;
            break;
        }
    }

    if caps_ok {
        for &cap_idx in &compiled.slot_to_caps[idx] {
            cap_used[cap_idx] += slot.count;
        }
        bits[idx] = true;

        if dfs_exact(
            idx + 1,
            compiled,
            suffix_red,
            suffix_blue,
            suffix_direction,
            next_remaining,
            cap_used,
            bits,
        ) {
            return true;
        }

        bits[idx] = false;
        for &cap_idx in &compiled.slot_to_caps[idx] {
            cap_used[cap_idx] -= slot.count;
        }
    }

    dfs_exact(
        idx + 1,
        compiled,
        suffix_red,
        suffix_blue,
        suffix_direction,
        remaining,
        cap_used,
        bits,
    )
}

pub fn rb_to_code(rule: &CodeRule, rb: RB) -> Result<TNTNumCode, ConvertError> {
    let compiled = compile_rule(rule)?;
    let direction = u64::try_from(rb.direction).map_err(|_| ConvertError::ValueOverflow)?;
    if direction > 3 {
        return Err(ConvertError::DirectionOutOfRange { value: direction });
    }

    let (suffix_red, suffix_blue, suffix_direction) = build_suffix_sums(&compiled.slots);
    let mut cap_used = vec![0_u64; compiled.caps.len()];
    let mut bits = vec![false; compiled.slots.len()];

    let found = dfs_exact(
        0,
        &compiled,
        &suffix_red,
        &suffix_blue,
        &suffix_direction,
        Remaining {
            red: rb.num.red,
            blue: rb.num.blue,
            direction,
        },
        &mut cap_used,
        &mut bits,
    );

    if !found {
        return Err(ConvertError::NoExactEncoding { rb });
    }

    Ok(TNTNumCode(bits))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{CodeExtra, CodeItem, CodeRule, Config, RB, Root, TNTNumCode, TNTNumRB};

    use super::*;

    fn load_rule() -> CodeRule {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let file = root.join("../test-config/config.json");
        let content = fs::read_to_string(file).expect("read test config");
        let root: Root = serde_json::from_str(&content).expect("parse json");
        let config = Config::from(root);
        config.code
    }

    #[test]
    fn code_to_rb_accumulates_red_blue_and_direction() {
        let rule = load_rule();
        let mut bits = vec![false; 32];

        bits[0] = true; // bit 1: red 340
        bits[15] = true; // bit 16: blue 1
        bits[6] = true; // bit 7: direction 2
        bits[23] = true; // bit 24: direction 1

        let rb = code_to_rb(&rule, TNTNumCode(bits)).expect("convert code to rb");
        assert_eq!(rb.num.red, 340);
        assert_eq!(rb.num.blue, 1);
        assert_eq!(rb.direction, 3);
    }

    #[test]
    fn normalize_code_applies_caps_with_high_weight_priority() {
        let rule = load_rule();
        let mut bits = vec![false; 32];

        // cap bits [12,13,14,15] with cap 10
        bits[11] = true;
        bits[12] = true;
        bits[13] = true;
        bits[14] = true;
        // cap bits [16,17,18,19] with cap 10
        bits[15] = true;
        bits[16] = true;
        bits[17] = true;
        bits[18] = true;

        let normalized = normalize_code(&rule, TNTNumCode(bits.clone())).expect("normalize");
        let actual = normalized.0;

        // red: keep 8 + 2
        assert!(actual[11]);
        assert!(!actual[12]);
        assert!(actual[13]);
        assert!(!actual[14]);

        // blue: keep 8 + 2
        assert!(!actual[15]);
        assert!(actual[16]);
        assert!(!actual[17]);
        assert!(actual[18]);

        let rb = code_to_rb(&rule, TNTNumCode(bits)).expect("convert");
        assert_eq!(rb.num.red, 10);
        assert_eq!(rb.num.blue, 10);
        assert_eq!(rb.direction, 0);
    }

    #[test]
    fn rb_roundtrip_with_code_rule() {
        let rule = load_rule();
        let expected = RB {
            num: TNTNumRB {
                red: 3110,
                blue: 2900,
            },
            direction: 1,
        };

        let code = rb_to_code(&rule, expected).expect("encode rb");
        let actual = code_to_rb(&rule, code).expect("decode code");
        assert_eq!(actual.num.red, expected.num.red);
        assert_eq!(actual.num.blue, expected.num.blue);
        assert_eq!(actual.direction, expected.direction);
    }

    #[test]
    fn rb_to_code_rejects_out_of_range_direction() {
        let rule = load_rule();
        let err = rb_to_code(
            &rule,
            RB {
                num: TNTNumRB { red: 0, blue: 0 },
                direction: 4,
            },
        )
        .expect_err("direction should be rejected");
        assert!(matches!(
            err,
            ConvertError::DirectionOutOfRange { value: 4 }
        ));
    }

    #[test]
    fn rb_to_code_reports_no_exact_encoding() {
        let rule = load_rule();
        let err = rb_to_code(
            &rule,
            RB {
                num: TNTNumRB {
                    red: 10_881,
                    blue: 0,
                },
                direction: 0,
            },
        )
        .expect_err("value should be impossible due caps");
        assert!(matches!(err, ConvertError::NoExactEncoding { .. }));
    }

    #[test]
    fn code_length_mismatch_returns_error() {
        let rule = load_rule();
        let err = normalize_code(&rule, TNTNumCode(vec![false; 31]))
            .expect_err("length mismatch should fail");
        assert!(matches!(
            err,
            ConvertError::CodeLengthMismatch {
                expected: 32,
                actual: 31
            }
        ));
    }

    #[test]
    fn dynamic_rule_length_is_supported() {
        let rule = CodeRule {
            default: vec![
                CodeItem::Red { count: 4 },
                CodeItem::Space,
                CodeItem::Blue { count: 2 },
                CodeItem::Direction { count: 1 },
            ],
            extra: CodeExtra { caps: Vec::new() },
        };
        let rb = code_to_rb(&rule, TNTNumCode(vec![true, true, true])).expect("convert");
        assert_eq!(rb.num.red, 4);
        assert_eq!(rb.num.blue, 2);
        assert_eq!(rb.direction, 1);
    }
}
