//! `smc look ui frame` -- deterministic, read-only UI frame inspection
//! (Issue #1365).
//!
//! Two input modes, exactly one required:
//!   `smc look ui frame --from <snapshot>` -- load a canonical Frame
//!     Snapshot v0 document ([`crate::ui_frame_snapshot`]). Never compiles
//!     or runs source, never activates a bundle.
//!   `smc look ui frame <source-file> [--events <script>]` -- compile
//!     Projection Source v0 text through the real, existing UI-DNA2
//!     pipeline (`prom_ui_runtime::reference_contour`), inject a
//!     deterministic bounded event script
//!     ([`crate::ui_event_script`]), and capture frames through
//!     `InMemoryBackend`. Never opens a native window.
//!
//! Every successful report carries an explicit non-authority notice:
//! inspection output is evidence only, not live runtime state, and does
//! not authorize any action or committed effect.

use std::fmt;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use prom_ui_runtime::reference_contour::{
    dispatch_input_event, render_frame, ReferenceContourError, ReferenceState,
};
use prom_ui_runtime::{DesktopSession, InMemoryBackend, InputEvent, LoopControl, WindowConfig};

use crate::ui_event_script::{EventScriptDecodeError, EventScriptV0};
use crate::ui_frame_snapshot::{FrameSnapshotV0, SnapshotDecodeError};

/// Bound on the raw Projection Source text file `smc look ui frame` will
/// read, checked against file metadata before any read.
const MAX_SOURCE_BYTES: u64 = 1024 * 1024;

const SOURCE_PROFILE: &str = "projection-source-v0";
const NOTICE: &str =
    "inspection output only; not live runtime authority; does not authorize action or committed effect";

/// Entry point for `smc look ui frame ...`; dispatched from `app::run` as
/// `"look" => cmd_look(&args[1..])`.
pub fn cmd_look(args: &[String]) -> Result<(), String> {
    if args.first().map(String::as_str) != Some("ui")
        || args.get(1).map(String::as_str) != Some("frame")
    {
        return Err(look_usage());
    }
    cmd_look_ui_frame(&args[2..]).map_err(|outcome| outcome.into_cli_error())
}

