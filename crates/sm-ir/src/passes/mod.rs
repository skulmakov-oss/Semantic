use crate::legacy_lowering::IrFunction;

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

#[derive(Debug, Clone, PartialEq)]
pub struct IrModule {
    pub functions: Vec<IrFunction>,
}

pub trait OptPass {
    fn name(&self) -> &'static str;
    fn version(&self) -> u32;
    fn run(&self, ir: &mut IrModule) -> OptReport;
}

pub fn run_default_opt_passes(functions: &mut Vec<IrFunction>) -> OptReport {
    let mut module = IrModule {
        functions: core::mem::take(functions),
    };
    let mut report = OptReport::default();
    let cleanup = cleanup::StructuralCleanupPass;
    report.merge(cleanup.run(&mut module));
    let fold = crystalfold::CrystalFoldPass::default();
    report.merge(fold.run(&mut module));
    *functions = module.functions;
    report
}
