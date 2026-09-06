use crate::legacy_lowering::{IrFunction, IrInstr, OwnershipPathEventKind, WriteSiteId};

pub mod cleanup;
pub mod crystalfold;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OptReport {
    pub changed: bool,
    pub num_rewrites: u32,
}

impl OptReport {
    pub fn merge(&mut self, other: OptReport) {
        self.changed |= other.changed;
        self.num_rewrites = self.num_rewrites.saturating_add(other.num_rewrites);
    }
}

/// A pass rejected malformed or incoherent input/output rather than silently
/// optimizing it away. #1726 Checkpoint C: passes must fail closed instead of
/// inferring "StoreVar missing therefore its Borrow is dead".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptError(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct IrModule {
    pub functions: Vec<IrFunction>,
}

/// #1726 Checkpoint C normative boundary — read before adding a new pass or
/// changing an existing one's instruction-rewrite strategy.
///
/// This trait's current implementors (StructuralCleanupPass, CrystalFoldPass)
/// are proven coherent with `ActivationSiteId` ONLY because every rewrite they
/// perform is one of: keep a surviving instruction exactly as-is and in its
/// original relative order, drop it wholesale (StructuralCleanupPass's proven
/// Ret/Jmp-until-Label unreachable deletion, reported via an explicit removal
/// receipt), or rewrite a non-StoreVar instruction's own operands in place
/// (CrystalFold's constant folding). Neither pass ever clones, splits, merges,
/// substitutes, or re-materializes a `StoreVar`, and neither reorders
/// survivors relative to each other — `validate_activation_sites` plus
/// CrystalFold's exact name/activation-site passthrough check verify exactly
/// this and fail closed if it's ever violated.
///
/// `ActivationSiteId` is authority-bearing metadata, not an ordinary IR field.
/// A future pass with the authority to CLONE, MERGE, SUBSTITUTE, or MOVE an
/// annotated StoreVar onto a different instruction (or reorder survivors)
/// does NOT automatically inherit this proof. `validate_activation_sites`
/// only re-checks that the pre/post bijection still holds — it cannot, by
/// itself, detect a pass that coherently moves a site from one legitimate
/// introduction to a different, wrong one (the "moving a marker to a
/// same-target reassignment" case) unless that pass proves its own
/// preservation of input-instruction origin, the way the two passes above do
/// here. Any such future pass must define and implement its own explicit
/// provenance/removal-receipt mechanism before this checkpoint's coherence
/// claim can be extended to cover it.
///
/// #1891 Checkpoint W2B extends this same boundary to `WriteSiteId`, as a
/// separate invariant - never by reusing the Borrow receipt mechanism above.
/// `StructuralCleanupPass`'s unreachable-code deletion is the only place
/// with authority to delete a write-capable instruction (`StoreVar` or
/// `MakeRecord`); it returns an explicit `WriteSiteId` removal receipt, and
/// every Write event carrying a removed site is removed by that receipt
/// only - respecting the deliberately different Write cardinality (a
/// `StoreVar` site pairs with exactly one Write event; a `MakeRecord` site
/// pairs with 1..N, all of which are removed together when its site is
/// removed). `CrystalFold` proves byte-for-byte `WriteSiteId` passthrough
/// the same way it already proves `ActivationSiteId` passthrough.
/// `validate_write_sites` is this boundary's fail-closed check, called at
/// the same points as `validate_activation_sites`.
///
/// This proof is valid only for optimizers that keep, drop (via receipt), or
/// rewrite a write-capable instruction in place. A future pass that clones,
/// merges, substitutes, moves, or re-materializes a `StoreVar` or
/// `MakeRecord` carrying a `WriteSiteId` invalidates this proof and requires
/// its own explicit provenance design, exactly as for `ActivationSiteId`.
pub trait OptPass {
    fn name(&self) -> &'static str;
    fn version(&self) -> u32;
    fn run(&self, ir: &mut IrModule) -> Result<OptReport, OptError>;
}