fn look_usage() -> String {
    [
        "usage:",
        "  smc look ui frame --from <snapshot> [--frame <n>] [--format text|draw-json] [--out <path>]",
        "  smc look ui frame <source-file> [--events <script>] [--frame <n>] [--format text|draw-json] [--out <path>]",
    ]
    .join("\n")
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LookStatus {
    Captured,
    Loaded,
    Denied,
    #[allow(dead_code)]
    Failed,
    InvalidArguments,
    InvalidSource,
    CompileFailed,
    ActivationDenied,
    InvalidEventScript,
    InvalidSnapshot,
    UnsupportedVersion,
    ResourceLimitExceeded,
    FrameNotFound,
    UnsupportedFormat,
    OutputWriteFailed,
    #[allow(dead_code)]
    InternalInspectionFault,
}

#[derive(Debug)]
struct LookOutcome {
    status: LookStatus,
    message: String,
}

impl LookOutcome {
    fn new(status: LookStatus, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    fn invalid_args(message: impl Into<String>) -> Self {
        Self::new(LookStatus::InvalidArguments, message)
    }

    fn into_cli_error(self) -> String {
        format!("{:?}: {}", self.status, self.message)
    }
}

impl From<SnapshotDecodeError> for LookOutcome {
    fn from(e: SnapshotDecodeError) -> Self {
        let status = match &e {
            SnapshotDecodeError::UnsupportedVersion { .. } => LookStatus::UnsupportedVersion,
            SnapshotDecodeError::ResourceLimitExceeded(_) => LookStatus::ResourceLimitExceeded,
            SnapshotDecodeError::OversizedInput { .. } | SnapshotDecodeError::Malformed { .. } => {
                LookStatus::InvalidSnapshot
            }
        };
        LookOutcome::new(status, e.to_string())
    }
}

impl From<EventScriptDecodeError> for LookOutcome {
    fn from(e: EventScriptDecodeError) -> Self {
        let status = match &e {
            EventScriptDecodeError::UnsupportedVersion { .. } => LookStatus::UnsupportedVersion,
            EventScriptDecodeError::ResourceLimitExceeded(_) => LookStatus::ResourceLimitExceeded,
            _ => LookStatus::InvalidEventScript,
        };
        LookOutcome::new(status, e.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    DrawJson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Source,
    Snapshot,
}

#[derive(Debug)]
struct ParsedArgs {
    from: Option<String>,
    source: Option<String>,
    events: Option<String>,
    frame: usize,
    format: OutputFormat,
    out: Option<String>,
}

fn next_value(args: &[String], i: &mut usize, flag: &str) -> Result<String, LookOutcome> {
    let value = args
        .get(*i + 1)
        .ok_or_else(|| LookOutcome::invalid_args(format!("missing value after '{flag}'")))?;
    *i += 2;
    Ok(value.clone())
}

fn parse_args(args: &[String]) -> Result<ParsedArgs, LookOutcome> {
    let mut from = None;
    let mut source = None;
    let mut events = None;
    let mut frame = 0usize;
    let mut format = OutputFormat::Text;
    let mut out = None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--from" => {
                if from.is_some() {
                    return Err(LookOutcome::invalid_args("duplicate '--from'"));
                }
                from = Some(next_value(args, &mut i, "--from")?);
            }
            "--events" => {
                if events.is_some() {
                    return Err(LookOutcome::invalid_args("duplicate '--events'"));
                }
                events = Some(next_value(args, &mut i, "--events")?);
            }
            "--frame" => {
                let v = next_value(args, &mut i, "--frame")?;
                frame = v.parse::<usize>().map_err(|_| {
                    LookOutcome::invalid_args(format!(
                        "--frame expects a non-negative integer, got '{v}'"
                    ))
                })?;
            }
            "--format" => {
                let v = next_value(args, &mut i, "--format")?;
                format = match v.as_str() {
                    "text" => OutputFormat::Text,
                    "draw-json" => OutputFormat::DrawJson,
                    other => {
                        return Err(LookOutcome::new(
                            LookStatus::UnsupportedFormat,
                            format!(
                                "unsupported --format '{other}'; expected 'text' or 'draw-json'"
                            ),
                        ));
                    }
                };
            }
            "--out" => {
                if out.is_some() {
                    return Err(LookOutcome::invalid_args("duplicate '--out'"));
                }
                out = Some(next_value(args, &mut i, "--out")?);
            }
            other if other.starts_with("--") => {
                return Err(LookOutcome::invalid_args(format!("unknown flag '{other}'")));
            }
            other => {
                if source.is_some() {
                    return Err(LookOutcome::invalid_args(
                        "duplicate positional source input; only one source file is accepted",
                    ));
                }
                source = Some(other.to_string());
                i += 1;
            }
        }
    }

    match (&from, &source) {
        (None, None) => {
            return Err(LookOutcome::invalid_args(
                "exactly one input is required: --from <snapshot> or a source file",
            ));
        }
        (Some(_), Some(_)) => {
            return Err(LookOutcome::invalid_args(
                "--from and a source file are mutually exclusive; exactly one input is allowed",
            ));
        }
        _ => {}
    }
    if from.is_some() && events.is_some() {
        return Err(LookOutcome::invalid_args(
            "--events is only valid in source mode, not together with --from",
        ));
    }

    Ok(ParsedArgs {
        from,
        source,
        events,
        frame,
        format,
        out,
    })
}

fn read_bounded_file(path: &str, max_bytes: u64) -> Result<Vec<u8>, String> {
    let meta = fs::metadata(path).map_err(|e| format!("failed to stat '{path}': {e}"))?;
    if meta.len() > max_bytes {
        return Err(format!(
            "'{path}' is {} bytes, exceeds the maximum of {max_bytes} bytes",
            meta.len()
        ));
    }
    fs::read(path).map_err(|e| format!("failed to read '{path}': {e}"))
}

struct CaptureResult {
    mode: Mode,
    input_path: String,
    source_profile: Option<&'static str>,
    events_path: Option<String>,
    snapshot: FrameSnapshotV0,
    /// `denied_by_frame[i]` is `Some(bool)` for source mode, recording
    /// whether the interaction most recently processed as of captured
    /// frame `i` was denied; `None` in snapshot mode, where there is no
    /// admission concept at all. Indexed identically to `snapshot.frames`.
    denied_by_frame: Option<Vec<bool>>,
}

fn run_snapshot_mode(from_path: &str) -> Result<CaptureResult, LookOutcome> {
    let bytes = read_bounded_file(from_path, crate::ui_frame_snapshot::MAX_SNAPSHOT_BYTES)
        .map_err(|msg| LookOutcome::new(LookStatus::InvalidSnapshot, msg))?;
    let snapshot = FrameSnapshotV0::decode_canonical(&bytes)?;
    Ok(CaptureResult {
        mode: Mode::Snapshot,
        input_path: from_path.to_string(),
        source_profile: None,
        events_path: None,
        snapshot,
        denied_by_frame: None,
    })
}

fn run_source_mode(
    source_path: &str,
    events_path: Option<&str>,
) -> Result<CaptureResult, LookOutcome> {
    let source_bytes = read_bounded_file(source_path, MAX_SOURCE_BYTES)
        .map_err(|msg| LookOutcome::new(LookStatus::InvalidSource, msg))?;
    let source_text = std::str::from_utf8(&source_bytes).map_err(|_| {
        LookOutcome::new(LookStatus::InvalidSource, "source file is not valid UTF-8")
    })?;

    let event_script = match events_path {
        Some(p) => {
            let bytes = read_bounded_file(p, crate::ui_event_script::MAX_EVENT_SCRIPT_BYTES)
                .map_err(|msg| LookOutcome::new(LookStatus::InvalidEventScript, msg))?;
            EventScriptV0::decode_canonical(&bytes)?
        }
        None => EventScriptV0::default(),
    };

    let mut state = ReferenceState::new(source_text).map_err(|e| match e {
        ReferenceContourError::CompileFailed(inner) => {
            LookOutcome::new(LookStatus::CompileFailed, format!("{inner:?}"))
        }
        ReferenceContourError::ActivationDenied(inner) => {
            LookOutcome::new(LookStatus::ActivationDenied, format!("{inner:?}"))
        }
    })?;

    let config = WindowConfig::new("smc-look-ui-frame", 640, 480);
    let backend = InMemoryBackend::new();
    let mut session = DesktopSession::create(backend, config)
        .map_err(|e| LookOutcome::new(LookStatus::InternalInspectionFault, format!("{e:?}")))?;

    // One harmless priming iteration to reach `SessionState::Running`: this
    // is the only public seam that performs that transition. It captures no
    // frame -- neither `DesktopSession::run`'s wrapper nor
    // `InMemoryBackend::run_event_loop` calls `draw_frame` during this call.
    session
        .run(|_buf, _frame| LoopControl::ExitRequested)
        .map_err(|e| LookOutcome::new(LookStatus::InternalInspectionFault, format!("{e:?}")))?;

    // `denied_by_frame[i]` records whether the interaction most recently
    // processed as of captured frame `i` was denied -- so a report about a
    // specific `--frame` describes that frame's own state, not whatever
    // the script did in later steps.
    let mut denied_by_frame: Vec<bool> = Vec::new();

    // Frame 0: the initial state, before any scripted event is delivered.
    session
        .tick_in_memory_frame(|_events, out_frame| {
            *out_frame = render_frame(&state);
            LoopControl::Continue
        })
        .map_err(|e| LookOutcome::new(LookStatus::InternalInspectionFault, format!("{e:?}")))?;
    denied_by_frame.push(state.last_denied());

    for step in &event_script.steps {
        let queued: Vec<InputEvent> = step
            .events
            .iter()
            .map(|e| InputEvent::new(e.to_input_event_kind()))
            .collect();
        session.backend_mut().extend_events(queued);

        session
            .tick_in_memory_frame(|events, out_frame| {
                for event in events {
                    // Headless capture processes the full, finite script
                    // regardless of a `close`/Escape event's `ExitRequested`
                    // signal -- there is no window to close, and a fixed,
                    // input-independent captured-frame count keeps the
                    // frame-step model simple and deterministic.
                    let _ = dispatch_input_event(&mut state, &event.kind);
                }
                *out_frame = render_frame(&state);
                LoopControl::Continue
            })
            .map_err(|e| LookOutcome::new(LookStatus::InternalInspectionFault, format!("{e:?}")))?;
        denied_by_frame.push(state.last_denied());
    }

    let captured = session.backend().captured_frames();
    let snapshot = FrameSnapshotV0::from_captured_frames(captured.iter().map(|v| v.as_slice()))
        .map_err(|e| LookOutcome::new(LookStatus::ResourceLimitExceeded, e.to_string()))?;

    Ok(CaptureResult {
        mode: Mode::Source,
        input_path: source_path.to_string(),
        source_profile: Some(SOURCE_PROFILE),
        events_path: events_path.map(|p| p.to_string()),
        snapshot,
        denied_by_frame: Some(denied_by_frame),
    })
}

struct Report {
    result: CaptureResult,
    requested_frame: usize,
    status: LookStatus,
}

fn cmd_look_ui_frame(args: &[String]) -> Result<(), LookOutcome> {
    let parsed = parse_args(args)?;

    let result = match (&parsed.from, &parsed.source) {
        (Some(from), None) => run_snapshot_mode(from)?,
        (None, Some(source)) => run_source_mode(source, parsed.events.as_deref())?,
        _ => unreachable!("parse_args already enforces exactly one input mode"),
    };

    if parsed.frame >= result.snapshot.frames.len() {
        return Err(LookOutcome::new(
            LookStatus::FrameNotFound,
            format!(
                "requested frame {} not found; {} frame(s) captured",
                parsed.frame,
                result.snapshot.frames.len()
            ),
        ));
    }

    let denied_at_selected_frame = result.denied_by_frame.as_ref().map(|v| v[parsed.frame]);
    let status = match result.mode {
        Mode::Snapshot => LookStatus::Loaded,
        Mode::Source if denied_at_selected_frame == Some(true) => LookStatus::Denied,
        Mode::Source => LookStatus::Captured,
    };

    let report = Report {
        result,
        requested_frame: parsed.frame,
        status,
    };

    let rendered = match parsed.format {
        OutputFormat::Text => render_text(&report),
        OutputFormat::DrawJson => report.result.snapshot.encode_canonical(),
    };

    match &parsed.out {
        Some(path) => write_output_atomic(path, &rendered)
            .map_err(|msg| LookOutcome::new(LookStatus::OutputWriteFailed, msg))?,
        None => print!("{rendered}"),
    }

    Ok(())
}

fn render_text(report: &Report) -> String {
    let mut lines = Vec::new();
    lines.push("smc look ui frame v0".to_string());
    lines.push(format!(
        "mode: {}",
        match report.result.mode {
            Mode::Source => "source",
            Mode::Snapshot => "snapshot",
        }
    ));
    lines.push(format!("input: {}", report.result.input_path));
    lines.push(format!(
        "source-profile: {}",
        report
            .result
            .source_profile
            .unwrap_or("n/a (snapshot mode)")
    ));
    lines.push(format!(
        "events: {}",
        report.result.events_path.as_deref().unwrap_or("none")
    ));
    lines.push(format!("requested-frame: {}", report.requested_frame));
    lines.push(format!(
        "captured-frames: {}",
        report.result.snapshot.frames.len()
    ));
    lines.push(format!("selected-frame: {}", report.requested_frame));
    lines.push(format!("status: {:?}", report.status));
    lines.push(format!(
        "admission: {}",
        match report.result.denied_by_frame.as_ref() {
            Some(v) => format!("last-denied={}", v[report.requested_frame]),
            None => "n/a (snapshot mode)".to_string(),
        }
    ));

    let commands = &report.result.snapshot.frames[report.requested_frame].commands;
    lines.push(format!("draw-commands: {}", commands.len()));
    for (index, cmd) in commands.iter().enumerate() {
        lines.push(format!("  {index}: {}", format_command(cmd)));
    }

    lines.push("warnings: none".to_string());
    lines.push(format!("NOTICE: {NOTICE}"));

    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn format_command(cmd: &crate::ui_frame_snapshot::DrawCommandV0) -> String {
    use crate::ui_frame_snapshot::DrawCommandV0;
    match cmd {
        DrawCommandV0::Clear { color } => {
            format!(
                "clear color=({},{},{},{})",
                color.r, color.g, color.b, color.a
            )
        }
        DrawCommandV0::FillRect { rect, color } => format!(
            "fill_rect rect=(x={},y={},w={},h={}) color=({},{},{},{})",
            rect.x, rect.y, rect.width, rect.height, color.r, color.g, color.b, color.a
        ),
        DrawCommandV0::DrawText { text, x, y, color } => format!(
            "draw_text {text:?} at ({x},{y}) color=({},{},{},{})",
            color.r, color.g, color.b, color.a
        ),
    }
}

fn write_output_atomic(path: &str, contents: &str) -> Result<(), String> {
    let dest = Path::new(path);
    let dir = match dest.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let file_name = dest
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("smc-look-ui-frame-out");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp_path = dir.join(format!(
        ".{file_name}.tmp.{}.{}",
        std::process::id(),
        suffix
    ));

    let write_result = (|| {
        use std::io::Write;
        let mut f = fs::File::create(&tmp_path)?;
        f.write_all(contents.as_bytes())?;
        f.sync_all()
    })();
    if let Err(e) = write_result {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!(
            "failed to write temporary file '{}': {e}",
            tmp_path.display()
        ));
    }

    if let Err(e) = fs::rename(&tmp_path, dest) {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!(
            "failed to atomically replace '{}': {e}",
            dest.display()
        ));
    }
    Ok(())
}

