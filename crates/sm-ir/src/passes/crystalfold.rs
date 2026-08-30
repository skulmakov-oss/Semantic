use super::{IrModule, OptPass, OptReport};
use crate::frontend::QuadVal;
use crate::legacy_lowering::IrInstr;
use std::collections::HashMap;

const FX_SCALE: i32 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CrystalFoldPass;

impl OptPass for CrystalFoldPass {
    fn name(&self) -> &'static str {
        "CrystalFold"
    }

    fn version(&self) -> u32 {
        1
    }

    fn run(&self, ir: &mut IrModule) -> OptReport {
        let mut rewrites = 0u32;
        for func in &mut ir.functions {
            rewrites = rewrites.saturating_add(fold_constants_and_identities(&mut func.instrs));
        }
        OptReport {
            changed: rewrites > 0,
            num_rewrites: rewrites,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConstVal {
    Quad(QuadVal),
    Bool(bool),
    F64(f64),
    I32(i32),
    U32(u32),
    Fx(i32),
}

fn fold_constants_and_identities(instrs: &mut Vec<IrInstr>) -> u32 {
    let mut rewrites = 0u32;
    let mut out = Vec::with_capacity(instrs.len());
    let mut cst: HashMap<u16, ConstVal> = HashMap::new();

    for instr in instrs.drain(..) {
        match instr {
            IrInstr::Label { name } => {
                cst.clear();
                out.push(IrInstr::Label { name });
            }
            IrInstr::Jmp { label } => {
                cst.clear();
                out.push(IrInstr::Jmp { label });
            }
            IrInstr::JmpIf { cond, label } => {
                cst.clear();
                out.push(IrInstr::JmpIf { cond, label });
            }
            IrInstr::Assert { cond } => {
                cst.clear();
                out.push(IrInstr::Assert { cond });
            }
            IrInstr::Call { dst, name, args } => {
                cst.clear();
                out.push(IrInstr::Call { dst, name, args });
            }
            IrInstr::MakeClosure {
                dst,
                name,
                captures,
            } => {
                cst.remove(&dst);
                out.push(IrInstr::MakeClosure {
                    dst,
                    name,
                    captures,
                });
            }
            IrInstr::ClosureCall { dst, closure, arg } => {
                if let Some(dst) = dst {
                    cst.remove(&dst);
                }
                out.push(IrInstr::ClosureCall { dst, closure, arg });
            }
            IrInstr::GateRead {
                dst,
                device_id,
                port,
            } => {
                cst.remove(&dst);
                out.push(IrInstr::GateRead {
                    dst,
                    device_id,
                    port,
                });
            }
            IrInstr::GateWrite {
                device_id,
                port,
                src,
            } => {
                out.push(IrInstr::GateWrite {
                    device_id,
                    port,
                    src,
                });
            }
            IrInstr::PulseEmit { signal } => {
                out.push(IrInstr::PulseEmit { signal });
            }
            IrInstr::StateQuery { dst, key } => {
                cst.remove(&dst);
                out.push(IrInstr::StateQuery { dst, key });
            }
            IrInstr::StateUpdate { key, src } => out.push(IrInstr::StateUpdate { key, src }),
            IrInstr::EventPost { signal } => out.push(IrInstr::EventPost { signal }),
            IrInstr::ClockRead { dst } => {
                cst.remove(&dst);
                out.push(IrInstr::ClockRead { dst });
            }
            IrInstr::Ret { src } => {
                cst.clear();
                out.push(IrInstr::Ret { src });
            }
            IrInstr::LoadQ { dst, val } => {
                cst.insert(dst, ConstVal::Quad(val));
                out.push(IrInstr::LoadQ { dst, val });
            }
            IrInstr::LoadBool { dst, val } => {
                cst.insert(dst, ConstVal::Bool(val));
                out.push(IrInstr::LoadBool { dst, val });
            }
            IrInstr::LoadI32 { dst, val } => {
                cst.insert(dst, ConstVal::I32(val));
                out.push(IrInstr::LoadI32 { dst, val });
            }
            IrInstr::LoadU32 { dst, val } => {
                cst.insert(dst, ConstVal::U32(val));
                out.push(IrInstr::LoadU32 { dst, val });
            }
            IrInstr::LoadF64 { dst, val } => {
                cst.insert(dst, ConstVal::F64(val));
                out.push(IrInstr::LoadF64 { dst, val });
            }
            IrInstr::LoadFx { dst, val } => {
                cst.insert(dst, ConstVal::Fx(val));
                out.push(IrInstr::LoadFx { dst, val });
            }
            IrInstr::LoadText { dst, val } => {
                cst.remove(&dst);
                out.push(IrInstr::LoadText { dst, val });
            }
            IrInstr::MakeSequence { dst, items } => {
                cst.remove(&dst);
                out.push(IrInstr::MakeSequence { dst, items });
            }
            IrInstr::MakeTuple { dst, items } => {
                cst.remove(&dst);
                out.push(IrInstr::MakeTuple { dst, items });
            }
            IrInstr::MakeRecord { dst, name, items } => {
                cst.remove(&dst);
                out.push(IrInstr::MakeRecord { dst, name, items });
            }
            IrInstr::MakeAdt {
                dst,
                adt_name,
                variant_name,
                tag,
                items,
            } => {
                cst.remove(&dst);
                out.push(IrInstr::MakeAdt {
                    dst,
                    adt_name,
                    variant_name,
                    tag,
                    items,
                });
            }
            IrInstr::AdtTag { dst, src, adt_name } => {
                cst.remove(&dst);
                out.push(IrInstr::AdtTag { dst, src, adt_name });
            }
            IrInstr::AdtGet {
                dst,
                src,
                adt_name,
                index,
            } => {
                cst.remove(&dst);
                out.push(IrInstr::AdtGet {
                    dst,
                    src,
                    adt_name,
                    index,
                });
            }
            IrInstr::RecordGet {
                dst,
                src,
                record_name,
                index,
            } => {
                cst.remove(&dst);
                out.push(IrInstr::RecordGet {
                    dst,
                    src,
                    record_name,
                    index,
                });
            }
            IrInstr::TupleGet { dst, src, index } => {
                cst.remove(&dst);
                out.push(IrInstr::TupleGet { dst, src, index });
            }
            IrInstr::SequenceGet { dst, src, index } => {
                cst.remove(&dst);
                out.push(IrInstr::SequenceGet { dst, src, index });
            }
            IrInstr::SequenceLen { dst, src } => {
                cst.remove(&dst);
                out.push(IrInstr::SequenceLen { dst, src });
            }
            IrInstr::SequenceIsEmpty { dst, src } => {
                cst.remove(&dst);
                out.push(IrInstr::SequenceIsEmpty { dst, src });
            }
            IrInstr::SequenceContains { dst, seq, val } => {
                cst.remove(&dst);
                out.push(IrInstr::SequenceContains { dst, seq, val });
            }
            IrInstr::SequencePush { dst, seq, val } => {
                cst.remove(&dst);
                out.push(IrInstr::SequencePush { dst, seq, val });
            }
            IrInstr::SequencePrepend { dst, seq, val } => {
                cst.remove(&dst);
                out.push(IrInstr::SequencePrepend { dst, seq, val });
            }
            IrInstr::SequencePop { dst, src } => {
                cst.remove(&dst);
                out.push(IrInstr::SequencePop { dst, src });
            }
            IrInstr::MapEmpty { dst } => {
                cst.remove(&dst);
                out.push(IrInstr::MapEmpty { dst });
            }
            IrInstr::MapContains { dst, map, key } => {
                cst.remove(&dst);
                out.push(IrInstr::MapContains { dst, map, key });
            }
            IrInstr::MapGet {
                dst,
                map,
                key,
                default_val,
            } => {
                cst.remove(&dst);
                out.push(IrInstr::MapGet {
                    dst,
                    map,
                    key,
                    default_val,
                });
            }
            IrInstr::MapSet { dst, map, key, val } => {
                cst.remove(&dst);
                out.push(IrInstr::MapSet { dst, map, key, val });
            }
            IrInstr::RngSeed { dst, seed } => {
                cst.remove(&dst);
                out.push(IrInstr::RngSeed { dst, seed });
            }
            IrInstr::RngNextI32 { dst, lo, hi } => {
                cst.remove(&dst);
                out.push(IrInstr::RngNextI32 { dst, lo, hi });
            }
            IrInstr::ConcatText { dst, lhs, rhs } => {
                cst.remove(&dst);
                out.push(IrInstr::ConcatText { dst, lhs, rhs });
            }
            IrInstr::LoadVar { dst, name } => {
                cst.remove(&dst);
                out.push(IrInstr::LoadVar { dst, name });
            }
            IrInstr::StoreVar { name, src } => {
                out.push(IrInstr::StoreVar { name, src });
            }
            IrInstr::BoolNot { dst, src } => {
                if let Some(ConstVal::Bool(b)) = cst.get(&src).copied() {
                    rewrites = rewrites.saturating_add(1);
                    cst.insert(dst, ConstVal::Bool(!b));
                    out.push(IrInstr::LoadBool { dst, val: !b });
                } else {
                    cst.remove(&dst);
                    out.push(IrInstr::BoolNot { dst, src });
                }
            }
            IrInstr::QNot { dst, src } => {
                if let Some(ConstVal::Quad(q)) = cst.get(&src).copied() {
                    rewrites = rewrites.saturating_add(1);
                    let nq = quad_not_const(q);
                    cst.insert(dst, ConstVal::Quad(nq));
                    out.push(IrInstr::LoadQ { dst, val: nq });
                } else {
                    cst.remove(&dst);
                    out.push(IrInstr::QNot { dst, src });
                }
            }
            IrInstr::BoolAnd { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Bool(a)), Some(ConstVal::Bool(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Bool(a && b));
                        out.push(IrInstr::LoadBool { dst, val: a && b });
                    }
                    _ if dst == lhs && matches!(cst.get(&rhs), Some(ConstVal::Bool(true))) => {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs && matches!(cst.get(&lhs), Some(ConstVal::Bool(true))) => {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if matches!(cst.get(&lhs), Some(ConstVal::Bool(false)))
                        || matches!(cst.get(&rhs), Some(ConstVal::Bool(false))) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Bool(false));
                        out.push(IrInstr::LoadBool { dst, val: false });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::BoolAnd { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::BoolOr { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Bool(a)), Some(ConstVal::Bool(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Bool(a || b));
                        out.push(IrInstr::LoadBool { dst, val: a || b });
                    }
                    _ if dst == lhs && matches!(cst.get(&rhs), Some(ConstVal::Bool(false))) => {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs && matches!(cst.get(&lhs), Some(ConstVal::Bool(false))) => {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if matches!(cst.get(&lhs), Some(ConstVal::Bool(true)))
                        || matches!(cst.get(&rhs), Some(ConstVal::Bool(true))) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Bool(true));
                        out.push(IrInstr::LoadBool { dst, val: true });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::BoolOr { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::QAnd { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Quad(a)), Some(ConstVal::Quad(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        let v = quad_and_const(a, b);
                        cst.insert(dst, ConstVal::Quad(v));
                        out.push(IrInstr::LoadQ { dst, val: v });
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::Quad(QuadVal::S))) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs
                        && matches!(cst.get(&lhs), Some(ConstVal::Quad(QuadVal::S))) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if matches!(cst.get(&lhs), Some(ConstVal::Quad(QuadVal::N)))
                        || matches!(cst.get(&rhs), Some(ConstVal::Quad(QuadVal::N))) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Quad(QuadVal::N));
                        out.push(IrInstr::LoadQ {
                            dst,
                            val: QuadVal::N,
                        });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::QAnd { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::QOr { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Quad(a)), Some(ConstVal::Quad(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        let v = quad_or_const(a, b);
                        cst.insert(dst, ConstVal::Quad(v));
                        out.push(IrInstr::LoadQ { dst, val: v });
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::Quad(QuadVal::N))) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs
                        && matches!(cst.get(&lhs), Some(ConstVal::Quad(QuadVal::N))) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if matches!(cst.get(&lhs), Some(ConstVal::Quad(QuadVal::S)))
                        || matches!(cst.get(&rhs), Some(ConstVal::Quad(QuadVal::S))) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Quad(QuadVal::S));
                        out.push(IrInstr::LoadQ {
                            dst,
                            val: QuadVal::S,
                        });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::QOr { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::QImpl { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Quad(a)), Some(ConstVal::Quad(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        let v = quad_or_const(quad_not_const(a), b);
                        cst.insert(dst, ConstVal::Quad(v));
                        out.push(IrInstr::LoadQ { dst, val: v });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::QImpl { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::QTruthAnd { dst, lhs, rhs } => {
                cst.remove(&dst);
                out.push(IrInstr::QTruthAnd { dst, lhs, rhs });
            }
            IrInstr::QTruthOr { dst, lhs, rhs } => {
                cst.remove(&dst);
                out.push(IrInstr::QTruthOr { dst, lhs, rhs });
            }
            IrInstr::QTruthNot { dst, src } => {
                cst.remove(&dst);
                out.push(IrInstr::QTruthNot { dst, src });
            }
            IrInstr::QTruthImpl { dst, lhs, rhs } => {
                cst.remove(&dst);
                out.push(IrInstr::QTruthImpl { dst, lhs, rhs });
            }
            IrInstr::CmpEq { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(a), Some(b)) => {
                        rewrites = rewrites.saturating_add(1);
                        let eq = const_eq(a, b);
                        cst.insert(dst, ConstVal::Bool(eq));
                        out.push(IrInstr::LoadBool { dst, val: eq });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::CmpEq { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::CmpNe { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(a), Some(b)) => {
                        rewrites = rewrites.saturating_add(1);
                        let ne = !const_eq(a, b);
                        cst.insert(dst, ConstVal::Bool(ne));
                        out.push(IrInstr::LoadBool { dst, val: ne });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::CmpNe { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::CmpI32Lt { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::I32(a)), Some(ConstVal::I32(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Bool(a < b));
                        out.push(IrInstr::LoadBool { dst, val: a < b });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::CmpI32Lt { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::CmpI32Le { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::I32(a)), Some(ConstVal::I32(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Bool(a <= b));
                        out.push(IrInstr::LoadBool { dst, val: a <= b });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::CmpI32Le { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::AddI32 { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::I32(a)), Some(ConstVal::I32(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::I32(a.wrapping_add(b)));
                        out.push(IrInstr::LoadI32 {
                            dst,
                            val: a.wrapping_add(b),
                        });
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::I32(v)) if *v == 0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs
                        && matches!(cst.get(&lhs), Some(ConstVal::I32(v)) if *v == 0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::AddI32 { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::SubI32 { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::I32(a)), Some(ConstVal::I32(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::I32(a.wrapping_sub(b)));
                        out.push(IrInstr::LoadI32 {
                            dst,
                            val: a.wrapping_sub(b),
                        });
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::I32(v)) if *v == 0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::SubI32 { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::MulI32 { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::I32(a)), Some(ConstVal::I32(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::I32(a.wrapping_mul(b)));
                        out.push(IrInstr::LoadI32 {
                            dst,
                            val: a.wrapping_mul(b),
                        });
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::I32(v)) if *v == 1) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs
                        && matches!(cst.get(&lhs), Some(ConstVal::I32(v)) if *v == 1) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::MulI32 { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::DivI32 { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::I32(a)), Some(ConstVal::I32(b))) => {
                        if let Some(q) = a.checked_div(b) {
                            rewrites = rewrites.saturating_add(1);
                            cst.insert(dst, ConstVal::I32(q));
                            out.push(IrInstr::LoadI32 { dst, val: q });
                        } else {
                            cst.remove(&dst);
                            out.push(IrInstr::DivI32 { dst, lhs, rhs });
                        }
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::DivI32 { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::ModI32 { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::I32(a)), Some(ConstVal::I32(b))) => {
                        if let Some(rem) = a.checked_rem(b) {
                            rewrites = rewrites.saturating_add(1);
                            cst.insert(dst, ConstVal::I32(rem));
                            out.push(IrInstr::LoadI32 { dst, val: rem });
                        } else {
                            cst.remove(&dst);
                            out.push(IrInstr::ModI32 { dst, lhs, rhs });
                        }
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::ModI32 { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::AddF64 { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::F64(a)), Some(ConstVal::F64(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::F64(a + b));
                        out.push(IrInstr::LoadF64 { dst, val: a + b });
                    }
                    // x + (-0.0) == x for every x (including x == +/-0.0), so
                    // eliding is sound. x + (+0.0) is NOT sound to elide here:
                    // if the untracked x happens to be -0.0 at runtime,
                    // (-0.0) + (+0.0) == +0.0 per IEEE 754, a sign change this
                    // rewrite would silently miss. See FA-04-004 / #1710.
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::F64(v)) if *v == 0.0 && v.is_sign_negative()) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs
                        && matches!(cst.get(&lhs), Some(ConstVal::F64(v)) if *v == 0.0 && v.is_sign_negative()) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::AddF64 { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::SubF64 { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::F64(a)), Some(ConstVal::F64(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::F64(a - b));
                        out.push(IrInstr::LoadF64 { dst, val: a - b });
                    }
                    // x - (+0.0) == x + (-0.0) == x for every x, so eliding is
                    // sound. x - (-0.0) == x + (+0.0) is NOT sound: if the
                    // untracked x happens to be -0.0 at runtime, that would
                    // silently miss the (-0.0) + (+0.0) == +0.0 sign change.
                    // See FA-04-004 / #1710.
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::F64(v)) if *v == 0.0 && v.is_sign_positive()) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::SubF64 { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::MulF64 { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::F64(a)), Some(ConstVal::F64(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::F64(a * b));
                        out.push(IrInstr::LoadF64 { dst, val: a * b });
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::F64(v)) if *v == 1.0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs
                        && matches!(cst.get(&lhs), Some(ConstVal::F64(v)) if *v == 1.0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::MulF64 { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::DivF64 { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::F64(a)), Some(ConstVal::F64(b))) if b != 0.0 => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::F64(a / b));
                        out.push(IrInstr::LoadF64 { dst, val: a / b });
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::F64(v)) if *v == 1.0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::DivF64 { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::AddFx { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Fx(a)), Some(ConstVal::Fx(b))) => {
                        if let Some(sum) = fx_add_raw(a, b) {
                            rewrites = rewrites.saturating_add(1);
                            cst.insert(dst, ConstVal::Fx(sum));
                            out.push(IrInstr::LoadFx { dst, val: sum });
                        } else {
                            cst.remove(&dst);
                            out.push(IrInstr::AddFx { dst, lhs, rhs });
                        }
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::Fx(v)) if *v == 0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs
                        && matches!(cst.get(&lhs), Some(ConstVal::Fx(v)) if *v == 0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::AddFx { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::SubFx { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Fx(a)), Some(ConstVal::Fx(b))) => {
                        if let Some(diff) = fx_sub_raw(a, b) {
                            rewrites = rewrites.saturating_add(1);
                            cst.insert(dst, ConstVal::Fx(diff));
                            out.push(IrInstr::LoadFx { dst, val: diff });
                        } else {
                            cst.remove(&dst);
                            out.push(IrInstr::SubFx { dst, lhs, rhs });
                        }
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::Fx(v)) if *v == 0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::SubFx { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::MulFx { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Fx(a)), Some(ConstVal::Fx(b))) => {
                        if let Some(prod) = fx_mul_raw(a, b) {
                            rewrites = rewrites.saturating_add(1);
                            cst.insert(dst, ConstVal::Fx(prod));
                            out.push(IrInstr::LoadFx { dst, val: prod });
                        } else {
                            cst.remove(&dst);
                            out.push(IrInstr::MulFx { dst, lhs, rhs });
                        }
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::Fx(v)) if *v == FX_SCALE) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs
                        && matches!(cst.get(&lhs), Some(ConstVal::Fx(v)) if *v == FX_SCALE) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::MulFx { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::DivFx { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Fx(a)), Some(ConstVal::Fx(b))) if b != 0 => {
                        if let Some(quo) = fx_div_raw(a, b) {
                            rewrites = rewrites.saturating_add(1);
                            cst.insert(dst, ConstVal::Fx(quo));
                            out.push(IrInstr::LoadFx { dst, val: quo });
                        } else {
                            cst.remove(&dst);
                            out.push(IrInstr::DivFx { dst, lhs, rhs });
                        }
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::Fx(v)) if *v == FX_SCALE) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::DivFx { dst, lhs, rhs });
                    }
                }
            }
        }
    }
    *instrs = out;
    rewrites
}

fn fx_add_raw(lhs: i32, rhs: i32) -> Option<i32> {
    i32::try_from(i64::from(lhs) + i64::from(rhs)).ok()
}

fn fx_sub_raw(lhs: i32, rhs: i32) -> Option<i32> {
    i32::try_from(i64::from(lhs) - i64::from(rhs)).ok()
}

fn fx_mul_raw(lhs: i32, rhs: i32) -> Option<i32> {
    i32::try_from((i64::from(lhs) * i64::from(rhs)) / i64::from(FX_SCALE)).ok()
}

fn fx_div_raw(lhs: i32, rhs: i32) -> Option<i32> {
    if rhs == 0 {
        return None;
    }
    i32::try_from((i64::from(lhs) * i64::from(FX_SCALE)) / i64::from(rhs)).ok()
}

fn quad_to_u8_const(q: QuadVal) -> u8 {
    match q {
        QuadVal::N => 0,
        QuadVal::F => 1,
        QuadVal::T => 2,
        QuadVal::S => 3,
    }
}

fn u8_to_quad_const(v: u8) -> QuadVal {
    match v & 0b11 {
        0 => QuadVal::N,
        1 => QuadVal::F,
        2 => QuadVal::T,
        _ => QuadVal::S,
    }
}

fn quad_not_const(a: QuadVal) -> QuadVal {
    let v = quad_to_u8_const(a);
    let r = ((v & 0b10) >> 1) | ((v & 0b01) << 1);
    u8_to_quad_const(r)
}

fn quad_and_const(a: QuadVal, b: QuadVal) -> QuadVal {
    u8_to_quad_const(quad_to_u8_const(a) & quad_to_u8_const(b))
}

fn quad_or_const(a: QuadVal, b: QuadVal) -> QuadVal {
    u8_to_quad_const(quad_to_u8_const(a) | quad_to_u8_const(b))
}

fn const_eq(a: ConstVal, b: ConstVal) -> bool {
    match (a, b) {
        (ConstVal::Quad(x), ConstVal::Quad(y)) => x == y,
        (ConstVal::Bool(x), ConstVal::Bool(y)) => x == y,
        (ConstVal::F64(x), ConstVal::F64(y)) => x == y,
        (ConstVal::I32(x), ConstVal::I32(y)) => x == y,
        (ConstVal::U32(x), ConstVal::U32(y)) => x == y,
        (ConstVal::Fx(x), ConstVal::Fx(y)) => x == y,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy_lowering::{IrFunction, IrInstr};

    #[test]
    fn crystalfold_surface_stays_frozen_at_v1() {
        let pass = CrystalFoldPass;
        assert_eq!(pass.name(), "CrystalFold");
        assert_eq!(pass.version(), 1);
    }

    #[test]
    fn crystalfold_idempotent() {
        let pass = CrystalFoldPass;
        let base = IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::LoadBool { dst: 0, val: true },
                IrInstr::LoadBool { dst: 1, val: false },
                IrInstr::BoolAnd {
                    dst: 2,
                    lhs: 0,
                    rhs: 1,
                },
                IrInstr::LoadF64 { dst: 3, val: 2.0 },
                IrInstr::LoadF64 { dst: 4, val: 3.0 },
                IrInstr::AddF64 {
                    dst: 5,
                    lhs: 3,
                    rhs: 4,
                },
                IrInstr::Ret { src: Some(5) },
            ],
            ownership_events: Vec::new(),
            params: Vec::new(),
        };

        let mut m1 = IrModule {
            functions: vec![base.clone()],
        };
        let r1 = pass.run(&mut m1);
        assert!(r1.changed);

        let mut m2 = m1.clone();
        let r2 = pass.run(&mut m2);
        assert!(!r2.changed);
        assert_eq!(m1, m2);
    }

    #[test]
    fn crystalfold_clears_constant_state_across_barriers() {
        let pass = CrystalFoldPass;
        let base = IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::LoadBool { dst: 0, val: true },
                IrInstr::Call {
                    dst: None,
                    name: "side".to_string(),
                    args: vec![],
                },
                IrInstr::BoolNot { dst: 1, src: 0 },
                IrInstr::LoadBool { dst: 2, val: false },
                IrInstr::Label {
                    name: "after".to_string(),
                },
                IrInstr::BoolNot { dst: 3, src: 2 },
                IrInstr::Ret { src: Some(3) },
            ],
            ownership_events: Vec::new(),
            params: Vec::new(),
        };

        let mut module = IrModule {
            functions: vec![base.clone()],
        };
        let report = pass.run(&mut module);

        assert!(
            !report.changed,
            "CrystalFold must not propagate constants across call/label barriers"
        );
        assert_eq!(module.functions[0], base);
    }

    #[test]
    fn crystalfold_rewrite_order_and_report_are_deterministic() {
        let pass = CrystalFoldPass;
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadBool { dst: 0, val: true },
                    IrInstr::LoadBool { dst: 1, val: false },
                    IrInstr::BoolOr {
                        dst: 2,
                        lhs: 0,
                        rhs: 1,
                    },
                    IrInstr::LoadF64 { dst: 3, val: 2.0 },
                    IrInstr::LoadF64 { dst: 4, val: 3.0 },
                    IrInstr::AddF64 {
                        dst: 5,
                        lhs: 3,
                        rhs: 4,
                    },
                    IrInstr::Label {
                        name: "after".to_string(),
                    },
                    IrInstr::LoadI32 { dst: 6, val: 1 },
                    IrInstr::LoadI32 { dst: 7, val: 2 },
                    IrInstr::AddI32 {
                        dst: 8,
                        lhs: 6,
                        rhs: 7,
                    },
                    IrInstr::Ret { src: Some(8) },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
        };

        let report = pass.run(&mut module);
        assert_eq!(
            report,
            OptReport {
                changed: true,
                num_rewrites: 3,
            }
        );
        assert_eq!(
            module.functions[0].instrs,
            vec![
                IrInstr::LoadBool { dst: 0, val: true },
                IrInstr::LoadBool { dst: 1, val: false },
                IrInstr::LoadBool { dst: 2, val: true },
                IrInstr::LoadF64 { dst: 3, val: 2.0 },
                IrInstr::LoadF64 { dst: 4, val: 3.0 },
                IrInstr::LoadF64 { dst: 5, val: 5.0 },
                IrInstr::Label {
                    name: "after".to_string(),
                },
                IrInstr::LoadI32 { dst: 6, val: 1 },
                IrInstr::LoadI32 { dst: 7, val: 2 },
                IrInstr::LoadI32 { dst: 8, val: 3 },
                IrInstr::Ret { src: Some(8) },
            ]
        );

        let report_again = pass.run(&mut module);
        assert_eq!(
            report_again,
            OptReport {
                changed: false,
                num_rewrites: 0,
            }
        );
    }

    #[test]
    fn crystalfold_preserves_u32_equality_semantics() {
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadU32 { dst: 0, val: 7 },
                    IrInstr::LoadU32 { dst: 1, val: 7 },
                    IrInstr::CmpEq {
                        dst: 2,
                        lhs: 0,
                        rhs: 1,
                    },
                    IrInstr::CmpNe {
                        dst: 3,
                        lhs: 0,
                        rhs: 1,
                    },
                    IrInstr::LoadU32 { dst: 4, val: 8 },
                    IrInstr::CmpEq {
                        dst: 5,
                        lhs: 0,
                        rhs: 4,
                    },
                    IrInstr::CmpNe {
                        dst: 6,
                        lhs: 0,
                        rhs: 4,
                    },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
        };

        CrystalFoldPass.run(&mut module);

        assert_eq!(
            module.functions[0].instrs[2],
            IrInstr::LoadBool { dst: 2, val: true },
            "7u32 == 7u32 must fold to true"
        );
        assert_eq!(
            module.functions[0].instrs[3],
            IrInstr::LoadBool { dst: 3, val: false },
            "7u32 != 7u32 must fold to false"
        );
        assert_eq!(
            module.functions[0].instrs[5],
            IrInstr::LoadBool { dst: 5, val: false },
            "7u32 == 8u32 must fold to false"
        );
        assert_eq!(
            module.functions[0].instrs[6],
            IrInstr::LoadBool { dst: 6, val: true },
            "7u32 != 8u32 must fold to true"
        );
    }

    /// Regression guard for #1730: inserting the `(U32, U32)` arm into
    /// `const_eq` must not disturb equality folding for any other admitted
    /// same-family pair.
    #[test]
    fn crystalfold_const_eq_other_families_unaffected() {
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadBool { dst: 0, val: true },
                    IrInstr::LoadBool { dst: 1, val: true },
                    IrInstr::CmpEq {
                        dst: 2,
                        lhs: 0,
                        rhs: 1,
                    },
                    IrInstr::LoadI32 { dst: 3, val: 5 },
                    IrInstr::LoadI32 { dst: 4, val: 6 },
                    IrInstr::CmpEq {
                        dst: 5,
                        lhs: 3,
                        rhs: 4,
                    },
                    IrInstr::LoadF64 { dst: 6, val: 1.5 },
                    IrInstr::LoadF64 { dst: 7, val: 1.5 },
                    IrInstr::CmpNe {
                        dst: 8,
                        lhs: 6,
                        rhs: 7,
                    },
                    IrInstr::LoadFx { dst: 9, val: 3 },
                    IrInstr::LoadFx { dst: 10, val: 3 },
                    IrInstr::CmpEq {
                        dst: 11,
                        lhs: 9,
                        rhs: 10,
                    },
                    IrInstr::LoadQ {
                        dst: 12,
                        val: QuadVal::T,
                    },
                    IrInstr::LoadQ {
                        dst: 13,
                        val: QuadVal::F,
                    },
                    IrInstr::CmpNe {
                        dst: 14,
                        lhs: 12,
                        rhs: 13,
                    },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
        };

        CrystalFoldPass.run(&mut module);

        assert_eq!(
            module.functions[0].instrs[2],
            IrInstr::LoadBool { dst: 2, val: true },
            "bool equality must still fold correctly"
        );
        assert_eq!(
            module.functions[0].instrs[5],
            IrInstr::LoadBool { dst: 5, val: false },
            "i32 equality must still fold correctly"
        );
        assert_eq!(
            module.functions[0].instrs[8],
            IrInstr::LoadBool { dst: 8, val: false },
            "f64 inequality must still fold correctly"
        );
        assert_eq!(
            module.functions[0].instrs[11],
            IrInstr::LoadBool { dst: 11, val: true },
            "fx equality must still fold correctly"
        );
        assert_eq!(
            module.functions[0].instrs[14],
            IrInstr::LoadBool { dst: 14, val: true },
            "quad inequality must still fold correctly"
        );
    }

    /// FA-04-004 / #1710: `x + (+0.0)` is not a safe no-op elision when `x`'s
    /// sign is unknown at compile time. `(-0.0) + (+0.0) == +0.0` per IEEE
    /// 754, a sign change a bare `dst == lhs` elision would silently miss.
    /// Register 0 is intentionally never loaded, so CrystalFold's constant
    /// tracker has no entry for it and must treat it as an unknown runtime
    /// value (exactly the case an in-place `x = x + 0.0` targets).
    #[test]
    fn crystalfold_does_not_elide_add_f64_positive_zero_when_lhs_is_untracked() {
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadF64 { dst: 1, val: 0.0 },
                    IrInstr::AddF64 {
                        dst: 0,
                        lhs: 0,
                        rhs: 1,
                    },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
        };

        CrystalFoldPass.run(&mut module);

        assert!(
            module.functions[0].instrs.iter().any(|i| matches!(
                i,
                IrInstr::AddF64 {
                    dst: 0,
                    lhs: 0,
                    rhs: 1
                }
            )),
            "x + (+0.0) must not be elided when x's sign is unknown, got {:?}",
            module.functions[0].instrs
        );
    }

    /// Mirror of the above for the `dst == rhs` identity branch: `(+0.0) + x`
    /// carries the same unsound elision risk by commutativity.
    #[test]
    fn crystalfold_does_not_elide_add_f64_positive_zero_when_rhs_is_untracked() {
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadF64 { dst: 0, val: 0.0 },
                    IrInstr::AddF64 {
                        dst: 1,
                        lhs: 0,
                        rhs: 1,
                    },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
        };

        CrystalFoldPass.run(&mut module);

        assert!(
            module.functions[0].instrs.iter().any(|i| matches!(
                i,
                IrInstr::AddF64 {
                    dst: 1,
                    lhs: 0,
                    rhs: 1
                }
            )),
            "(+0.0) + x must not be elided when x's sign is unknown, got {:?}",
            module.functions[0].instrs
        );
    }

    /// `x + (-0.0) == x` for every x, including x == +/-0.0, so this identity
    /// remains sound and CrystalFold should keep folding it away — proves the
    /// #1710 fix narrows admission rather than disabling the optimization.
    #[test]
    fn crystalfold_still_elides_add_f64_negative_zero_when_lhs_is_untracked() {
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadF64 { dst: 1, val: -0.0 },
                    IrInstr::AddF64 {
                        dst: 0,
                        lhs: 0,
                        rhs: 1,
                    },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
        };

        CrystalFoldPass.run(&mut module);

        assert!(
            !module.functions[0]
                .instrs
                .iter()
                .any(|i| matches!(i, IrInstr::AddF64 { .. })),
            "x + (-0.0) is sound to elide and must still be folded away, got {:?}",
            module.functions[0].instrs
        );
    }

    /// FA-04-004 / #1710: `x - (-0.0)` is not safe to elide for the same
    /// reason as `x + (+0.0)` (subtraction is addition of the negation).
    #[test]
    fn crystalfold_does_not_elide_sub_f64_negative_zero_when_lhs_is_untracked() {
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadF64 { dst: 1, val: -0.0 },
                    IrInstr::SubF64 {
                        dst: 0,
                        lhs: 0,
                        rhs: 1,
                    },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
        };

        CrystalFoldPass.run(&mut module);

        assert!(
            module.functions[0].instrs.iter().any(|i| matches!(
                i,
                IrInstr::SubF64 {
                    dst: 0,
                    lhs: 0,
                    rhs: 1
                }
            )),
            "x - (-0.0) must not be elided when x's sign is unknown, got {:?}",
            module.functions[0].instrs
        );
    }

    /// `x - (+0.0) == x` for every x, so this identity remains sound.
    #[test]
    fn crystalfold_still_elides_sub_f64_positive_zero_when_lhs_is_untracked() {
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadF64 { dst: 1, val: 0.0 },
                    IrInstr::SubF64 {
                        dst: 0,
                        lhs: 0,
                        rhs: 1,
                    },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
        };

        CrystalFoldPass.run(&mut module);

        assert!(
            !module.functions[0]
                .instrs
                .iter()
                .any(|i| matches!(i, IrInstr::SubF64 { .. })),
            "x - (+0.0) is sound to elide and must still be folded away, got {:?}",
            module.functions[0].instrs
        );
    }
}
