use crate::legacy_lowering::{IrFunction, IrInstr, OwnershipPathEventKind};

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