impl fmt::Debug for LookStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            LookStatus::Captured => "Captured",
            LookStatus::Loaded => "Loaded",
            LookStatus::Denied => "Denied",
            LookStatus::Failed => "Failed",
            LookStatus::InvalidArguments => "InvalidArguments",
            LookStatus::InvalidSource => "InvalidSource",
            LookStatus::CompileFailed => "CompileFailed",
            LookStatus::ActivationDenied => "ActivationDenied",
            LookStatus::InvalidEventScript => "InvalidEventScript",
            LookStatus::InvalidSnapshot => "InvalidSnapshot",
            LookStatus::UnsupportedVersion => "UnsupportedVersion",
            LookStatus::ResourceLimitExceeded => "ResourceLimitExceeded",
            LookStatus::FrameNotFound => "FrameNotFound",
            LookStatus::UnsupportedFormat => "UnsupportedFormat",
            LookStatus::OutputWriteFailed => "OutputWriteFailed",
            LookStatus::InternalInspectionFault => "InternalInspectionFault",
        };
        f.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_requires_exactly_one_input_mode() {
        assert_eq!(
            parse_args(&[]).unwrap_err().status,
            LookStatus::InvalidArguments
        );
        assert_eq!(
            parse_args(&["--from".to_string(), "a".to_string(), "b".to_string()])
                .unwrap_err()
                .status,
            LookStatus::InvalidArguments
        );
    }

    #[test]
    fn parse_args_rejects_events_with_from() {
        let args = vec![
            "--from".to_string(),
            "snap.json".to_string(),
            "--events".to_string(),
            "e.json".to_string(),
        ];
        assert_eq!(
            parse_args(&args).unwrap_err().status,
            LookStatus::InvalidArguments
        );
    }

    #[test]
    fn parse_args_rejects_unsupported_format() {
        let args = vec![
            "source.txt".to_string(),
            "--format".to_string(),
            "png".to_string(),
        ];
        assert_eq!(
            parse_args(&args).unwrap_err().status,
            LookStatus::UnsupportedFormat
        );
    }

    #[test]
    fn parse_args_accepts_source_plus_events_plus_frame() {
        let args = vec![
            "source.txt".to_string(),
            "--events".to_string(),
            "e.json".to_string(),
            "--frame".to_string(),
            "2".to_string(),
        ];
        let parsed = parse_args(&args).expect("parses");
        assert_eq!(parsed.source.as_deref(), Some("source.txt"));
        assert_eq!(parsed.events.as_deref(), Some("e.json"));
        assert_eq!(parsed.frame, 2);
    }

    #[test]
    fn cmd_look_rejects_unknown_subcommand_shape() {
        assert!(cmd_look(&["ui".to_string()]).is_err());
        assert!(cmd_look(&["frame".to_string()]).is_err());
        assert!(cmd_look(&[]).is_err());
    }
}
