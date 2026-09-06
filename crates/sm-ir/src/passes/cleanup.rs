use super::{validate_activation_sites, IrModule, OptError, OptPass, OptReport};
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

    fn run(&self, ir: &mut IrModule) -> Result<OptReport, OptError> {
        let mut rewrites = 0u32;
        for func in &mut ir.functions {
            validate_activation_sites(func)?;
            rewrites =
                rewrites.saturating_add(remove_duplicate_consecutive_labels(&mut func.instrs));

            // #1726 Checkpoint C: the only place in this pass with authority to
            // delete an annotated (Borrow-introducing) StoreVar is this proven
            // Ret/Jmp-until-Label unreachable-code deletion. It returns an
            // explicit removal receipt of the exact ActivationSiteId(s) it
            // deleted; the paired Borrow event is removed here, by that receipt
            // only, never inferred from "no StoreVar found for this site".
            let (removed, removed_sites) = remove_unreachable_until_label(&mut func.instrs);
            rewrites = rewrites.saturating_add(removed);
            if !removed_sites.is_empty() {
                let before = func.ownership_events.len();
                func.ownership_events.retain(|event| {
                    !matches!(event.activation_site, Some(site) if removed_sites.contains(&site))
                });
                let actually_removed = before - func.ownership_events.len();
                if actually_removed != removed_sites.len() {
                    return Err(OptError(format!(
                        "function `{}`: unreachable-code cleanup deleted {} annotated StoreVar site(s) but only found {} paired Borrow event(s) to remove",
                        func.name,
                        removed_sites.len(),
                        actually_removed
                    )));
                }
            }

            rewrites = rewrites.saturating_add(remove_noop_jumps(&mut func.instrs));
            rewrites =
                rewrites.saturating_add(remove_redundant_consecutive_loads(&mut func.instrs));
            validate_activation_sites(func)?;
        }
        Ok(OptReport {
            changed: rewrites != 0,
            num_rewrites: rewrites,
        })
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

fn remove_unreachable_until_label(
    instrs: &mut Vec<IrInstr>,
) -> (u32, Vec<crate::legacy_lowering::ActivationSiteId>) {
    let before = instrs.len();
    let mut out = Vec::with_capacity(instrs.len());
    let mut removed_sites = Vec::new();
    let mut unreachable = false;
    for instr in instrs.drain(..) {
        match &instr {
            IrInstr::Label { .. } => {
                unreachable = false;
                out.push(instr);
            }
            _ if unreachable => {
                if let IrInstr::StoreVar {
                    activation_site: Some(site),
                    ..
                } = &instr
                {
                    removed_sites.push(*site);
                }
            }
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
    (removed, removed_sites)
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
    use crate::legacy_lowering::{
        AccessPath, ActivationSiteId, IrFunction, OwnershipPathEvent, OwnershipPathEventKind,
    };

    // #1726 Checkpoint C: the original counterexample that forced the pivot
    // away from target-symbol activation — an annotated Borrow-introducing
    // StoreVar sitting in code that is genuinely unreachable (after a Ret,
    // before the next Label). `remove_unreachable_until_label` deletes it;
    // this proves the paired Borrow event is deleted too, via the explicit
    // removal receipt, not left orphaned.
    #[test]
    fn structural_cleanup_removes_unreachable_borrow_introduction_and_its_paired_event() {
        let site = ActivationSiteId(0);
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::Ret { src: None },
                    IrInstr::StoreVar {
                        name: "__sm_local_1_left".to_string(),
                        src: 0,
                        activation_site: Some(site),
                    },
                ],
                ownership_events: vec![OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new("__sm_local_0_pair".to_string()),
                    activation_site: Some(site),
                }],
                params: Vec::new(),
            }],
        };

        let report = StructuralCleanupPass
            .run(&mut module)
            .expect("coherent unreachable removal must not be rejected");
        assert!(report.changed);
        assert!(module.functions[0]
            .instrs
            .iter()
            .all(|i| !matches!(i, IrInstr::StoreVar { .. })));
        assert!(module.functions[0].ownership_events.is_empty());
    }

    // Negative control: two StoreVars sharing one ActivationSiteId is a
    // malformed lowering bug, never a valid input — the pass must fail
    // closed at the pre-pass validation, not silently pick one.
    #[test]
    fn structural_cleanup_fails_closed_on_duplicate_activation_site() {
        let site = ActivationSiteId(0);
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::StoreVar {
                        name: "__sm_local_1_left".to_string(),
                        src: 0,
                        activation_site: Some(site),
                    },
                    IrInstr::StoreVar {
                        name: "__sm_local_2_other".to_string(),
                        src: 1,
                        activation_site: Some(site),
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: vec![OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new("__sm_local_0_pair".to_string()),
                    activation_site: Some(site),
                }],
                params: Vec::new(),
            }],
        };

        assert!(StructuralCleanupPass.run(&mut module).is_err());
    }

    // Negative control: a Borrow event whose site has no matching StoreVar at
    // all (never introduced, or already lost upstream) must never be treated
    // as "missing, therefore dead" — it must fail closed, not be silently
    // dropped or silently ignored.
    #[test]
    fn structural_cleanup_fails_closed_on_orphan_borrow_event_site() {
        let site = ActivationSiteId(7);
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![IrInstr::Ret { src: None }],
                ownership_events: vec![OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new("__sm_local_0_pair".to_string()),
                    activation_site: Some(site),
                }],
                params: Vec::new(),
            }],
        };

        assert!(StructuralCleanupPass.run(&mut module).is_err());
    }

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

        let report = StructuralCleanupPass
            .run(&mut module)
            .expect("valid fixture, no activation sites");
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

        let report = StructuralCleanupPass
            .run(&mut module)
            .expect("valid fixture, no activation sites");
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

        let report = StructuralCleanupPass
            .run(&mut module)
            .expect("valid fixture, no activation sites");
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