/// Validates the frozen Tuple/Record Borrow activation-site bijection for one
/// function: every ActivationSiteId annotated on a StoreVar is unique, pairs
/// with exactly one Borrow ownership event carrying that same site, and no
/// Write event (or FrameEntry Borrow) ever carries a site. Called before and
/// after every optimizer pass so a pass that drops or duplicates either side
/// of the pairing fails closed instead of emitting a silently-broken artifact.
pub fn validate_activation_sites(func: &IrFunction) -> Result<(), OptError> {
    let mut store_sites = Vec::new();
    for instr in &func.instrs {
        if let IrInstr::StoreVar {
            activation_site: Some(site),
            ..
        } = instr
        {
            if store_sites.contains(site) {
                return Err(OptError(format!(
                    "function `{}`: ActivationSiteId({}) annotated on more than one StoreVar",
                    func.name, site.0
                )));
            }
            store_sites.push(*site);
        }
    }
    let mut event_sites = Vec::new();
    for event in &func.ownership_events {
        match (event.kind, event.activation_site) {
            (OwnershipPathEventKind::Write, Some(site)) => {
                return Err(OptError(format!(
                    "function `{}`: Write event must never carry an activation site, found ActivationSiteId({})",
                    func.name, site.0
                )));
            }
            (OwnershipPathEventKind::Borrow, Some(site)) => {
                if event_sites.contains(&site) {
                    return Err(OptError(format!(
                        "function `{}`: ActivationSiteId({}) claimed by more than one Borrow event",
                        func.name, site.0
                    )));
                }
                event_sites.push(site);
            }
            _ => {}
        }
    }
    if store_sites.len() != event_sites.len()
        || store_sites.iter().any(|s| !event_sites.contains(s))
    {
        return Err(OptError(format!(
            "function `{}`: activation-site bijection broken ({} annotated StoreVar site(s), {} AtStore Borrow event site(s))",
            func.name,
            store_sites.len(),
            event_sites.len()
        )));
    }
    Ok(())
}

/// #1891 Checkpoint W2A: fail-closed validator for the `WriteSiteId`
/// pairing between write-capable IR instructions (`StoreVar`, `MakeRecord`)
/// and `OwnershipPathEvent::Write` events. Mirrors `validate_activation_sites`'
/// discipline for Borrow, but enforces a deliberately different cardinality:
/// a `StoreVar` site (producers A/`assign_tuple_items` and B/`Stmt::Assign`)
/// pairs with exactly one Write event; a `MakeRecord` site (producer C,
/// `Expr::RecordUpdate`) may legitimately pair with one or more Write
/// events, since one RecordUpdate's N overridden fields share one
/// `WriteSiteId`. Never infers the opcode class from an event's
/// `AccessPath` shape and never silently discards or clears orphan
/// metadata on either side - every mismatch fails closed.
pub fn validate_write_sites(func: &IrFunction) -> Result<(), OptError> {
    use std::collections::HashMap;

    #[derive(Clone, Copy)]
    enum SiteInstr {
        StoreVar,
        MakeRecord,
    }

    let mut instr_sites: HashMap<WriteSiteId, SiteInstr> = HashMap::new();
    for instr in &func.instrs {
        let (site, kind) = match instr {
            IrInstr::StoreVar {
                write_site: Some(site),
                ..
            } => (*site, SiteInstr::StoreVar),
            IrInstr::MakeRecord {
                write_site: Some(site),
                ..
            } => (*site, SiteInstr::MakeRecord),
            _ => continue,
        };
        if instr_sites.insert(site, kind).is_some() {
            return Err(OptError(format!(
                "function `{}`: WriteSiteId({}) annotated on more than one write-capable instruction",
                func.name, site.0
            )));
        }
    }

    let mut event_counts: HashMap<WriteSiteId, usize> = HashMap::new();
    for event in &func.ownership_events {
        match (event.kind, event.write_site) {
            (OwnershipPathEventKind::Borrow, Some(site)) => {
                return Err(OptError(format!(
                    "function `{}`: Borrow event must never carry a write site, found WriteSiteId({})",
                    func.name, site.0
                )));
            }
            (OwnershipPathEventKind::Write, Some(site)) => {
                *event_counts.entry(site).or_insert(0) += 1;
            }
            (OwnershipPathEventKind::Write, None) => {
                return Err(OptError(format!(
                    "function `{}`: Write event has no WriteSiteId",
                    func.name
                )));
            }
            (OwnershipPathEventKind::Borrow, None) => {}
        }
    }

    for (site, kind) in &instr_sites {
        let count = event_counts.get(site).copied().unwrap_or(0);
        if count == 0 {
            return Err(OptError(format!(
                "function `{}`: WriteSiteId({}) is annotated on an instruction but claimed by no Write event",
                func.name, site.0
            )));
        }
        if matches!(kind, SiteInstr::StoreVar) && count != 1 {
            return Err(OptError(format!(
                "function `{}`: WriteSiteId({}) is a StoreVar site but is claimed by {} Write event(s), expected exactly 1",
                func.name, site.0, count
            )));
        }
    }

    for site in event_counts.keys() {
        if !instr_sites.contains_key(site) {
            return Err(OptError(format!(
                "function `{}`: WriteSiteId({}) is claimed by a Write event but is not annotated on any instruction",
                func.name, site.0
            )));
        }
    }

    Ok(())
}

