use sm_front::hello_parser::{
    HelloCompleteQuad, HelloEntry, HelloObserveText, HelloQuadLit, HelloRequireQuadEq, HelloStateDecl,
    HelloStmt,
};
use sm_front::hello_sema::HelloCheckedFile;
use sm_front::FrontendError;
use std::string::String;
use std::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloIrModule {
    pub entry: HelloIrEntry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloIrEntry {
    pub name: String,
    pub body: Vec<HelloIrStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelloIrStmt {
    LocalQuad(HelloIrLocalQuad),
    RequireQuadEq(HelloIrRequireQuadEq),
    ObserveText(HelloIrObserveText),
    CompleteQuad(HelloIrCompleteQuad),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloIrLocalQuad {
    pub symbol: String,
    pub value: HelloIrQuadLit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloIrRequireQuadEq {
    pub symbol: String,
    pub expected: HelloIrQuadLit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloIrObserveText {
    pub text: String,
    pub observation_class: HelloIrObservationClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloIrCompleteQuad {
    pub value: HelloIrQuadLit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloIrObservationClass {
    Controlled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelloIrQuadLit {
    T,
    F,
    N,
    S,
}

pub fn lower_hello_checked_file(
    checked: &HelloCheckedFile,
) -> Result<HelloIrModule, FrontendError> {
    if checked.report.secondary_shape {
        return Err(FrontendError::syntax(
            0,
            "secondary Hello shape is not admitted for IR lowering",
        ));
    }
    if !checked.report.canonical_shape || !checked.report.architecture_bearing {
        return Err(FrontendError::syntax(
            0,
            "only the canonical verbose Hello shape is admitted for IR lowering",
        ));
    }

    let entry = lower_entry(&checked.file.entry)?;
    Ok(HelloIrModule { entry })
}

fn lower_entry(entry: &HelloEntry) -> Result<HelloIrEntry, FrontendError> {
    let body = entry
        .body
        .iter()
        .map(lower_stmt)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(HelloIrEntry {
        name: entry.name.clone(),
        body,
    })
}

fn lower_stmt(stmt: &HelloStmt) -> Result<HelloIrStmt, FrontendError> {
    match stmt {
        HelloStmt::State(HelloStateDecl { name, value, .. }) => Ok(HelloIrStmt::LocalQuad(
            HelloIrLocalQuad {
                symbol: name.clone(),
                value: (*value).into(),
            },
        )),
        HelloStmt::Require(HelloRequireQuadEq { state, value }) => {
            Ok(HelloIrStmt::RequireQuadEq(HelloIrRequireQuadEq {
                symbol: state.clone(),
                expected: (*value).into(),
            }))
        }
        HelloStmt::Observe(HelloObserveText { text }) => Ok(HelloIrStmt::ObserveText(
            HelloIrObserveText {
                text: text.clone(),
                observation_class: HelloIrObservationClass::Controlled,
            },
        )),
        HelloStmt::Complete(HelloCompleteQuad { value }) => Ok(HelloIrStmt::CompleteQuad(
            HelloIrCompleteQuad {
                value: (*value).into(),
            },
        )),
    }
}

impl From<HelloQuadLit> for HelloIrQuadLit {
    fn from(value: HelloQuadLit) -> Self {
        match value {
            HelloQuadLit::T => HelloIrQuadLit::T,
            HelloQuadLit::F => HelloIrQuadLit::F,
            HelloQuadLit::N => HelloIrQuadLit::N,
            HelloQuadLit::S => HelloIrQuadLit::S,
        }
    }
}
