use prom_abi::{AbiError, AbiFailureKind, ApplicationHostAbi, HostCallId};
use prom_cap::required_capability_for_call;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

pub(crate) const APPLICATION_AUDIT_SCHEMA: &str = "semantic.foundation.application.audit/0.1";
const MAX_APPLICATION_TEXT_BYTES: usize = 16 * 1024 * 1024;
const MAX_APPLICATION_PATH_BYTES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ApplicationAuditRecord {
    sequence: u64,
    call: HostCallId,
    byte_len: usize,
    path_hash: Option<u64>,
    payload_hash: Option<u64>,
    allowed: bool,
}

impl ApplicationAuditRecord {
    pub(crate) fn render(&self) -> String {
        format!(
            "schema={} seq={} capability={} decision={} bytes={} path_hash={} payload_hash={}",
            APPLICATION_AUDIT_SCHEMA,
            self.sequence,
            required_capability_for_call(self.call).id(),
            if self.allowed { "allow" } else { "deny" },
            self.byte_len,
            display_hash(self.path_hash),
            display_hash(self.payload_hash),
        )
    }
}

fn display_hash(value: Option<u64>) -> String {
    value
        .map(|value| format!("{value:016x}"))
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) struct CliApplicationHost {
    root: PathBuf,
    args: Vec<String>,
    duration_millis: Option<u32>,
    stdin: Option<String>,
    observed: bool,
    audit: Vec<ApplicationAuditRecord>,
}

impl CliApplicationHost {
    pub(crate) fn new(
        root: &Path,
        args: Vec<String>,
        duration_millis: Option<u32>,
    ) -> Result<Self, String> {
        let root_metadata = fs::symlink_metadata(root)
            .map_err(|error| format!("failed to inspect application root: {error}"))?;
        if is_reparse_point(&root_metadata) {
            return Err("application root must not be a symlink or reparse point".to_string());
        }
        let root = root
            .canonicalize()
            .map_err(|error| format!("failed to resolve application root: {error}"))?;
        if !root.is_dir() {
            return Err(format!(
                "application root '{}' is not a directory",
                root.display()
            ));
        }
        Ok(Self {
            root,
            args,
            duration_millis,
            stdin: None,
            observed: false,
            audit: Vec::new(),
        })
    }

    pub(crate) fn audit(&self) -> &[ApplicationAuditRecord] {
        &self.audit
    }

    pub(crate) fn record_denied(&mut self, call: HostCallId) {
        let sequence = self.audit.len() as u64;
        self.audit.push(ApplicationAuditRecord {
            sequence,
            call,
            byte_len: 0,
            path_hash: None,
            payload_hash: None,
            allowed: false,
        });
    }

    fn error(
        &self,
        call: HostCallId,
        kind: AbiFailureKind,
        message: impl Into<String>,
    ) -> AbiError {
        AbiError::new(call, kind, message)
    }

    fn record(
        &mut self,
        call: HostCallId,
        byte_len: usize,
        path_hash: Option<u64>,
        payload_hash: Option<u64>,
    ) {
        let sequence = self.audit.len() as u64;
        self.audit.push(ApplicationAuditRecord {
            sequence,
            call,
            byte_len,
            path_hash,
            payload_hash,
            allowed: true,
        });
    }

    fn mark_observation(&mut self) {
        self.observed = true;
    }

    fn require_observation(&self, call: HostCallId) -> Result<(), AbiError> {
        if self.observed {
            Ok(())
        } else {
            Err(self.error(
                call,
                AbiFailureKind::InvalidInput,
                "application writes require a preceding captured observation",
            ))
        }
    }

