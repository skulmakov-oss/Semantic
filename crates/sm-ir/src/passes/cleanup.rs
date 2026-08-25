use super::{IrModule, OptPass, OptReport};
use crate::legacy_lowering::IrInstr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StructuralCleanupPass;

impl OptPass for StructuralCleanupPass {
    fn name(&self) -> &'static str {
        "StructuralCleanup"
    }

    fn version(&self) -> u32 {
        1
    }

    fn run(&self, ir: &mut IrModule) -> OptReport {
        let mut rewrites = 0u32;
        for func in &mut ir.functions {
            rewrites = rewrites
                .saturating_add(remove_duplicate_consecutive_labels(&mut func.instrs))
                .saturating_add(remove_unreachable_until_label(&mut func.instrs))
                .saturating_add(remove_noop_jumps(&mut func.instrs))
                .saturating_add(remove_redundant_consecutive_loads(&mut func.instrs));
        }
        OptReport {
            changed: rewrites != 0,
            num_rewrites: rewrites,
        }
    }
}

fn remove_duplicate_consecutive_labels(instrs: &mut Vec<IrInstr>) -> u32 {
    let before = instrs.len();
    let mut out = Vec::with_capacity(instrs.len());
    for instr in instrs.drain(..) {
        let dup = matches!(
            (out.last(), &instr),
            (
                Some(IrInstr::Label { name: a }),
                IrInstr::Label { name: b }
            ) if a == b
        );
        if !dup {
            out.push(instr);
        }
    }
    let removed = before.saturating_sub(out.len()) as u32;
    *instrs = out;
    removed
}

fn remove_unreachable_until_label(instrs: &mut Vec<IrInstr>) -> u32 {
    let before = instrs.len();
    let mut out = Vec::with_capacity(instrs.len());
    let mut unreachable = false;
    for instr in instrs.drain(..) {
        match &instr {
            IrInstr::Label { .. } => {
                unreachable = false;
                out.push(instr);
            }
            _ if unreachable => {}
            _ => {
                let terminal = matches!(instr, IrInstr::Ret { .. } | IrInstr::Jmp { .. });
                out.push(instr);
                if terminal {
                    unreachable = true;
                }
            }
        }
    }
    let removed = before.saturating_sub(out.len()) as u32;
    *instrs = out;
    removed
}

fn remove_noop_jumps(instrs: &mut Vec<IrInstr>) -> u32 {
    let before = instrs.len();
    let mut out = Vec::with_capacity(instrs.len());
    let mut input = core::mem::take(instrs).into_iter().peekable();
    while let Some(instr) = input.next() {
        let skip = if let IrInstr::Jmp { label } = &instr {
            matches!(
                input.peek(),
                Some(IrInstr::Label { name }) if name == label
            )
        } else {
            false
        };
        if !skip {
            out.push(instr);
        }
    }
    let removed = before.saturating_sub(out.len()) as u32;
    *instrs = out;
    removed
}

fn load_dst_and_payload(instr: &IrInstr) -> Option<(u16, u64)> {
    match instr {
        IrInstr::LoadQ { dst, val } => Some((*dst, 0x1000 | (*val as u64))),
        IrInstr::LoadBool { dst, val } => Some((*dst, 0x2000 | (*val as u64))),
        IrInstr::LoadI32 { dst, val } => Some((*dst, 0x3000 | (*val as i64 as u64))),
        IrInstr::LoadF64 { dst, val } => Some((*dst, 0x4000 | val.to_bits())),
        IrInstr::LoadFx { dst, val } => Some((*dst, 0x6000 | (*val as i64 as u64))),
        IrInstr::LoadVar { dst, name } => {
            let mut h = 0xcbf29ce484222325u64;
            for b in name.as_bytes() {
                h ^= *b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            Some((*dst, 0x5000 ^ h))
        }
        _ => None,
    }
}

fn remove_redundant_consecutive_loads(instrs: &mut Vec<IrInstr>) -> u32 {
    let before = instrs.len();
    let mut out = Vec::with_capacity(instrs.len());
    let mut input = core::mem::take(instrs).into_iter().peekable();
    while let Some(instr) = input.next() {
        let drop_curr = if let (Some(a), Some(b)) = (
            load_dst_and_payload(&instr),
            input.peek().and_then(|next| load_dst_and_payload(next)),
        ) {
            a.0 == b.0
        } else {
            false
        };
        if !drop_curr {
            out.push(instr);
        }
    }
    let removed = before.saturating_sub(out.len()) as u32;
    *instrs = out;
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy_lowering::IrFunction;

    #[test]
    fn structural_cleanup_removes_unreachable_and_noop_jmp() {
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::Label {
                        name: "entry".to_string(),
                    },
                    IrInstr::Jmp {
                        label: "l1".to_string(),
                    },
                    IrInstr::LoadBool { dst: 0, val: true },
                    IrInstr::Label {
                        name: "l1".to_string(),
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
        };

        let report = StructuralCleanupPass.run(&mut module);
        assert!(report.changed);
        assert!(matches!(
            module.functions[0].instrs[0],
            IrInstr::Label { .. }
        ));
        assert!(module.functions[0]
            .instrs
            .iter()
            .all(|i| !matches!(i, IrInstr::LoadBool { dst: 0, val: true })));
    }

    #[test]
    fn structural_cleanup_removes_redundant_consecutive_loads() {
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::LoadI32 { dst: 1, val: 10 },
                    IrInstr::LoadI32 { dst: 1, val: 11 },
                    IrInstr::Ret { src: Some(1) },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
        };

        let report = StructuralCleanupPass.run(&mut module);
        assert!(report.changed);
        let loads = module.functions[0]
            .instrs
            .iter()
            .filter(|i| matches!(i, IrInstr::LoadI32 { dst: 1, .. }))
            .count();
        assert_eq!(loads, 1);
        assert!(matches!(
            module.functions[0].instrs[0],
            IrInstr::LoadI32 { dst: 1, val: 11 }
        ));
    }

    #[test]
    fn structural_cleanup_deduplicates_consecutive_labels() {
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::Label {
                        name: "l0".to_string(),
                    },
                    IrInstr::Label {
                        name: "l0".to_string(),
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: Vec::new(),
                params: Vec::new(),
            }],
        };

        let report = StructuralCleanupPass.run(&mut module);
        assert!(report.changed);
        assert_eq!(
            module.functions[0]
                .instrs
                .iter()
                .filter(|i| matches!(i, IrInstr::Label { name } if name == "l0"))
                .count(),
            1
        );
    }
}
