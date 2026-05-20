use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SevenHellOutputMode {
    Human,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SevenHellSkeletonOptions {
    pub target: String,
    pub output_mode: SevenHellOutputMode,
}

const STAGES: [(usize, &str, &str); 7] = [
    (1, "Syntax Hell", "syntax"),
    (2, "Type Hell", "type"),
    (3, "Lowering Hell", "lowering"),
    (4, "Verifier Hell", "verifier"),
    (5, "VM Hell", "vm"),
    (6, "Practical Hell", "practical"),
    (7, "User Pain / Diagnostics Hell", "diagnostics"),
];

pub fn parse_7hell_args(args: &[String]) -> Result<SevenHellSkeletonOptions, String> {
    if args.is_empty() {
        return Err("usage: smc 7hell <input.sm> [--json]".to_string());
    }

    let mut output_mode = SevenHellOutputMode::Human;
    let mut target: Option<String> = None;

    for arg in args {
        match arg.as_str() {
            "--json" => output_mode = SevenHellOutputMode::Json,
            "--help" | "-h" => return Err("usage: smc 7hell <input.sm> [--json]".to_string()),
            value if value.starts_with('-') => return Err(format!("unknown flag '{}'", value)),
            value => {
                if target.is_some() {
                    return Err("usage: smc 7hell <input.sm> [--json]".to_string());
                }
                target = Some(value.to_string());
            }
        }
    }

    let target = target.ok_or_else(|| "usage: smc 7hell <input.sm> [--json]".to_string())?;
    Ok(SevenHellSkeletonOptions { target, output_mode })
}

pub fn render_7hell_skeleton_report(options: &SevenHellSkeletonOptions) -> String {
    let normalized = normalize_display_path(&options.target);
    match options.output_mode {
        SevenHellOutputMode::Human => render_human_report(&normalized),
        SevenHellOutputMode::Json => render_json_report(&normalized),
    }
}

pub fn run_7hell_skeleton(args: &[String]) -> Result<(), String> {
    let options = parse_7hell_args(args)?;
    print!("{}", render_7hell_skeleton_report(&options));
    Ok(())
}

fn normalize_display_path(path: &str) -> String {
    let raw = Path::new(path).to_string_lossy().replace('\\', "/");
    if raw.is_empty() {
        ".".to_string()
    } else {
        raw
    }
}

fn render_human_report(target: &str) -> String {
    let mut out = String::new();
    out.push_str("Semantic 7hell qualification\n");
    out.push_str(&format!("target: {}\n", target));
    out.push_str("mode: single-file\n");
    out.push_str("profile: default\n\n");
    for (index, name, _) in STAGES {
        out.push_str(&format!("[{}/7] {:<29} NOT_IMPLEMENTED\n", index, name));
    }
    out.push_str("\nresult: INCOMPLETE\n");
    out.push_str("boundary: skeleton command only; no check, compile, verify, VM run, CI gate, release readiness, or CTF closure\n");
    out
}

fn render_json_report(target: &str) -> String {
    let mut stages = String::new();
    for (idx, (index, name, key)) in STAGES.iter().enumerate() {
        if idx > 0 {
            stages.push_str(",\n");
        }
        stages.push_str(&format!(
            "    {{\n      \"index\": {},\n      \"name\": \"{}\",\n      \"key\": \"{}\",\n      \"status\": \"not_implemented\",\n      \"summary\": \"skeleton stage placeholder\",\n      \"diagnostic_ids\": [],\n      \"evidence_ids\": [],\n      \"blocked_by\": null\n    }}",
            index,
            json_escape(name),
            json_escape(key)
        ));
    }

    format!(
        "{{\n  \"schema\": \"semantic.7hell.report.v0\",\n  \"tool\": \"smc 7hell\",\n  \"target\": {{\n    \"kind\": \"single-file\",\n    \"display\": \"{}\",\n    \"normalized\": \"{}\"\n  }},\n  \"profile\": \"default\",\n  \"result\": \"incomplete\",\n  \"stages\": [\n{}\n  ],\n  \"diagnostics\": [],\n  \"evidence\": [],\n  \"ctf\": [],\n  \"boundaries\": [\n    {{\n      \"id\": \"B001\",\n      \"scope\": \"7hell-skeleton\",\n      \"status\": \"skeleton-only\",\n      \"reason\": \"no check, compile, verify, VM run, CI gate, release readiness, or CTF closure\"\n    }}\n  ]\n}}\n",
        json_escape(target),
        json_escape(target),
        stages
    )
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_file_target() {
        let args = vec!["program.sm".to_string()];
        let parsed = parse_7hell_args(&args).expect("parse");
        assert_eq!(parsed.target, "program.sm");
        assert_eq!(parsed.output_mode, SevenHellOutputMode::Human);
    }

    #[test]
    fn renders_human_skeleton_boundary() {
        let options = SevenHellSkeletonOptions {
            target: "program.sm".to_string(),
            output_mode: SevenHellOutputMode::Human,
        };
        let rendered = render_7hell_skeleton_report(&options);
        assert!(rendered.contains("Semantic 7hell qualification"));
        assert!(rendered.contains("[1/7] Syntax Hell"));
        assert!(rendered.contains("[7/7] User Pain / Diagnostics Hell"));
        assert!(rendered.contains("result: INCOMPLETE"));
        assert!(rendered.contains("skeleton command only"));
        assert!(rendered.contains("no check, compile, verify, VM run, CI gate, release readiness, or CTF closure"));
        assert!(!rendered.contains("result: PASS"));
    }

    #[test]
    fn renders_json_skeleton_schema() {
        let options = SevenHellSkeletonOptions {
            target: "program.sm".to_string(),
            output_mode: SevenHellOutputMode::Json,
        };
        let rendered = render_7hell_skeleton_report(&options);
        assert!(rendered.contains("\"schema\": \"semantic.7hell.report.v0\""));
        assert!(rendered.contains("\"result\": \"incomplete\""));
        assert!(rendered.contains("\"key\": \"syntax\""));
        assert!(rendered.contains("\"key\": \"diagnostics\""));
    }
}
