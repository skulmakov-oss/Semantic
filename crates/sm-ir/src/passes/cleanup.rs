use super::{
    validate_activation_sites, validate_write_sites, IrModule, OptError, OptPass, OptReport,
};
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
            validate_write_sites(func)?;
            rewrites =
                rewrites.saturating_add(remove_duplicate_consecutive_labels(&mut func.instrs));

            // #1726 Checkpoint C: the only place in this pass with authority to
            // delete an annotated (Borrow-introducing) StoreVar is this proven
            // Ret/Jmp-until-Label unreachable-code deletion. It returns an
            // explicit removal receipt of the exact ActivationSiteId(s) it
            // deleted; the paired Borrow event is removed here, by that receipt
            // only, never inferred from "no StoreVar found for this site".
            //
            // #1891 Checkpoint W2B: the same deletion can just as legitimately
            // hit a write-capable instruction (StoreVar or MakeRecord)
            // annotated with a WriteSiteId. `removed_write_sites` is a second,
            // independent receipt - never conflated with `removed_sites` -
            // and drives its own coherent removal below, respecting the
            // Write-side's different cardinality (a MakeRecord site legitimately
            // pairs with 1..N Write events, never assumed to be exactly 1).
            let (removed, removed_sites, removed_write_sites) =
                remove_unreachable_until_label(&mut func.instrs);
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
            if !removed_write_sites.is_empty() {
                for site in &removed_write_sites {
                    let has_event = func
                        .ownership_events
                        .iter()
                        .any(|event| event.write_site == Some(*site));
                    if !has_event {
                        return Err(OptError(format!(
                            "function `{}`: unreachable-code cleanup deleted an annotated write-site instruction for WriteSiteId({}) but found zero paired Write event(s) to remove",
                            func.name, site.0
                        )));
                    }
                }
                // A MakeRecord site legitimately pairs with 1..N Write events
                // (unlike a StoreVar site, which pairs with exactly 1) - every
                // event carrying a removed site is removed together, never
                // partially, since the site itself (not count) is the receipt.
                func.ownership_events.retain(|event| {
                    !matches!(event.write_site, Some(w) if removed_write_sites.contains(&w))
                });
            }

            rewrites = rewrites.saturating_add(remove_noop_jumps(&mut func.instrs));
            rewrites =
                rewrites.saturating_add(remove_redundant_consecutive_loads(&mut func.instrs));
            validate_activation_sites(func)?;
            validate_write_sites(func)?;
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
) -> (
    u32,
    Vec<crate::legacy_lowering::ActivationSiteId>,
    Vec<crate::legacy_lowering::WriteSiteId>,
) {
    let before = instrs.len();
    let mut out = Vec::with_capacity(instrs.len());
    let mut removed_sites = Vec::new();
    let mut removed_write_sites = Vec::new();
    let mut unreachable = false;
    for instr in instrs.drain(..) {
        match &instr {
            IrInstr::Label { .. } => {
                unreachable = false;
                out.push(instr);
            }
            _ if unreachable => match &instr {
                IrInstr::StoreVar {
                    activation_site,
                    write_site,
                    ..
                } => {
                    if let Some(site) = activation_site {
                        removed_sites.push(*site);
                    }
                    if let Some(w) = write_site {
                        removed_write_sites.push(*w);
                    }
                }
                IrInstr::MakeRecord {
                    write_site: Some(w),
                    ..
                } => {
                    removed_write_sites.push(*w);
                }
                _ => {}
            },
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
    (removed, removed_sites, removed_write_sites)
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
    use crate::frontend::SymbolId;
    use crate::legacy_lowering::{
        AccessPath, ActivationSiteId, IrFunction, OwnershipPathEvent, OwnershipPathEventKind,
        WriteSiteId,
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
                        write_site: None,
                    },
                ],
                ownership_events: vec![OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new("__sm_local_0_pair".to_string()),
                    activation_site: Some(site),
                    write_site: None,
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
                        write_site: None,
                    },
                    IrInstr::StoreVar {
                        name: "__sm_local_2_other".to_string(),
                        src: 1,
                        activation_site: Some(site),
                        write_site: None,
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: vec![OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new("__sm_local_0_pair".to_string()),
                    activation_site: Some(site),
                    write_site: None,
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
                    write_site: None,
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

    // #1891 Checkpoint W2B (item 8.A): the same unreachable-code deletion
    // that #1726 proved for a Borrow-introducing StoreVar must be equally
    // coherent for a write-site-annotated one - deleting the annotated
    // instruction without its paired Write event(s) would leave orphaned
    // metadata, which `validate_write_sites` must never silently tolerate.
    #[test]
    fn structural_cleanup_removes_unreachable_write_site_store_var_and_its_paired_event() {
        let site = WriteSiteId(0);
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::Ret { src: None },
                    IrInstr::StoreVar {
                        name: "__sm_local_1_x".to_string(),
                        src: 0,
                        activation_site: None,
                        write_site: Some(site),
                    },
                ],
                ownership_events: vec![OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Write,
                    path: AccessPath::new("__sm_local_1_x".to_string()),
                    activation_site: None,
                    write_site: Some(site),
                }],
                params: Vec::new(),
            }],
        };

        let report = StructuralCleanupPass
            .run(&mut module)
            .expect("coherent unreachable write-site removal must not be rejected");
        assert!(report.changed);
        assert!(module.functions[0]
            .instrs
            .iter()
            .all(|i| !matches!(i, IrInstr::StoreVar { .. })));
        assert!(module.functions[0].ownership_events.is_empty());
    }

    // Item 8.B: multiple annotated StoreVars (e.g. a dead tuple-destructuring
    // assignment) after the same Ret - each site's own removal receipt must
    // drive removal of exactly its own paired event, never conflated with
    // another site's.
    #[test]
    fn structural_cleanup_removes_multiple_unreachable_write_sites_and_their_paired_events() {
        let site0 = WriteSiteId(0);
        let site1 = WriteSiteId(1);
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::Ret { src: None },
                    IrInstr::StoreVar {
                        name: "__sm_local_1_a".to_string(),
                        src: 0,
                        activation_site: None,
                        write_site: Some(site0),
                    },
                    IrInstr::StoreVar {
                        name: "__sm_local_2_b".to_string(),
                        src: 1,
                        activation_site: None,
                        write_site: Some(site1),
                    },
                ],
                ownership_events: vec![
                    OwnershipPathEvent {
                        kind: OwnershipPathEventKind::Write,
                        path: AccessPath::new("__sm_local_1_a".to_string()),
                        activation_site: None,
                        write_site: Some(site0),
                    },
                    OwnershipPathEvent {
                        kind: OwnershipPathEventKind::Write,
                        path: AccessPath::new("__sm_local_2_b".to_string()),
                        activation_site: None,
                        write_site: Some(site1),
                    },
                ],
                params: Vec::new(),
            }],
        };

        let report = StructuralCleanupPass
            .run(&mut module)
            .expect("coherent unreachable write-site removal must not be rejected");
        assert!(report.changed);
        assert!(module.functions[0]
            .instrs
            .iter()
            .all(|i| !matches!(i, IrInstr::StoreVar { .. })));
        assert!(module.functions[0].ownership_events.is_empty());
    }

    // Item 8.C: a dead RecordUpdate's MakeRecord carries one WriteSiteId
    // shared by N (here 2) field-Write events. Removing the annotated
    // MakeRecord must remove ALL of them together - the site itself, not a
    // 1:1 receipt-to-event count, is the authority (item 4).
    #[test]
    fn structural_cleanup_removes_unreachable_make_record_site_and_all_its_write_events() {
        let site = WriteSiteId(0);
        let field_a = SymbolId(10);
        let field_b = SymbolId(11);
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::Ret { src: None },
                    IrInstr::MakeRecord {
                        dst: 2,
                        name: "R".to_string(),
                        items: vec![0, 1],
                        write_site: Some(site),
                    },
                ],
                ownership_events: vec![
                    OwnershipPathEvent {
                        kind: OwnershipPathEventKind::Write,
                        path: AccessPath::new("__sm_local_0_base".to_string()).field(field_a),
                        activation_site: None,
                        write_site: Some(site),
                    },
                    OwnershipPathEvent {
                        kind: OwnershipPathEventKind::Write,
                        path: AccessPath::new("__sm_local_0_base".to_string()).field(field_b),
                        activation_site: None,
                        write_site: Some(site),
                    },
                ],
                params: Vec::new(),
            }],
        };

        let report = StructuralCleanupPass
            .run(&mut module)
            .expect("coherent unreachable MakeRecord write-site removal must not be rejected");
        assert!(report.changed);
        assert!(module.functions[0]
            .instrs
            .iter()
            .all(|i| !matches!(i, IrInstr::MakeRecord { .. })));
        assert!(
            module.functions[0].ownership_events.is_empty(),
            "both field-Write events sharing the removed MakeRecord's site must be removed together, got {:?}",
            module.functions[0].ownership_events
        );
    }

    // Item 8.D: a reachable RecordUpdate's MakeRecord/events must survive
    // completely untouched while an unreachable occurrence elsewhere in the
    // same function is removed coherently - the site is the sole identity
    // used for matching, never root, field, ordering, or proximity (item 5).
    #[test]
    fn structural_cleanup_leaves_reachable_make_record_site_untouched_while_removing_unreachable_one(
    ) {
        let live_site = WriteSiteId(0);
        let dead_site = WriteSiteId(1);
        let field_a = SymbolId(10);
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::MakeRecord {
                        dst: 2,
                        name: "R".to_string(),
                        items: vec![0, 1],
                        write_site: Some(live_site),
                    },
                    IrInstr::Ret { src: None },
                    IrInstr::MakeRecord {
                        dst: 3,
                        name: "R".to_string(),
                        items: vec![0, 1],
                        write_site: Some(dead_site),
                    },
                ],
                ownership_events: vec![
                    OwnershipPathEvent {
                        kind: OwnershipPathEventKind::Write,
                        path: AccessPath::new("__sm_local_0_live".to_string()).field(field_a),
                        activation_site: None,
                        write_site: Some(live_site),
                    },
                    OwnershipPathEvent {
                        kind: OwnershipPathEventKind::Write,
                        path: AccessPath::new("__sm_local_1_dead".to_string()).field(field_a),
                        activation_site: None,
                        write_site: Some(dead_site),
                    },
                ],
                params: Vec::new(),
            }],
        };

        let report = StructuralCleanupPass
            .run(&mut module)
            .expect("mixed reachable/unreachable MakeRecord sites must be handled coherently");
        assert!(report.changed);
        assert_eq!(
            module.functions[0]
                .instrs
                .iter()
                .filter(|i| matches!(i, IrInstr::MakeRecord { write_site: Some(w), .. } if *w == live_site))
                .count(),
            1,
            "the reachable MakeRecord must survive unchanged"
        );
        assert!(
            module.functions[0].instrs.iter().all(
                |i| !matches!(i, IrInstr::MakeRecord { write_site: Some(w), .. } if *w == dead_site)
            ),
            "the unreachable MakeRecord must be removed"
        );
        assert_eq!(
            module.functions[0].ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Write,
                path: AccessPath::new("__sm_local_0_live".to_string()).field(field_a),
                activation_site: None,
                write_site: Some(live_site),
            }],
            "only the dead site's Write event must be removed, the live one survives untouched"
        );
    }

    // Item 8.E: two reachable reassignments to the same binding must keep
    // their own distinct WriteSiteIds - the optimizer has no dead code here
    // and must not merge or drop either producer-B site.
    #[test]
    fn structural_cleanup_preserves_distinct_write_sites_for_repeated_reachable_assignments() {
        let site0 = WriteSiteId(0);
        let site1 = WriteSiteId(1);
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::StoreVar {
                        name: "__sm_local_1_x".to_string(),
                        src: 0,
                        activation_site: None,
                        write_site: Some(site0),
                    },
                    IrInstr::StoreVar {
                        name: "__sm_local_1_x".to_string(),
                        src: 1,
                        activation_site: None,
                        write_site: Some(site1),
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: vec![
                    OwnershipPathEvent {
                        kind: OwnershipPathEventKind::Write,
                        path: AccessPath::new("__sm_local_1_x".to_string()),
                        activation_site: None,
                        write_site: Some(site0),
                    },
                    OwnershipPathEvent {
                        kind: OwnershipPathEventKind::Write,
                        path: AccessPath::new("__sm_local_1_x".to_string()),
                        activation_site: None,
                        write_site: Some(site1),
                    },
                ],
                params: Vec::new(),
            }],
        };

        let report = StructuralCleanupPass
            .run(&mut module)
            .expect("both reachable reassignments must survive with distinct write sites");
        assert!(
            !report.changed,
            "no dead code here, nothing should be rewritten"
        );
        let sites: Vec<_> = module.functions[0]
            .instrs
            .iter()
            .filter_map(|i| match i {
                IrInstr::StoreVar {
                    write_site: Some(w),
                    ..
                } => Some(*w),
                _ => None,
            })
            .collect();
        assert_eq!(
            sites,
            vec![site0, site1],
            "both distinct sites must survive, unmerged and in order"
        );
        assert_eq!(module.functions[0].ownership_events.len(), 2);
    }

    // Item 8.F: two branch arms assigning the same root binding must keep
    // distinct static WriteSiteIds - the optimizer must never collapse or
    // duplicate them merely because their paths/roots are equal. Both arms
    // are physically reachable (a JmpIf target, not Ret/Jmp-until-Label dead
    // code), so `remove_unreachable_until_label` must not touch either.
    #[test]
    fn structural_cleanup_does_not_collapse_distinct_write_sites_across_a_branch() {
        let then_site = WriteSiteId(0);
        let else_site = WriteSiteId(1);
        let mut module = IrModule {
            functions: vec![IrFunction {
                name: "main".to_string(),
                instrs: vec![
                    IrInstr::JmpIf {
                        cond: 0,
                        label: "else_arm".to_string(),
                    },
                    IrInstr::StoreVar {
                        name: "__sm_local_1_x".to_string(),
                        src: 1,
                        activation_site: None,
                        write_site: Some(then_site),
                    },
                    IrInstr::Jmp {
                        label: "join".to_string(),
                    },
                    IrInstr::Label {
                        name: "else_arm".to_string(),
                    },
                    IrInstr::StoreVar {
                        name: "__sm_local_1_x".to_string(),
                        src: 2,
                        activation_site: None,
                        write_site: Some(else_site),
                    },
                    IrInstr::Label {
                        name: "join".to_string(),
                    },
                    IrInstr::Ret { src: None },
                ],
                ownership_events: vec![
                    OwnershipPathEvent {
                        kind: OwnershipPathEventKind::Write,
                        path: AccessPath::new("__sm_local_1_x".to_string()),
                        activation_site: None,
                        write_site: Some(then_site),
                    },
                    OwnershipPathEvent {
                        kind: OwnershipPathEventKind::Write,
                        path: AccessPath::new("__sm_local_1_x".to_string()),
                        activation_site: None,
                        write_site: Some(else_site),
                    },
                ],
                params: Vec::new(),
            }],
        };

        let report = StructuralCleanupPass
            .run(&mut module)
            .expect("both branch arms are reachable and must not be touched");
        assert!(!report.changed);
        let sites: Vec<_> = module.functions[0]
            .instrs
            .iter()
            .filter_map(|i| match i {
                IrInstr::StoreVar {
                    write_site: Some(w),
                    ..
                } => Some(*w),
                _ => None,
            })
            .collect();
        assert_eq!(
            sites,
            vec![then_site, else_site],
            "each branch's own static write site must remain distinct, never collapsed just because the root path is equal"
        );
        assert_eq!(module.functions[0].ownership_events.len(), 2);
    }
}
