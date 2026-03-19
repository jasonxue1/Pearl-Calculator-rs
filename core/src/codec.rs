use crate::{CodeItem, CodeRule, PearlError, RB, TNTNumCode, TNTNumRB};

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

fn compile_rule(rule: &CodeRule) -> Result<CompiledRule, PearlError> {
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
                return Err(PearlError::InvalidCapBit { bit, max: slot_len });
            }
            let idx = bit - 1;
            if seen[idx] {
                return Err(PearlError::DuplicateCapBit { bit });
            }
            if used_by_any_cap[idx] {
                return Err(PearlError::OverlappingCapBit { bit });
            }
            seen[idx] = true;
            used_by_any_cap[idx] = true;
            let this_kind = slots[idx].kind;
            if let Some(prev) = kind {
                if prev != this_kind {
                    return Err(PearlError::MixedCapKinds);
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

fn ensure_code_len(compiled: &CompiledRule, code: &TNTNumCode) -> Result<(), PearlError> {
    let expected = compiled.slots.len();
    let actual = code.0.len();
    if expected != actual {
        return Err(PearlError::CodeLengthMismatch { expected, actual });
    }
    Ok(())
}

pub fn code_to_rb(rule: &CodeRule, code: TNTNumCode) -> Result<RB, PearlError> {
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
                    .ok_or(PearlError::ValueOverflow)?
            }
            SlotKind::Blue => {
                blue = blue
                    .checked_add(slot.count)
                    .ok_or(PearlError::ValueOverflow)?
            }
            SlotKind::Direction => {
                direction = direction
                    .checked_add(slot.count)
                    .ok_or(PearlError::ValueOverflow)?
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
            SlotKind::Red => red = red.checked_add(clamped).ok_or(PearlError::ValueOverflow)?,
            SlotKind::Blue => blue = blue.checked_add(clamped).ok_or(PearlError::ValueOverflow)?,
            SlotKind::Direction => {
                direction = direction
                    .checked_add(clamped)
                    .ok_or(PearlError::ValueOverflow)?
            }
        }
    }

    if direction > 3 {
        return Err(PearlError::DirectionOutOfRange { value: direction });
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

pub fn rb_to_code(rule: &CodeRule, rb: RB) -> Result<TNTNumCode, PearlError> {
    let compiled = compile_rule(rule)?;
    let direction = u64::try_from(rb.direction).map_err(|_| PearlError::ValueOverflow)?;
    if direction > 3 {
        return Err(PearlError::DirectionOutOfRange { value: direction });
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
        return Err(PearlError::NoExactEncoding { rb });
    }

    Ok(TNTNumCode(bits))
}
