use std::fmt;

use crate::{CodeItem, CodeRule, RB, TNTNumCode, TNTNumRB};

#[derive(Debug, Clone)]
pub enum ConvertError {
    CodeLengthMismatch { expected: usize, actual: usize },
    InvalidCapBit { bit: usize, max: usize },
    DuplicateCapBit { bit: usize },
    OverlappingCapBit { bit: usize },
    MixedCapKinds,
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
            Self::OverlappingCapBit { bit } => {
                write!(f, "cap bit index overlaps across groups: {bit}")
            }
            Self::MixedCapKinds => {
                write!(f, "all bits in one cap group must have the same type")
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    kind: SlotKind,
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
    let mut used_by_any_cap = vec![false; slot_len];
    for cap in &rule.extra.caps {
        let mut bits = Vec::with_capacity(cap.bits.len());
        let mut seen = vec![false; slot_len];
        let mut kind = None;
        for &bit in &cap.bits {
            if bit == 0 || bit > slot_len {
                return Err(ConvertError::InvalidCapBit { bit, max: slot_len });
            }
            let idx = bit - 1;
            if seen[idx] {
                return Err(ConvertError::DuplicateCapBit { bit });
            }
            if used_by_any_cap[idx] {
                return Err(ConvertError::OverlappingCapBit { bit });
            }
            seen[idx] = true;
            used_by_any_cap[idx] = true;
            let this_kind = slots[idx].kind;
            if let Some(prev) = kind {
                if prev != this_kind {
                    return Err(ConvertError::MixedCapKinds);
                }
            } else {
                kind = Some(this_kind);
            }
            bits.push(idx);
        }
        caps.push(CapGroup {
            bits,
            cap: cap.cap,
            kind: kind.unwrap_or(SlotKind::Red),
        });
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

fn ensure_code_len(compiled: &CompiledRule, code: &TNTNumCode) -> Result<(), ConvertError> {
    let expected = compiled.slots.len();
    let actual = code.0.len();
    if expected != actual {
        return Err(ConvertError::CodeLengthMismatch { expected, actual });
    }
    Ok(())
}

pub fn code_to_rb(rule: &CodeRule, code: TNTNumCode) -> Result<RB, ConvertError> {
    let compiled = compile_rule(rule)?;
    ensure_code_len(&compiled, &code)?;

    let bits = code.0;

    let mut red = 0_u64;
    let mut blue = 0_u64;
    let mut direction = 0_u64;

    for (idx, slot) in compiled.slots.iter().enumerate() {
        if !bits[idx] {
            continue;
        }
        if !compiled.slot_to_caps[idx].is_empty() {
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

    for cap in &compiled.caps {
        let sum: u64 = cap
            .bits
            .iter()
            .filter(|&&idx| bits[idx])
            .map(|&idx| compiled.slots[idx].count)
            .sum();
        let clamped = sum.min(cap.cap);
        match cap.kind {
            SlotKind::Red => {
                red = red
                    .checked_add(clamped)
                    .ok_or(ConvertError::ValueOverflow)?
            }
            SlotKind::Blue => {
                blue = blue
                    .checked_add(clamped)
                    .ok_or(ConvertError::ValueOverflow)?
            }
            SlotKind::Direction => {
                direction = direction
                    .checked_add(clamped)
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

struct DfsCtx<'a> {
    compiled: &'a CompiledRule,
    suffix_red: &'a [u64],
    suffix_blue: &'a [u64],
    suffix_direction: &'a [u64],
    cap_used: &'a mut [u64],
    bits: &'a mut [bool],
}

fn dfs_exact(ctx: &mut DfsCtx<'_>, idx: usize, remaining: Remaining) -> bool {
    if remaining.red > ctx.suffix_red[idx]
        || remaining.blue > ctx.suffix_blue[idx]
        || remaining.direction > ctx.suffix_direction[idx]
    {
        return false;
    }

    if idx == ctx.compiled.slots.len() {
        return remaining.red == 0 && remaining.blue == 0 && remaining.direction == 0;
    }

    let slot = ctx.compiled.slots[idx];
    let next_remaining = match slot.kind {
        SlotKind::Red => Remaining {
            red: match remaining.red.checked_sub(slot.count) {
                Some(v) => v,
                None => return dfs_exact(ctx, idx + 1, remaining),
            },
            blue: remaining.blue,
            direction: remaining.direction,
        },
        SlotKind::Blue => Remaining {
            red: remaining.red,
            blue: match remaining.blue.checked_sub(slot.count) {
                Some(v) => v,
                None => return dfs_exact(ctx, idx + 1, remaining),
            },
            direction: remaining.direction,
        },
        SlotKind::Direction => Remaining {
            red: remaining.red,
            blue: remaining.blue,
            direction: match remaining.direction.checked_sub(slot.count) {
                Some(v) => v,
                None => return dfs_exact(ctx, idx + 1, remaining),
            },
        },
    };

    let mut caps_ok = true;
    for &cap_idx in &ctx.compiled.slot_to_caps[idx] {
        if ctx.cap_used[cap_idx].saturating_add(slot.count) > ctx.compiled.caps[cap_idx].cap {
            caps_ok = false;
            break;
        }
    }

    if caps_ok {
        for &cap_idx in &ctx.compiled.slot_to_caps[idx] {
            ctx.cap_used[cap_idx] += slot.count;
        }
        ctx.bits[idx] = true;

        if dfs_exact(ctx, idx + 1, next_remaining) {
            return true;
        }

        ctx.bits[idx] = false;
        for &cap_idx in &ctx.compiled.slot_to_caps[idx] {
            ctx.cap_used[cap_idx] -= slot.count;
        }
    }

    dfs_exact(ctx, idx + 1, remaining)
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

    let mut ctx = DfsCtx {
        compiled: &compiled,
        suffix_red: &suffix_red,
        suffix_blue: &suffix_blue,
        suffix_direction: &suffix_direction,
        cap_used: &mut cap_used,
        bits: &mut bits,
    };
    let found = dfs_exact(
        &mut ctx,
        0,
        Remaining {
            red: rb.num.red,
            blue: rb.num.blue,
            direction,
        },
    );

    if !found {
        return Err(ConvertError::NoExactEncoding { rb });
    }

    Ok(TNTNumCode(bits))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{CodeCaps, CodeExtra, Config, RB, Root};

    use super::*;

    fn load_rule() -> CodeRule {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let file = root.join("../test-config/config.json");
        let content = fs::read_to_string(file).expect("read test config");
        let root: Root = serde_json::from_str(&content).expect("parse json");
        let config = Config::try_from(root).expect("validate config");
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
    fn code_to_rb_applies_min_cap_to_group_sum() {
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
        let err = code_to_rb(&rule, TNTNumCode(vec![false; 31]))
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

    #[test]
    fn overlapping_caps_are_rejected() {
        let rule = CodeRule {
            default: vec![
                CodeItem::Red { count: 8 },
                CodeItem::Red { count: 4 },
                CodeItem::Red { count: 2 },
            ],
            extra: CodeExtra {
                caps: vec![
                    CodeCaps {
                        bits: vec![1, 2],
                        cap: 10,
                    },
                    CodeCaps {
                        bits: vec![2, 3],
                        cap: 10,
                    },
                ],
            },
        };
        let err = code_to_rb(&rule, TNTNumCode(vec![true, true, true]))
            .expect_err("overlapping caps should be rejected");
        assert!(matches!(err, ConvertError::OverlappingCapBit { bit: 2 }));
    }
}
