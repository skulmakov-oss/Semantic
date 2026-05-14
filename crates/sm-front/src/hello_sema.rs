use crate::hello_parser::{HelloFile, HelloStmt};
use crate::types::FrontendError;
use alloc::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloCheckedFile {
    pub file: HelloFile,
    pub report: HelloSemaReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HelloSemaReport {
    pub canonical_shape: bool,
    pub architecture_bearing: bool,
    pub secondary_shape: bool,
    pub complete_present: bool,
    pub state_count: usize,
    pub require_count: usize,
    pub observe_count: usize,
}

pub fn validate_hello_file(file: HelloFile) -> Result<HelloCheckedFile, FrontendError> {
    let entry = &file.entry;
    if entry.name.trim().is_empty() {
        return Err(FrontendError::syntax(
            0,
            "Hello entry name must not be empty",
        ));
    }

    let mut state_names = BTreeSet::new();
    let mut report = HelloSemaReport::default();

    for stmt in &entry.body {
        match stmt {
            HelloStmt::State(state) => {
                if !state_names.insert(state.name.clone()) {
                    return Err(FrontendError::syntax(
                        0,
                        format!("duplicate Hello state symbol '{}'", state.name),
                    ));
                }
                report.state_count += 1;
            }
            HelloStmt::Require(require) => {
                if !state_names.contains(&require.state) {
                    return Err(FrontendError::syntax(
                        0,
                        format!("unknown Hello state symbol '{}'", require.state),
                    ));
                }
                report.require_count += 1;
            }
            HelloStmt::Observe(_) => {
                report.observe_count += 1;
            }
            HelloStmt::Complete(_) => {
                report.complete_present = true;
            }
        }
    }

    let canonical_shape = report.complete_present
        && report.state_count == 1
        && report.require_count == 1
        && report.observe_count == 1
        && entry.body.len() == 4;
    let secondary_shape = !report.complete_present
        && report.state_count == 0
        && report.require_count == 0
        && report.observe_count == 1
        && entry.body.len() == 1;

    if canonical_shape {
        report.canonical_shape = true;
        report.architecture_bearing = true;
    } else if secondary_shape {
        report.secondary_shape = true;
    } else {
        return Err(FrontendError::syntax(
            0,
            "Hello semantic admission requires either the canonical verbose shape or the secondary minimal-observe shape",
        ));
    }

    Ok(HelloCheckedFile { file, report })
}