    fn relative_path(&self, call: HostCallId, raw: &str) -> Result<PathBuf, AbiError> {
        if raw.is_empty() {
            return Err(self.error(call, AbiFailureKind::InvalidInput, "path must not be empty"));
        }
        if raw.len() > MAX_APPLICATION_PATH_BYTES {
            return Err(self.error(
                call,
                AbiFailureKind::InvalidInput,
                "path exceeds the application boundary limit",
            ));
        }
        let path = Path::new(raw);
        if path.is_absolute()
            || path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(self.error(
                call,
                AbiFailureKind::InvalidInput,
                "path must be relative and must not contain parent traversal",
            ));
        }
        Ok(path.to_path_buf())
    }

    fn ensure_text_limit(&self, call: HostCallId, byte_len: usize) -> Result<(), AbiError> {
        if byte_len <= MAX_APPLICATION_TEXT_BYTES {
            Ok(())
        } else {
            Err(self.error(
                call,
                AbiFailureKind::InvalidInput,
                "text exceeds the application boundary limit",
            ))
        }
    }

    fn read_bounded_text<R: Read>(&self, call: HostCallId, reader: R) -> Result<String, AbiError> {
        let mut bytes = Vec::new();
        reader
            .take((MAX_APPLICATION_TEXT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| self.error(call, AbiFailureKind::HostFault, error.to_string()))?;
        self.ensure_text_limit(call, bytes.len())?;
        String::from_utf8(bytes).map_err(|_| {
            self.error(
                call,
                AbiFailureKind::InvalidInput,
                "application text must be valid UTF-8",
            )
        })
    }

    fn reject_reparse_components(&self, call: HostCallId, relative: &Path) -> Result<(), AbiError> {
        let mut current = self.root.clone();
        for component in relative.components() {
            if let Component::Normal(segment) = component {
                current.push(segment);
                match fs::symlink_metadata(&current) {
                    Ok(metadata) if is_reparse_point(&metadata) => {
                        return Err(self.error(
                            call,
                            AbiFailureKind::InvalidInput,
                            "symlink and reparse-point traversal is denied",
                        ));
                    }
                    Ok(_) => {}
                    Err(error) if error.kind() == io::ErrorKind::NotFound => break,
                    Err(error) => {
                        return Err(self.error(call, AbiFailureKind::HostFault, error.to_string()));
                    }
                }
            }
        }
        Ok(())
    }

    fn existing_path(&self, call: HostCallId, raw: &str) -> Result<PathBuf, AbiError> {
        let relative = self.relative_path(call, raw)?;
        self.reject_reparse_components(call, &relative)?;
        let resolved = self
            .root
            .join(relative)
            .canonicalize()
            .map_err(|error| self.error(call, AbiFailureKind::HostFault, error.to_string()))?;
        if !resolved.starts_with(&self.root) {
            return Err(self.error(
                call,
                AbiFailureKind::InvalidInput,
                "resolved path escapes the application root",
            ));
        }
        Ok(resolved)
    }

    fn write_path(&self, call: HostCallId, raw: &str) -> Result<PathBuf, AbiError> {
        let relative = self.relative_path(call, raw)?;
        self.reject_reparse_components(call, &relative)?;
        let target = self.root.join(&relative);
        let parent = target.parent().ok_or_else(|| {
            self.error(
                call,
                AbiFailureKind::InvalidInput,
                "output path has no parent",
            )
        })?;
        let resolved_parent = parent
            .canonicalize()
            .map_err(|error| self.error(call, AbiFailureKind::HostFault, error.to_string()))?;
        if !resolved_parent.starts_with(&self.root) {
            return Err(self.error(
                call,
                AbiFailureKind::InvalidInput,
                "resolved output parent escapes the application root",
            ));
        }
        let file_name = target.file_name().ok_or_else(|| {
            self.error(
                call,
                AbiFailureKind::InvalidInput,
                "output path has no file name",
            )
        })?;
        Ok(resolved_parent.join(file_name))
    }
}

impl ApplicationHostAbi for CliApplicationHost {
    fn args_read(&mut self, index: u32) -> Result<String, AbiError> {
        let value = self.args.get(index as usize).ok_or_else(|| {
            self.error(
                HostCallId::ArgsRead,
                AbiFailureKind::InvalidInput,
                format!("application argument {index} is unavailable"),
            )
        })?;
        self.ensure_text_limit(HostCallId::ArgsRead, value.len())?;
        let value = value.clone();
        self.mark_observation();
        self.record(
            HostCallId::ArgsRead,
            value.len(),
            None,
            Some(stable_hash(value.as_bytes())),
        );
        Ok(value)
    }

    fn stdin_read_text(&mut self) -> Result<String, AbiError> {
        if self.stdin.is_none() {
            let value = self.read_bounded_text(HostCallId::StdinReadText, io::stdin())?;
            self.stdin = Some(value);
        }
        let value = self.stdin.clone().unwrap_or_default();
        self.mark_observation();
        self.record(
            HostCallId::StdinReadText,
            value.len(),
            None,
            Some(stable_hash(value.as_bytes())),
        );
        Ok(value)
    }

    fn stdout_write(&mut self, text: &str) -> Result<(), AbiError> {
        self.require_observation(HostCallId::StdoutWrite)?;
        self.ensure_text_limit(HostCallId::StdoutWrite, text.len())?;
        io::stdout().write_all(text.as_bytes()).map_err(|error| {
            self.error(
                HostCallId::StdoutWrite,
                AbiFailureKind::HostFault,
                error.to_string(),
            )
        })?;
        self.record(
            HostCallId::StdoutWrite,
            text.len(),
            None,
            Some(stable_hash(text.as_bytes())),
        );
        Ok(())
    }

    fn stderr_write(&mut self, text: &str) -> Result<(), AbiError> {
        self.require_observation(HostCallId::StderrWrite)?;
        self.ensure_text_limit(HostCallId::StderrWrite, text.len())?;
        io::stderr().write_all(text.as_bytes()).map_err(|error| {
            self.error(
                HostCallId::StderrWrite,
                AbiFailureKind::HostFault,
                error.to_string(),
            )
        })?;
        self.record(
            HostCallId::StderrWrite,
            text.len(),
            None,
            Some(stable_hash(text.as_bytes())),
        );
        Ok(())
    }