pub fn run_default_opt_passes(functions: &mut Vec<IrFunction>) -> Result<OptReport, OptError> {
    let mut module = IrModule {
        functions: core::mem::take(functions),
    };
    let mut report = OptReport::default();
    let cleanup = cleanup::StructuralCleanupPass;
    report.merge(cleanup.run(&mut module)?);
    let fold = crystalfold::CrystalFoldPass::default();
    report.merge(fold.run(&mut module)?);
    *functions = module.functions;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy_lowering::{AccessPath, OwnershipPathEvent};

    fn store_var_with_write_site(name: &str, site: WriteSiteId) -> IrInstr {
        IrInstr::StoreVar {
            name: name.to_string(),
            src: 0,
            activation_site: None,
            write_site: Some(site),
        }
    }

    fn make_record_with_write_site(site: WriteSiteId) -> IrInstr {
        IrInstr::MakeRecord {
            dst: 0,
            name: "R".to_string(),
            items: vec![],
            write_site: Some(site),
        }
    }

    fn write_event(root: &str, site: WriteSiteId) -> OwnershipPathEvent {
        OwnershipPathEvent {
            kind: OwnershipPathEventKind::Write,
            path: AccessPath::new(root.to_string()),
            activation_site: None,
            write_site: Some(site),
        }
    }

    // #1891 Checkpoint W2B (item 9): a Write event survives but its
    // annotated instruction does not - an orphan on the event side. Must
    // fail closed, never be silently tolerated as "instruction pruned
    // elsewhere, event still meaningful".
    #[test]
    fn validate_write_sites_fails_closed_when_event_survives_without_its_instruction() {
        let func = IrFunction {
            name: "main".to_string(),
            instrs: vec![],
            ownership_events: vec![write_event("__sm_local_1_x", WriteSiteId(0))],
            params: Vec::new(),
        };
        assert!(validate_write_sites(&func).is_err());
    }

    // Item 9: the annotated instruction survives (a StoreVar site here) but
    // every Write event that should carry its site is gone - an orphan on
    // the instruction side.
    #[test]
    fn validate_write_sites_fails_closed_when_store_var_site_has_no_events() {
        let func = IrFunction {
            name: "main".to_string(),
            instrs: vec![store_var_with_write_site("__sm_local_1_x", WriteSiteId(0))],
            ownership_events: vec![],
            params: Vec::new(),
        };
        assert!(validate_write_sites(&func).is_err());
    }

    // Item 9 (the MakeRecord-specific instance of the same orphan-instruction
    // shape, listed separately since MakeRecord's own cardinality (1..N) is
    // otherwise easy to conflate with "zero is just the low end of the
    // range" - zero is never valid, only the *lower bound of the valid
    // nonzero range* is 1.
    #[test]
    fn validate_write_sites_fails_closed_when_make_record_site_has_zero_events() {
        let func = IrFunction {
            name: "main".to_string(),
            instrs: vec![make_record_with_write_site(WriteSiteId(0))],
            ownership_events: vec![],
            params: Vec::new(),
        };
        assert!(validate_write_sites(&func).is_err());
    }

    // Item 9: the same WriteSiteId annotated on two distinct write-capable
    // instructions is a malformed lowering bug, never a valid input -
    // mirrors `validate_activation_sites`'s duplicate-site check for Borrow.
    #[test]
    fn validate_write_sites_fails_closed_on_duplicate_instruction_annotation() {
        let site = WriteSiteId(0);
        let func = IrFunction {
            name: "main".to_string(),
            instrs: vec![
                store_var_with_write_site("__sm_local_1_x", site),
                store_var_with_write_site("__sm_local_2_y", site),
            ],
            ownership_events: vec![write_event("__sm_local_1_x", site)],
            params: Vec::new(),
        };
        assert!(validate_write_sites(&func).is_err());
    }

    // Item 9: a StoreVar site (producer A/B) must pair with exactly one
    // Write event - unlike MakeRecord's legitimate 1..N, a second event
    // sharing a StoreVar's site is never valid.
    #[test]
    fn validate_write_sites_fails_closed_when_store_var_site_has_more_than_one_event() {
        let site = WriteSiteId(0);
        let func = IrFunction {
            name: "main".to_string(),
            instrs: vec![store_var_with_write_site("__sm_local_1_x", site)],
            ownership_events: vec![
                write_event("__sm_local_1_x", site),
                write_event("__sm_local_1_x", site),
            ],
            params: Vec::new(),
        };
        assert!(validate_write_sites(&func).is_err());
    }

    // Positive control: a MakeRecord site legitimately claimed by N (here 2)
    // Write events must be accepted - the deliberately different cardinality
    // from StoreVar's exactly-1 (item 4), proven passing, not just documented.
    #[test]
    fn validate_write_sites_accepts_make_record_site_with_multiple_events() {
        let site = WriteSiteId(0);
        let func = IrFunction {
            name: "main".to_string(),
            instrs: vec![make_record_with_write_site(site)],
            ownership_events: vec![
                write_event("__sm_local_0_base", site),
                write_event("__sm_local_0_base", site),
            ],
            params: Vec::new(),
        };
        assert!(validate_write_sites(&func).is_ok());
    }
}