    fn path_inspect(&mut self, path: &str) -> Result<bool, AbiError> {
        let relative = self.relative_path(HostCallId::PathInspect, path)?;
        self.reject_reparse_components(HostCallId::PathInspect, &relative)?;
        let exists = self.root.join(relative).try_exists().map_err(|error| {
            self.error(
                HostCallId::PathInspect,
                AbiFailureKind::HostFault,
                error.to_string(),
            )
        })?;
        self.mark_observation();
        self.record(
            HostCallId::PathInspect,
            1,
            Some(stable_hash(path.as_bytes())),
            Some(stable_hash(&[u8::from(exists)])),
        );
        Ok(exists)
    }

    fn fs_read_text(&mut self, path: &str) -> Result<String, AbiError> {
        let resolved = self.existing_path(HostCallId::FsRead, path)?;
        let file = fs::File::open(resolved).map_err(|error| {
            self.error(
                HostCallId::FsRead,
                AbiFailureKind::HostFault,
                error.to_string(),
            )
        })?;
        let value = self.read_bounded_text(HostCallId::FsRead, file)?;
        self.mark_observation();
        self.record(
            HostCallId::FsRead,
            value.len(),
            Some(stable_hash(path.as_bytes())),
            Some(stable_hash(value.as_bytes())),
        );
        Ok(value)
    }

    fn fs_write_text(&mut self, path: &str, text: &str) -> Result<(), AbiError> {
        self.require_observation(HostCallId::FsWrite)?;
        self.ensure_text_limit(HostCallId::FsWrite, text.len())?;
        let resolved = self.write_path(HostCallId::FsWrite, path)?;
        fs::write(resolved, text).map_err(|error| {
            self.error(
                HostCallId::FsWrite,
                AbiFailureKind::HostFault,
                error.to_string(),
            )
        })?;
        self.record(
            HostCallId::FsWrite,
            text.len(),
            Some(stable_hash(path.as_bytes())),
            Some(stable_hash(text.as_bytes())),
        );
        Ok(())
    }

    fn time_duration_millis(&mut self) -> Result<u32, AbiError> {
        let value = self.duration_millis.ok_or_else(|| {
            self.error(
                HostCallId::TimeDuration,
                AbiFailureKind::Unavailable,
                "time.duration requires an explicit --duration-ms input",
            )
        })?;
        self.mark_observation();
        self.record(
            HostCallId::TimeDuration,
            core::mem::size_of::<u32>(),
            None,
            Some(stable_hash(&value.to_le_bytes())),
        );
        Ok(value)
    }
}

fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_traversal_is_denied() {
        let root = std::env::temp_dir();
        let host = CliApplicationHost::new(&root, Vec::new(), None).expect("host");
        let error = host
            .relative_path(HostCallId::FsRead, "../secret")
            .expect_err("deny");
        assert_eq!(error.kind, AbiFailureKind::InvalidInput);
    }

    #[test]
    fn writes_require_an_observation() {
        let root = std::env::temp_dir();
        let mut host = CliApplicationHost::new(&root, Vec::new(), None).expect("host");
        let error = host
            .fs_write_text("ssf04-no-write.txt", "x")
            .expect_err("deny");
        assert_eq!(error.call, HostCallId::FsWrite);
    }

    #[test]
    fn oversized_paths_are_denied() {
        let root = std::env::temp_dir();
        let host = CliApplicationHost::new(&root, Vec::new(), None).expect("host");
        let error = host
            .relative_path(
                HostCallId::FsRead,
                &"a".repeat(MAX_APPLICATION_PATH_BYTES + 1),
            )
            .expect_err("deny");
        assert_eq!(error.kind, AbiFailureKind::InvalidInput);
    }

    #[test]
    fn text_budget_and_malformed_utf8_fail_closed() {
        let root = std::env::temp_dir();
        let host = CliApplicationHost::new(&root, Vec::new(), None).expect("host");
        let budget_error = host
            .ensure_text_limit(HostCallId::FsRead, MAX_APPLICATION_TEXT_BYTES + 1)
            .expect_err("budget");
        assert_eq!(budget_error.kind, AbiFailureKind::InvalidInput);

        let utf8_error = host
            .read_bounded_text(HostCallId::FsRead, io::Cursor::new([0xff]))
            .expect_err("utf8");
        assert_eq!(utf8_error.kind, AbiFailureKind::InvalidInput);
    }

    #[test]
    fn denied_audit_record_is_redacted() {
        let root = std::env::temp_dir();
        let mut host = CliApplicationHost::new(&root, Vec::new(), None).expect("host");
        host.record_denied(HostCallId::FsWrite);
        let rendered = host.audit()[0].render();
        assert!(rendered.contains("capability=fs.write decision=deny"));
        assert!(rendered.contains("path_hash=- payload_hash=-"));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_root_is_denied_before_canonicalization() {
        use std::os::unix::fs::symlink;

        let target = std::env::temp_dir();
        let link = target.join(format!("semantic-ssf04-root-link-{}", std::process::id()));
        let _ = fs::remove_file(&link);
        symlink(&target, &link).expect("symlink");
        let error = CliApplicationHost::new(&link, Vec::new(), None)
            .err()
            .expect("symlink root must fail");
        assert!(error.contains("symlink or reparse point"));
        fs::remove_file(link).expect("cleanup");
    }
}
