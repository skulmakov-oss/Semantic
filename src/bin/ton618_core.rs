//! Retained non-owning TON618 compatibility CLI shim.
//!
//! Canonical public CLI ownership lives in `smc-cli`.
//! This binary remains only as part of the retained compatibility perimeter for
//! pre-v1 `ton618_core` workflows.

pub use semantic_language::{QuadroReg, F, LSB_MASK, MSB_MASK, N, S, T};
use serde::Deserialize;
use sm_profile::{train_profile_in_place, ParserProfile, TrainingSample};
use std::collections::HashMap;
use std::env;
use std::process::ExitCode;

use language::{
    compile_human_program, execute_machine_program, parse_machine_program, render_machine_program,
};

#[derive(Debug, Deserialize)]
struct JsonSample {
    input: String,
    target: String,
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<ExitCode, String> {
    if args.is_empty() {
        println!("{}", usage());
        return Ok(ExitCode::SUCCESS);
    }

    match args[0].as_str() {
        "help" | "--help" | "-h" => {
            println!("{}", usage());
            Ok(ExitCode::SUCCESS)
        }
        "profile" => {
            run_profile(&args[1..])?;
            Ok(ExitCode::SUCCESS)
        }
        "lang" => {
            run_lang(&args[1..])?;
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown command '{}'\n\n{}", other, usage())),
    }
}

fn run_profile(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err(profile_usage());
    }

    match args[0].as_str() {
        "train" => cmd_profile_train(&args[1..]),
        "save" => cmd_profile_save(&args[1..]),
        "load" => cmd_profile_load(&args[1..]),
        other => Err(format!(
            "unknown profile subcommand '{}'\n\n{}",
            other,
            profile_usage()
        )),
    }
}

fn cmd_profile_train(args: &[String]) -> Result<(), String> {
    let mut samples_path: Option<&str> = None;
    let mut out_path: Option<&str> = None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--samples" => {
                i += 1;
                samples_path = Some(req_arg(args, i, "--samples")?);
            }
            "--out" => {
                i += 1;
                out_path = Some(req_arg(args, i, "--out")?);
            }
            other => return Err(format!("unknown flag '{}'\n\n{}", other, train_usage())),
        }
        i += 1;
    }

    let samples_path = samples_path.ok_or_else(train_usage)?;
    let out_path = out_path.ok_or_else(train_usage)?;

    let raw = std::fs::read_to_string(samples_path)
        .map_err(|e| format!("failed to read samples '{}': {}", samples_path, e))?;
    let samples_json: Vec<JsonSample> = serde_json::from_str(&raw)
        .map_err(|e| format!("invalid samples JSON '{}': {}", samples_path, e))?;

    let mut profile = ParserProfile::default();
    let borrowed: Vec<TrainingSample<'_>> = samples_json
        .iter()
        .map(|s| TrainingSample {
            input: &s.input,
            target: &s.target,
        })
        .collect();
    train_profile_in_place(&mut profile, &borrowed);
    profile
        .save_to_file(out_path)
        .map_err(|e| format!("failed to save profile '{}': {}", out_path, e))?;

    println!(
        "trained profile saved to '{}' ({} aliases)",
        out_path,
        profile.aliases.len()
    );
    Ok(())
}

fn cmd_profile_save(args: &[String]) -> Result<(), String> {
    let mut out_path: Option<&str> = None;
    let mut aliases: Vec<(String, String)> = Vec::new();

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                i += 1;
                out_path = Some(req_arg(args, i, "--out")?);
            }
            "--alias" => {
                i += 1;
                let raw = req_arg(args, i, "--alias")?;
                aliases.push(parse_alias_pair(raw)?);
            }
            other => return Err(format!("unknown flag '{}'\n\n{}", other, save_usage())),
        }
        i += 1;
    }

    let out_path = out_path.ok_or_else(save_usage)?;
    let mut profile = ParserProfile::default();
    for (raw, canonical) in aliases {
        profile.add_alias(raw, canonical);
    }

    profile
        .save_to_file(out_path)
        .map_err(|e| format!("failed to save profile '{}': {}", out_path, e))?;
    println!(
        "profile saved to '{}' ({} aliases)",
        out_path,
        profile.aliases.len()
    );
    Ok(())
}

fn cmd_profile_load(args: &[String]) -> Result<(), String> {
    let mut in_path: Option<&str> = None;
    let mut as_json = false;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--in" => {
                i += 1;
                in_path = Some(req_arg(args, i, "--in")?);
            }
            "--json" => as_json = true,
            other => return Err(format!("unknown flag '{}'\n\n{}", other, load_usage())),
        }
        i += 1;
    }

    let in_path = in_path.ok_or_else(load_usage)?;
    let profile = ParserProfile::load_from_file(in_path)
        .map_err(|e| format!("failed to load profile '{}': {}", in_path, e))?;

    if as_json {
        let json = profile
            .to_json()
            .map_err(|e| format!("failed to encode profile as JSON: {}", e))?;
        println!("{json}");
    } else {
        println!("profile '{}' aliases: {}", in_path, profile.aliases.len());
        let mut keys: Vec<_> = profile.aliases.keys().collect();
        keys.sort_unstable();
        for key in keys {
            if let Some(val) = profile.aliases.get(key) {
                println!("{key} => {val}");
            }
        }
    }

    Ok(())
}

fn run_lang(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err(lang_usage());
    }

    match args[0].as_str() {
        "compile" => cmd_lang_compile(&args[1..]),
        "run-human" => cmd_lang_run_human(&args[1..]),
        "run-machine" => cmd_lang_run_machine(&args[1..]),
        other => Err(format!(
            "unknown lang subcommand '{}'\n\n{}",
            other,
            lang_usage()
        )),
    }
}

fn cmd_lang_compile(args: &[String]) -> Result<(), String> {
    let mut in_path: Option<&str> = None;
    let mut out_path: Option<&str> = None;
    let mut profile_path: Option<&str> = None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--in" => {
                i += 1;
                in_path = Some(req_arg(args, i, "--in")?);
            }
            "--out" => {
                i += 1;
                out_path = Some(req_arg(args, i, "--out")?);
            }
            "--profile" => {
                i += 1;
                profile_path = Some(req_arg(args, i, "--profile")?);
            }
            other => {
                return Err(format!(
                    "unknown flag '{}'\n\n{}",
                    other,
                    lang_compile_usage()
                ))
            }
        }
        i += 1;
    }

    let in_path = in_path.ok_or_else(lang_compile_usage)?;
    let out_path = out_path.ok_or_else(lang_compile_usage)?;
    let human = std::fs::read_to_string(in_path)
        .map_err(|e| format!("failed to read input '{}': {}", in_path, e))?;
    let profile = load_profile_opt(profile_path)?;
    let program = compile_human_program(&human, profile.as_ref())
        .map_err(|e| format!("failed to compile human program: {}", e))?;
    let machine = render_machine_program(&program);
    std::fs::write(out_path, machine)
        .map_err(|e| format!("failed to write machine output '{}': {}", out_path, e))?;
    println!(
        "compiled '{}' -> '{}' ({} instructions)",
        in_path,
        out_path,
        program.instructions.len()
    );
    Ok(())
}

fn cmd_lang_run_human(args: &[String]) -> Result<(), String> {
    let mut in_path: Option<&str> = None;
    let mut profile_path: Option<&str> = None;
    let mut seeds: Vec<(String, u8)> = Vec::new();

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--in" => {
                i += 1;
                in_path = Some(req_arg(args, i, "--in")?);
            }
            "--profile" => {
                i += 1;
                profile_path = Some(req_arg(args, i, "--profile")?);
            }
            "--set" => {
                i += 1;
                let raw = req_arg(args, i, "--set")?;
                seeds.push(parse_seed_pair(raw)?);
            }
            other => {
                return Err(format!(
                    "unknown flag '{}'\n\n{}",
                    other,
                    lang_run_human_usage()
                ))
            }
        }
        i += 1;
    }

    let in_path = in_path.ok_or_else(lang_run_human_usage)?;
    let human = std::fs::read_to_string(in_path)
        .map_err(|e| format!("failed to read input '{}': {}", in_path, e))?;
    let profile = load_profile_opt(profile_path)?;
    let program = compile_human_program(&human, profile.as_ref())
        .map_err(|e| format!("failed to compile human program: {}", e))?;

    let mut env = HashMap::<String, QuadroReg>::new();
    seed_env(&mut env, &seeds);
    execute_machine_program(&program, &mut env)
        .map_err(|e| format!("failed to execute machine program: {}", e))?;
    print_env(&env);
    Ok(())
}

fn cmd_lang_run_machine(args: &[String]) -> Result<(), String> {
    let mut in_path: Option<&str> = None;
    let mut seeds: Vec<(String, u8)> = Vec::new();

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--in" => {
                i += 1;
                in_path = Some(req_arg(args, i, "--in")?);
            }
            "--set" => {
                i += 1;
                let raw = req_arg(args, i, "--set")?;
                seeds.push(parse_seed_pair(raw)?);
            }
            other => {
                return Err(format!(
                    "unknown flag '{}'\n\n{}",
                    other,
                    lang_run_machine_usage()
                ))
            }
        }
        i += 1;
    }

    let in_path = in_path.ok_or_else(lang_run_machine_usage)?;
    let machine_src = std::fs::read_to_string(in_path)
        .map_err(|e| format!("failed to read machine input '{}': {}", in_path, e))?;
    let program = parse_machine_program(&machine_src)
        .map_err(|e| format!("failed to parse machine program: {}", e))?;
    let mut env = HashMap::<String, QuadroReg>::new();
    seed_env(&mut env, &seeds);
    execute_machine_program(&program, &mut env)
        .map_err(|e| format!("failed to execute machine program: {}", e))?;
    print_env(&env);
    Ok(())
}

fn load_profile_opt(path: Option<&str>) -> Result<Option<ParserProfile>, String> {
    match path {
        Some(p) => ParserProfile::load_from_file(p)
            .map(Some)
            .map_err(|e| format!("failed to load profile '{}': {}", p, e)),
        None => Ok(None),
    }
}

fn seed_env(env: &mut HashMap<String, QuadroReg>, seeds: &[(String, u8)]) {
    for (name, state) in seeds {
        env.insert(name.clone(), state_to_reg(*state));
    }
}

fn state_to_reg(state: u8) -> QuadroReg {
    match state {
        N => QuadroReg::from_raw(0),
        F => QuadroReg::from_raw(semantic_language::LSB_MASK),
        T => QuadroReg::from_raw(semantic_language::MSB_MASK),
        S => QuadroReg::from_raw(u64::MAX),
        _ => QuadroReg::from_raw(0),
    }
}

fn parse_seed_pair(raw: &str) -> Result<(String, u8), String> {
    let (name, state) = raw
        .split_once('=')
        .ok_or_else(|| format!("invalid --set '{}', expected name=N|F|T|S", raw))?;
    if name.is_empty() {
        return Err(format!("invalid --set '{}', empty variable name", raw));
    }
    let state = match state {
        "N" => N,
        "F" => F,
        "T" => T,
        "S" => S,
        _ => {
            return Err(format!(
                "invalid state in --set '{}', expected N|F|T|S",
                raw
            ))
        }
    };
    Ok((name.to_string(), state))
}

fn print_env(env: &HashMap<String, QuadroReg>) {
    let mut keys: Vec<_> = env.keys().collect();
    keys.sort_unstable();
    for key in keys {
        if let Some(v) = env.get(key) {
            println!("{} = {:#018x}", key, v.raw());
        }
    }
}

fn req_arg<'a>(args: &'a [String], idx: usize, flag: &str) -> Result<&'a str, String> {
    args.get(idx)
        .map(|s| s.as_str())
        .ok_or_else(|| format!("missing value for {}\n\n{}", flag, usage()))
}

fn parse_alias_pair(raw: &str) -> Result<(String, String), String> {
    let (lhs, rhs) = raw
        .split_once('=')
        .ok_or_else(|| format!("invalid alias '{}', expected RAW=CANONICAL", raw))?;
    if lhs.is_empty() || rhs.is_empty() {
        return Err(format!("invalid alias '{}', expected RAW=CANONICAL", raw));
    }
    Ok((lhs.to_string(), rhs.to_string()))
}

fn usage() -> String {
    let mut out = String::new();
    out.push_str("Usage:\n");
    out.push_str("  ton618_core help\n");
    out.push_str("  ton618_core profile <train|save|load> [options]\n");
    out.push_str("  ton618_core lang <compile|run-human|run-machine> [options]\n");
    out.push_str("\nQuick Start:\n");
    out.push_str(
		"  ton618_core profile save --out profile.json --alias \"AND=&\" --alias \"OR=|\" --alias \"NOT=!\" --alias \"TRUE=T\" --alias \"FALSE=F\"\n",
	);
    out.push_str(
        "  ton618_core lang compile --in human.sm --out machine.sem --profile profile.json\n",
    );
    out.push_str("  ton618_core lang run-machine --in machine.sem\n\n");
    out.push_str(&profile_usage());
    out.push('\n');
    out.push_str(&lang_usage());
    out
}

fn profile_usage() -> String {
    format!("{}\n{}\n{}", train_usage(), save_usage(), load_usage())
}

fn train_usage() -> String {
    "  ton618_core profile train --samples <samples.json> --out <profile.json>\n  samples.json format: [{\"input\":\"out = a AND b\", \"target\":\"out = a & b\"}]".to_string()
}

fn save_usage() -> String {
    "  ton618_core profile save --out <profile.json> --alias RAW=CANONICAL [--alias RAW=CANONICAL ...]".to_string()
}

fn load_usage() -> String {
    "  ton618_core profile load --in <profile.json> [--json]".to_string()
}

fn lang_usage() -> String {
    format!(
        "{}\n{}\n{}",
        lang_compile_usage(),
        lang_run_human_usage(),
        lang_run_machine_usage()
    )
}

fn lang_compile_usage() -> String {
    "  ton618_core lang compile --in <human.sm> --out <machine.sem> [--profile <profile.json>]"
        .to_string()
}

fn lang_run_human_usage() -> String {
    "  ton618_core lang run-human --in <human.sm> [--profile <profile.json>] [--set var=N|F|T|S ...]".to_string()
}

fn lang_run_machine_usage() -> String {
    "  ton618_core lang run-machine --in <machine.sem> [--set var=N|F|T|S ...]".to_string()
}

mod parser {
    //! Compatibility-only parser/profile helpers for the retained TON618 CLI shim.

    #![allow(dead_code)]

    use crate::{QuadroReg, F, N, S, T};
    use sm_profile::ParserProfile;
    #[cfg(test)]
    use sm_profile::{train_profile, TrainingSample};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum UnaryOp {
        Not,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BinaryOp {
        Or,
        Xor,
        And,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Expr<'a> {
        State(u8),
        Ident(&'a str),
        Unary {
            op: UnaryOp,
            expr: Box<Expr<'a>>,
        },
        Binary {
            op: BinaryOp,
            left: Box<Expr<'a>>,
            right: Box<Expr<'a>>,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Assignment<'a> {
        pub target: &'a str,
        pub expr: Expr<'a>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ParseError {
        pub position: usize,
        pub message: &'static str,
    }

    impl core::fmt::Display for ParseError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "parse error at {}: {}", self.position, self.message)
        }
    }

    #[cfg(feature = "std")]
    impl std::error::Error for ParseError {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct EvalError<'a> {
        pub ident: &'a str,
        pub message: &'static str,
    }

    impl<'a> core::fmt::Display for EvalError<'a> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "eval error for '{}': {}", self.ident, self.message)
        }
    }

    #[cfg(feature = "std")]
    impl<'a> std::error::Error for EvalError<'a> {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ProgramError {
        pub line: usize,
        pub column: usize,
        pub message: String,
    }

    impl core::fmt::Display for ProgramError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(
                f,
                "program error at line {}, column {}: {}",
                self.line, self.column, self.message
            )
        }
    }

    #[cfg(feature = "std")]
    impl std::error::Error for ProgramError {}

    pub fn parse_expression(input: &str) -> Result<Expr<'_>, ParseError> {
        let mut p = Parser::new(input);
        let expr = p.parse_expr()?;
        p.skip_ws();
        if p.is_eof() {
            Ok(expr)
        } else {
            Err(p.err("unexpected trailing input"))
        }
    }

    pub fn parse_assignment(input: &str) -> Result<Assignment<'_>, ParseError> {
        let mut p = Parser::new(input);
        let target = p.parse_identifier()?;
        p.skip_ws();
        if !p.eat_byte(b'=') {
            return Err(p.err("expected '=' after identifier"));
        }
        let expr = p.parse_expr()?;
        p.skip_ws();
        if !p.is_eof() {
            return Err(p.err("unexpected trailing input"));
        }
        Ok(Assignment { target, expr })
    }

    pub fn eval_expression<'a, R>(
        expr: &'a Expr<'a>,
        resolve_ident: &R,
    ) -> Result<QuadroReg, EvalError<'a>>
    where
        R: Fn(&'a str) -> Option<QuadroReg>,
    {
        match expr {
            Expr::State(state) => Ok(fill_state(*state)),
            Expr::Ident(name) => resolve_ident(name).ok_or(EvalError {
                ident: name,
                message: "unknown identifier",
            }),
            Expr::Unary { op, expr } => {
                let v = eval_expression(expr, resolve_ident)?;
                match op {
                    UnaryOp::Not => Ok(v.inverse()),
                }
            }
            Expr::Binary { op, left, right } => {
                let l = eval_expression(left, resolve_ident)?;
                let r = eval_expression(right, resolve_ident)?;
                match op {
                    BinaryOp::Or => Ok(l.merge(r)),
                    BinaryOp::Xor => Ok(l.raw_delta(r)),
                    BinaryOp::And => Ok(l.intersect(r)),
                }
            }
        }
    }

    pub fn eval_assignment<'a, R>(
        assignment: &'a Assignment<'a>,
        resolve_ident: &R,
    ) -> Result<(&'a str, QuadroReg), EvalError<'a>>
    where
        R: Fn(&'a str) -> Option<QuadroReg>,
    {
        let value = eval_expression(&assignment.expr, resolve_ident)?;
        Ok((assignment.target, value))
    }

    pub fn execute_program(
        program: &str,
        env: &mut HashMap<String, QuadroReg>,
    ) -> Result<(), ProgramError> {
        for (line_idx, raw_line) in program.lines().enumerate() {
            let line_no = line_idx + 1;
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }

            let assignment = parse_assignment(line).map_err(|e| ProgramError {
                line: line_no,
                column: e.position + 1,
                message: e.message.to_string(),
            })?;

            let (target, value) = eval_assignment(&assignment, &|name| env.get(name).copied())
                .map_err(|e| ProgramError {
                    line: line_no,
                    column: 1,
                    message: format!("{} ('{}')", e.message, e.ident),
                })?;

            env.insert(target.to_string(), value);
        }

        Ok(())
    }

    pub fn execute_program_with_profile(
        program: &str,
        env: &mut HashMap<String, QuadroReg>,
        profile: &ParserProfile,
    ) -> Result<(), ProgramError> {
        for (line_idx, raw_line) in program.lines().enumerate() {
            let line_no = line_idx + 1;
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }

            let normalized = profile.normalize(line);
            let assignment = parse_assignment(&normalized).map_err(|e| ProgramError {
                line: line_no,
                column: e.position + 1,
                message: e.message.to_string(),
            })?;

            let (target, value) = eval_assignment(&assignment, &|name| env.get(name).copied())
                .map_err(|e| ProgramError {
                    line: line_no,
                    column: 1,
                    message: format!("{} ('{}')", e.message, e.ident),
                })?;

            env.insert(target.to_string(), value);
        }

        Ok(())
    }

    #[inline]
    fn strip_comment(line: &str) -> &str {
        let slash = line.find("//");
        let hash = line.find('#');
        match (slash, hash) {
            (Some(a), Some(b)) => &line[..a.min(b)],
            (Some(a), None) => &line[..a],
            (None, Some(b)) => &line[..b],
            (None, None) => line,
        }
    }

    #[inline]
    fn fill_state(state: u8) -> QuadroReg {
        debug_assert!(state <= 0b11);
        match state {
            N => QuadroReg::from_raw(0),
            F => QuadroReg::from_raw(crate::LSB_MASK),
            T => QuadroReg::from_raw(crate::MSB_MASK),
            S => QuadroReg::from_raw(u64::MAX),
            _ => unreachable!("invalid quadit state"),
        }
    }

    struct Parser<'a> {
        src: &'a str,
        bytes: &'a [u8],
        pos: usize,
    }

    impl<'a> Parser<'a> {
        fn new(src: &'a str) -> Self {
            Self {
                src,
                bytes: src.as_bytes(),
                pos: 0,
            }
        }

        fn is_eof(&self) -> bool {
            self.pos >= self.bytes.len()
        }

        fn err(&self, message: &'static str) -> ParseError {
            ParseError {
                position: self.pos,
                message,
            }
        }

        fn peek(&self) -> Option<u8> {
            self.bytes.get(self.pos).copied()
        }

        fn bump(&mut self) -> Option<u8> {
            let ch = self.peek()?;
            self.pos += 1;
            Some(ch)
        }

        fn eat_byte(&mut self, expected: u8) -> bool {
            if self.peek() == Some(expected) {
                self.pos += 1;
                true
            } else {
                false
            }
        }

        fn skip_ws(&mut self) {
            while let Some(ch) = self.peek() {
                if ch.is_ascii_whitespace() {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }

        fn parse_expr(&mut self) -> Result<Expr<'a>, ParseError> {
            self.parse_or()
        }

        fn parse_or(&mut self) -> Result<Expr<'a>, ParseError> {
            let mut left = self.parse_xor()?;
            loop {
                self.skip_ws();
                if !self.eat_byte(b'|') {
                    break;
                }
                let right = self.parse_xor()?;
                left = Expr::Binary {
                    op: BinaryOp::Or,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            }
            Ok(left)
        }

        fn parse_xor(&mut self) -> Result<Expr<'a>, ParseError> {
            let mut left = self.parse_and()?;
            loop {
                self.skip_ws();
                if !self.eat_byte(b'^') {
                    break;
                }
                let right = self.parse_and()?;
                left = Expr::Binary {
                    op: BinaryOp::Xor,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            }
            Ok(left)
        }

        fn parse_and(&mut self) -> Result<Expr<'a>, ParseError> {
            let mut left = self.parse_unary()?;
            loop {
                self.skip_ws();
                if !self.eat_byte(b'&') {
                    break;
                }
                let right = self.parse_unary()?;
                left = Expr::Binary {
                    op: BinaryOp::And,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            }
            Ok(left)
        }

        fn parse_unary(&mut self) -> Result<Expr<'a>, ParseError> {
            self.skip_ws();
            if self.eat_byte(b'!') {
                return Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(self.parse_unary()?),
                });
            }
            self.parse_primary()
        }

        fn parse_primary(&mut self) -> Result<Expr<'a>, ParseError> {
            self.skip_ws();
            match self.peek() {
                Some(b'(') => {
                    self.bump();
                    let expr = self.parse_expr()?;
                    self.skip_ws();
                    if !self.eat_byte(b')') {
                        return Err(self.err("expected ')'"));
                    }
                    Ok(expr)
                }
                Some(ch) if is_ident_start(ch) => {
                    let ident = self.parse_identifier()?;
                    match ident {
                        "N" => Ok(Expr::State(N)),
                        "F" => Ok(Expr::State(F)),
                        "T" => Ok(Expr::State(T)),
                        "S" => Ok(Expr::State(S)),
                        _ => Ok(Expr::Ident(ident)),
                    }
                }
                Some(_) => Err(self.err("unexpected token")),
                None => Err(self.err("unexpected end of input")),
            }
        }

        fn parse_identifier(&mut self) -> Result<&'a str, ParseError> {
            self.skip_ws();
            let start = self.pos;
            let first = self.peek().ok_or_else(|| self.err("expected identifier"))?;
            if !is_ident_start(first) {
                return Err(self.err("expected identifier"));
            }
            self.pos += 1;
            while let Some(ch) = self.peek() {
                if is_ident_continue(ch) {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            self.src
                .get(start..self.pos)
                .ok_or_else(|| self.err("invalid utf-8 boundary"))
        }
    }

    #[inline]
    fn is_ident_start(ch: u8) -> bool {
        ch.is_ascii_alphabetic() || ch == b'_'
    }

    #[inline]
    fn is_ident_continue(ch: u8) -> bool {
        ch.is_ascii_alphanumeric() || ch == b'_'
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse_respects_precedence() {
            let expr = parse_expression("T | F & N ^ S").expect("must parse");
            let expected = Expr::Binary {
                op: BinaryOp::Or,
                left: Box::new(Expr::State(T)),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::Xor,
                    left: Box::new(Expr::Binary {
                        op: BinaryOp::And,
                        left: Box::new(Expr::State(F)),
                        right: Box::new(Expr::State(N)),
                    }),
                    right: Box::new(Expr::State(S)),
                }),
            };
            assert_eq!(expr, expected);
        }

        #[test]
        fn parse_parentheses_and_unary() {
            let expr = parse_expression("!(a | T) & _x2").expect("must parse");
            let expected = Expr::Binary {
                op: BinaryOp::And,
                left: Box::new(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(Expr::Binary {
                        op: BinaryOp::Or,
                        left: Box::new(Expr::Ident("a")),
                        right: Box::new(Expr::State(T)),
                    }),
                }),
                right: Box::new(Expr::Ident("_x2")),
            };
            assert_eq!(expr, expected);
        }

        #[test]
        fn parse_assignment_ok() {
            let parsed = parse_assignment("out = a & !F").expect("must parse");
            assert_eq!(parsed.target, "out");
            assert_eq!(
                parsed.expr,
                Expr::Binary {
                    op: BinaryOp::And,
                    left: Box::new(Expr::Ident("a")),
                    right: Box::new(Expr::Unary {
                        op: UnaryOp::Not,
                        expr: Box::new(Expr::State(F)),
                    }),
                }
            );
        }

        #[test]
        fn parse_errors_are_reported() {
            let err = parse_expression("(T | F").expect_err("must fail");
            assert_eq!(err.message, "expected ')'");

            let err = parse_assignment("1bad = T").expect_err("must fail");
            assert_eq!(err.message, "expected identifier");
        }

        #[test]
        fn eval_expression_operators_work_on_registers() {
            let mut vars = HashMap::new();
            let mut a = QuadroReg::new();
            a = a.set_by_mask(1_u64 << (0 * 2), T);
            a = a.set_by_mask(1_u64 << (1 * 2), F);
            let mut b = QuadroReg::new();
            b = b.set_by_mask(1_u64 << (0 * 2), F);
            b = b.set_by_mask(1_u64 << (1 * 2), T);
            vars.insert("a", a);
            vars.insert("b", b);

            let expr = parse_expression("!(a & b) | (a ^ b)").expect("must parse");
            let got = eval_expression(&expr, &|name| vars.get(name).copied()).expect("must eval");

            let expected = a.intersect(b).inverse().merge(a.raw_delta(b));
            assert_eq!(got.raw(), expected.raw());
        }

        #[test]
        fn eval_assignment_returns_target_and_value() {
            let mut vars = HashMap::new();
            let x = QuadroReg::from_raw(0x1234_5678_9abc_def0);
            let y = QuadroReg::from_raw(0x0f0f_f0f0_55aa_a55a);
            vars.insert("x", x);
            vars.insert("y", y);

            let stmt = parse_assignment("out = x & y").expect("must parse");
            let (target, value) =
                eval_assignment(&stmt, &|name| vars.get(name).copied()).expect("must eval");

            assert_eq!(target, "out");
            assert_eq!(value.raw(), x.intersect(y).raw());
        }

        #[test]
        fn eval_unknown_identifier_fails() {
            let expr = parse_expression("a | T").expect("must parse");
            let err = eval_expression(&expr, &|_| None).expect_err("must fail");
            assert_eq!(err.ident, "a");
            assert_eq!(err.message, "unknown identifier");
        }

        #[test]
        fn execute_program_updates_context_in_order() {
            let mut env = HashMap::<String, QuadroReg>::new();
            let program = r#"
			a = T
			b = !a
			c = a ^ b
		"#;

            execute_program(program, &mut env).expect("must execute");

            assert_eq!(
                env.get("a").copied(),
                Some(QuadroReg::from_raw(crate::MSB_MASK))
            );
            assert_eq!(
                env.get("b").copied(),
                Some(QuadroReg::from_raw(crate::LSB_MASK))
            );
            assert_eq!(env.get("c").copied(), Some(QuadroReg::from_raw(u64::MAX)));
        }

        #[test]
        fn execute_program_supports_comments_and_blanks() {
            let mut env = HashMap::<String, QuadroReg>::new();
            let program = r#"
			# set initial value
			x = F // inline comment

			y = !x
		"#;

            execute_program(program, &mut env).expect("must execute");
            assert_eq!(
                env.get("x").copied(),
                Some(QuadroReg::from_raw(crate::LSB_MASK))
            );
            assert_eq!(
                env.get("y").copied(),
                Some(QuadroReg::from_raw(crate::MSB_MASK))
            );
        }

        #[test]
        fn execute_program_reports_line_for_eval_error() {
            let mut env = HashMap::<String, QuadroReg>::new();
            let program = "a = T\nb = z & a";
            let err = execute_program(program, &mut env).expect_err("must fail");

            assert_eq!(err.line, 2);
            assert_eq!(err.column, 1);
            assert!(err.message.contains("unknown identifier"));
            assert!(err.message.contains("z"));
        }

        #[test]
        fn execute_program_reports_line_for_parse_error() {
            let mut env = HashMap::<String, QuadroReg>::new();
            let program = "a = T\nbad line";
            let err = execute_program(program, &mut env).expect_err("must fail");

            assert_eq!(err.line, 2);
            assert!(err.column >= 1);
        }

        #[test]
        fn train_profile_learns_aliases_from_examples() {
            let samples = [
                TrainingSample {
                    input: "out = a AND b",
                    target: "out = a & b",
                },
                TrainingSample {
                    input: "q = a OR b",
                    target: "q = a | b",
                },
                TrainingSample {
                    input: "x = NOT FALSE",
                    target: "x = ! F",
                },
                TrainingSample {
                    input: "y = TRUE XOR x",
                    target: "y = T ^ x",
                },
            ];
            let profile = train_profile(&samples);

            assert_eq!(profile.aliases.get("AND"), Some(&"&".to_string()));
            assert_eq!(profile.aliases.get("OR"), Some(&"|".to_string()));
            assert_eq!(profile.aliases.get("NOT"), Some(&"!".to_string()));
            assert_eq!(profile.aliases.get("TRUE"), Some(&"T".to_string()));
            assert_eq!(profile.aliases.get("FALSE"), Some(&"F".to_string()));
            assert_eq!(profile.aliases.get("XOR"), Some(&"^".to_string()));
            assert_eq!(profile.normalize("z = TRUE AND NOT a"), "z = T & ! a");
        }

        #[test]
        fn execute_program_with_profile_uses_trained_aliases() {
            let samples = [
                TrainingSample {
                    input: "out = a AND b",
                    target: "out = a & b",
                },
                TrainingSample {
                    input: "q = a OR b",
                    target: "q = a | b",
                },
                TrainingSample {
                    input: "x = NOT FALSE",
                    target: "x = ! F",
                },
                TrainingSample {
                    input: "y = TRUE XOR x",
                    target: "y = T ^ x",
                },
            ];
            let profile = train_profile(&samples);

            let mut env = HashMap::<String, QuadroReg>::new();
            env.insert("a".to_string(), QuadroReg::from_raw(crate::MSB_MASK));
            env.insert("b".to_string(), QuadroReg::from_raw(crate::LSB_MASK));

            let program = r#"
			x = NOT FALSE
			y = TRUE XOR x
			out = y OR (a AND b)
		"#;
            execute_program_with_profile(program, &mut env, &profile).expect("must execute");

            let x = QuadroReg::from_raw(crate::LSB_MASK).inverse();
            let y = QuadroReg::from_raw(crate::MSB_MASK).raw_delta(x);
            let out = y.merge(
                QuadroReg::from_raw(crate::MSB_MASK)
                    .intersect(QuadroReg::from_raw(crate::LSB_MASK)),
            );
            assert_eq!(env.get("out").copied(), Some(out));
        }

        #[test]
        fn profile_json_roundtrip() {
            let mut profile = ParserProfile::default();
            profile.add_alias("AND", "&");
            profile.add_alias("TRUE", "T");

            let json = profile.to_json().expect("serialize");
            let restored = ParserProfile::from_json(&json).expect("deserialize");
            assert_eq!(restored, profile);
        }

        #[test]
        fn profile_save_and_load_file() {
            let mut profile = ParserProfile::default();
            profile.add_alias("OR", "|");
            profile.add_alias("NOT", "!");

            let path = std::env::temp_dir().join("smcode_parser_profile_test.json");
            profile.save_to_file(&path).expect("save");
            let loaded = ParserProfile::load_from_file(&path).expect("load");
            let _ = std::fs::remove_file(&path);

            assert_eq!(loaded, profile);
        }
    }
}

mod language {
    //! Compatibility-only language helpers for the retained TON618 CLI shim.

    use super::parser::{parse_assignment, BinaryOp as PBinaryOp, Expr, UnaryOp};
    use super::{QuadroReg, F, N, S, T};
    use sm_profile::ParserProfile;
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct MachineProgram {
        pub instructions: Vec<MachineInstr>,
    }

    impl MachineProgram {
        pub fn new() -> Self {
            Self {
                instructions: Vec::new(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum MachineInstr {
        SetState {
            dst: String,
            state: u8,
        },
        Mov {
            dst: String,
            src: String,
        },
        Not {
            dst: String,
            src: String,
        },
        And {
            dst: String,
            lhs: String,
            rhs: String,
        },
        Or {
            dst: String,
            lhs: String,
            rhs: String,
        },
        Xor {
            dst: String,
            lhs: String,
            rhs: String,
        },
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CompileError {
        pub line: usize,
        pub column: usize,
        pub message: String,
    }

    impl core::fmt::Display for CompileError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(
                f,
                "compile error at line {}, column {}: {}",
                self.line, self.column, self.message
            )
        }
    }

    impl std::error::Error for CompileError {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct MachineParseError {
        pub line: usize,
        pub column: usize,
        pub message: String,
    }

    impl core::fmt::Display for MachineParseError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(
                f,
                "machine parse error at line {}, column {}: {}",
                self.line, self.column, self.message
            )
        }
    }

    impl std::error::Error for MachineParseError {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RuntimeError {
        pub ip: usize,
        pub message: String,
    }

    impl core::fmt::Display for RuntimeError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(
                f,
                "runtime error at instruction {}: {}",
                self.ip, self.message
            )
        }
    }

    impl std::error::Error for RuntimeError {}

    pub fn compile_human_program(
        program: &str,
        profile: Option<&ParserProfile>,
    ) -> Result<MachineProgram, CompileError> {
        let mut out = MachineProgram::new();
        let mut lower = LoweringCtx::new();

        for (line_idx, raw_line) in program.lines().enumerate() {
            let line_no = line_idx + 1;
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }
            let normalized = if let Some(p) = profile {
                p.normalize(line)
            } else {
                line.to_string()
            };

            let assignment = parse_assignment(&normalized).map_err(|e| CompileError {
                line: line_no,
                column: e.position + 1,
                message: e.message.to_string(),
            })?;

            let src = lower.lower_expr(&assignment.expr);
            let dst = assignment.target.to_string();
            if src != dst {
                lower.instructions.push(MachineInstr::Mov { dst, src });
            }
        }

        out.instructions = optimize_machine_program(&MachineProgram {
            instructions: lower.instructions,
        })
        .instructions;
        Ok(out)
    }

    pub fn optimize_machine_program(program: &MachineProgram) -> MachineProgram {
        let folded = fold_and_simplify(&program.instructions);
        let dce = eliminate_dead_temps(&folded);
        MachineProgram { instructions: dce }
    }

    pub fn render_machine_program(program: &MachineProgram) -> String {
        let mut out = String::new();
        for instr in &program.instructions {
            match instr {
                MachineInstr::SetState { dst, state } => {
                    out.push_str("SET ");
                    out.push_str(dst);
                    out.push(' ');
                    out.push_str(match *state {
                        N => "N",
                        F => "F",
                        T => "T",
                        S => "S",
                        _ => "N",
                    });
                }
                MachineInstr::Mov { dst, src } => {
                    out.push_str("MOV ");
                    out.push_str(dst);
                    out.push(' ');
                    out.push_str(src);
                }
                MachineInstr::Not { dst, src } => {
                    out.push_str("NOT ");
                    out.push_str(dst);
                    out.push(' ');
                    out.push_str(src);
                }
                MachineInstr::And { dst, lhs, rhs } => {
                    out.push_str("AND ");
                    out.push_str(dst);
                    out.push(' ');
                    out.push_str(lhs);
                    out.push(' ');
                    out.push_str(rhs);
                }
                MachineInstr::Or { dst, lhs, rhs } => {
                    out.push_str("OR ");
                    out.push_str(dst);
                    out.push(' ');
                    out.push_str(lhs);
                    out.push(' ');
                    out.push_str(rhs);
                }
                MachineInstr::Xor { dst, lhs, rhs } => {
                    out.push_str("XOR ");
                    out.push_str(dst);
                    out.push(' ');
                    out.push_str(lhs);
                    out.push(' ');
                    out.push_str(rhs);
                }
            }
            out.push('\n');
        }
        out
    }

    pub fn parse_machine_program(src: &str) -> Result<MachineProgram, MachineParseError> {
        let mut program = MachineProgram::new();

        for (line_idx, raw_line) in src.lines().enumerate() {
            let line_no = line_idx + 1;
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            let op = parts[0].to_ascii_uppercase();
            let instr = match op.as_str() {
                "SET" => {
                    if parts.len() != 3 {
                        return Err(parse_err(line_no, "SET expects: SET <dst> <N|F|T|S>"));
                    }
                    let state = parse_state(parts[2])
                        .ok_or_else(|| parse_err(line_no, "invalid SET state, expected N|F|T|S"))?;
                    MachineInstr::SetState {
                        dst: parts[1].to_string(),
                        state,
                    }
                }
                "MOV" => {
                    if parts.len() != 3 {
                        return Err(parse_err(line_no, "MOV expects: MOV <dst> <src>"));
                    }
                    MachineInstr::Mov {
                        dst: parts[1].to_string(),
                        src: parts[2].to_string(),
                    }
                }
                "NOT" => {
                    if parts.len() != 3 {
                        return Err(parse_err(line_no, "NOT expects: NOT <dst> <src>"));
                    }
                    MachineInstr::Not {
                        dst: parts[1].to_string(),
                        src: parts[2].to_string(),
                    }
                }
                "AND" | "OR" | "XOR" => {
                    if parts.len() != 4 {
                        return Err(parse_err(
                            line_no,
                            "AND/OR/XOR expects: <OP> <dst> <lhs> <rhs>",
                        ));
                    }
                    match op.as_str() {
                        "AND" => MachineInstr::And {
                            dst: parts[1].to_string(),
                            lhs: parts[2].to_string(),
                            rhs: parts[3].to_string(),
                        },
                        "OR" => MachineInstr::Or {
                            dst: parts[1].to_string(),
                            lhs: parts[2].to_string(),
                            rhs: parts[3].to_string(),
                        },
                        _ => MachineInstr::Xor {
                            dst: parts[1].to_string(),
                            lhs: parts[2].to_string(),
                            rhs: parts[3].to_string(),
                        },
                    }
                }
                _ => {
                    return Err(parse_err(
                        line_no,
                        "unknown instruction, expected SET|MOV|NOT|AND|OR|XOR",
                    ))
                }
            };
            program.instructions.push(instr);
        }

        Ok(program)
    }

    pub fn execute_machine_program(
        program: &MachineProgram,
        env: &mut HashMap<String, QuadroReg>,
    ) -> Result<(), RuntimeError> {
        for (ip, instr) in program.instructions.iter().enumerate() {
            match instr {
                MachineInstr::SetState { dst, state } => {
                    env.insert(dst.clone(), fill_state(*state));
                }
                MachineInstr::Mov { dst, src } => {
                    let v = *env
                        .get(src)
                        .ok_or_else(|| rt_err(ip, format!("unknown variable '{}'", src)))?;
                    env.insert(dst.clone(), v);
                }
                MachineInstr::Not { dst, src } => {
                    let v = *env
                        .get(src)
                        .ok_or_else(|| rt_err(ip, format!("unknown variable '{}'", src)))?;
                    env.insert(dst.clone(), v.inverse());
                }
                MachineInstr::And { dst, lhs, rhs } => {
                    let lv = *env
                        .get(lhs)
                        .ok_or_else(|| rt_err(ip, format!("unknown variable '{}'", lhs)))?;
                    let rv = *env
                        .get(rhs)
                        .ok_or_else(|| rt_err(ip, format!("unknown variable '{}'", rhs)))?;
                    env.insert(dst.clone(), lv.intersect(rv));
                }
                MachineInstr::Or { dst, lhs, rhs } => {
                    let lv = *env
                        .get(lhs)
                        .ok_or_else(|| rt_err(ip, format!("unknown variable '{}'", lhs)))?;
                    let rv = *env
                        .get(rhs)
                        .ok_or_else(|| rt_err(ip, format!("unknown variable '{}'", rhs)))?;
                    env.insert(dst.clone(), lv.merge(rv));
                }
                MachineInstr::Xor { dst, lhs, rhs } => {
                    let lv = *env
                        .get(lhs)
                        .ok_or_else(|| rt_err(ip, format!("unknown variable '{}'", lhs)))?;
                    let rv = *env
                        .get(rhs)
                        .ok_or_else(|| rt_err(ip, format!("unknown variable '{}'", rhs)))?;
                    env.insert(dst.clone(), lv.raw_delta(rv));
                }
            }
        }
        Ok(())
    }

    struct LoweringCtx {
        instructions: Vec<MachineInstr>,
        next_temp: usize,
    }

    impl LoweringCtx {
        fn new() -> Self {
            Self {
                instructions: Vec::new(),
                next_temp: 0,
            }
        }

        fn tmp(&mut self) -> String {
            let n = self.next_temp;
            self.next_temp += 1;
            format!("__t{}", n)
        }

        fn lower_expr(&mut self, expr: &Expr<'_>) -> String {
            match expr {
                Expr::State(state) => {
                    let dst = self.tmp();
                    self.instructions.push(MachineInstr::SetState {
                        dst: dst.clone(),
                        state: *state,
                    });
                    dst
                }
                Expr::Ident(name) => (*name).to_string(),
                Expr::Unary { op, expr } => {
                    let src = self.lower_expr(expr);
                    let dst = self.tmp();
                    match op {
                        UnaryOp::Not => self.instructions.push(MachineInstr::Not {
                            dst: dst.clone(),
                            src,
                        }),
                    }
                    dst
                }
                Expr::Binary { op, left, right } => {
                    let lhs = self.lower_expr(left);
                    let rhs = self.lower_expr(right);
                    let dst = self.tmp();
                    match op {
                        PBinaryOp::And => self.instructions.push(MachineInstr::And {
                            dst: dst.clone(),
                            lhs,
                            rhs,
                        }),
                        PBinaryOp::Or => self.instructions.push(MachineInstr::Or {
                            dst: dst.clone(),
                            lhs,
                            rhs,
                        }),
                        PBinaryOp::Xor => self.instructions.push(MachineInstr::Xor {
                            dst: dst.clone(),
                            lhs,
                            rhs,
                        }),
                    }
                    dst
                }
            }
        }
    }

    #[inline]
    fn fill_state(state: u8) -> QuadroReg {
        match state {
            N => QuadroReg::from_raw(0),
            F => QuadroReg::from_raw(crate::LSB_MASK),
            T => QuadroReg::from_raw(crate::MSB_MASK),
            S => QuadroReg::from_raw(u64::MAX),
            _ => QuadroReg::from_raw(0),
        }
    }

    #[inline]
    fn parse_state(s: &str) -> Option<u8> {
        match s {
            "N" => Some(N),
            "F" => Some(F),
            "T" => Some(T),
            "S" => Some(S),
            _ => None,
        }
    }

    #[inline]
    fn strip_comment(line: &str) -> &str {
        let slash = line.find("//");
        let hash = line.find('#');
        match (slash, hash) {
            (Some(a), Some(b)) => &line[..a.min(b)],
            (Some(a), None) => &line[..a],
            (None, Some(b)) => &line[..b],
            (None, None) => line,
        }
    }

    #[inline]
    fn parse_err(line: usize, message: &str) -> MachineParseError {
        MachineParseError {
            line,
            column: 1,
            message: message.to_string(),
        }
    }

    #[inline]
    fn rt_err(ip: usize, message: String) -> RuntimeError {
        RuntimeError { ip, message }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Value {
        Const(u8),
        Alias(String),
        Unknown,
    }

    fn fold_and_simplify(input: &[MachineInstr]) -> Vec<MachineInstr> {
        let mut out = Vec::with_capacity(input.len());
        let mut values: HashMap<String, Value> = HashMap::new();

        for instr in input {
            match instr {
                MachineInstr::SetState { dst, state } => {
                    values.insert(dst.clone(), Value::Const(*state));
                    out.push(MachineInstr::SetState {
                        dst: dst.clone(),
                        state: *state,
                    });
                }
                MachineInstr::Mov { dst, src } => {
                    let resolved = resolve_alias(src, &values);
                    let src_v = read_value(&resolved, &values);
                    if dst == &resolved {
                        values.insert(dst.clone(), src_v);
                        continue;
                    }
                    match src_v {
                        Value::Const(c) => {
                            values.insert(dst.clone(), Value::Const(c));
                            out.push(MachineInstr::SetState {
                                dst: dst.clone(),
                                state: c,
                            });
                        }
                        _ => {
                            values.insert(dst.clone(), Value::Alias(resolved.clone()));
                            out.push(MachineInstr::Mov {
                                dst: dst.clone(),
                                src: resolved,
                            });
                        }
                    }
                }
                MachineInstr::Not { dst, src } => {
                    let src_r = resolve_alias(src, &values);
                    match read_value(&src_r, &values) {
                        Value::Const(c) => {
                            let r = inverse_state(c);
                            values.insert(dst.clone(), Value::Const(r));
                            out.push(MachineInstr::SetState {
                                dst: dst.clone(),
                                state: r,
                            });
                        }
                        _ => {
                            values.insert(dst.clone(), Value::Unknown);
                            out.push(MachineInstr::Not {
                                dst: dst.clone(),
                                src: src_r,
                            });
                        }
                    }
                }
                MachineInstr::And { dst, lhs, rhs } => {
                    let lhs_r = resolve_alias(lhs, &values);
                    let rhs_r = resolve_alias(rhs, &values);
                    let lhs_v = read_value(&lhs_r, &values);
                    let rhs_v = read_value(&rhs_r, &values);
                    let simplified = simplify_binary("AND", dst, &lhs_r, &rhs_r, &lhs_v, &rhs_v);
                    track_result_value(&mut values, &simplified);
                    out.push(simplified);
                }
                MachineInstr::Or { dst, lhs, rhs } => {
                    let lhs_r = resolve_alias(lhs, &values);
                    let rhs_r = resolve_alias(rhs, &values);
                    let lhs_v = read_value(&lhs_r, &values);
                    let rhs_v = read_value(&rhs_r, &values);
                    let simplified = simplify_binary("OR", dst, &lhs_r, &rhs_r, &lhs_v, &rhs_v);
                    track_result_value(&mut values, &simplified);
                    out.push(simplified);
                }
                MachineInstr::Xor { dst, lhs, rhs } => {
                    let lhs_r = resolve_alias(lhs, &values);
                    let rhs_r = resolve_alias(rhs, &values);
                    let lhs_v = read_value(&lhs_r, &values);
                    let rhs_v = read_value(&rhs_r, &values);
                    let simplified = simplify_binary("XOR", dst, &lhs_r, &rhs_r, &lhs_v, &rhs_v);
                    track_result_value(&mut values, &simplified);
                    out.push(simplified);
                }
            }
        }

        out
    }

    fn eliminate_dead_temps(input: &[MachineInstr]) -> Vec<MachineInstr> {
        let mut live: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut keep = vec![false; input.len()];

        for (i, instr) in input.iter().enumerate().rev() {
            let def = defined_var(instr);
            let uses = used_vars(instr);
            let must_keep = def
                .as_ref()
                .map(|d| !is_temp(d) || live.contains(d))
                .unwrap_or(true);
            if must_keep {
                keep[i] = true;
                if let Some(d) = def {
                    live.remove(&d);
                }
                for u in uses {
                    live.insert(u);
                }
            }
        }

        let mut out = Vec::with_capacity(input.len());
        for (i, instr) in input.iter().enumerate() {
            if keep[i] {
                out.push(instr.clone());
            }
        }
        out
    }

    fn defined_var(instr: &MachineInstr) -> Option<String> {
        match instr {
            MachineInstr::SetState { dst, .. } => Some(dst.clone()),
            MachineInstr::Mov { dst, .. } => Some(dst.clone()),
            MachineInstr::Not { dst, .. } => Some(dst.clone()),
            MachineInstr::And { dst, .. } => Some(dst.clone()),
            MachineInstr::Or { dst, .. } => Some(dst.clone()),
            MachineInstr::Xor { dst, .. } => Some(dst.clone()),
        }
    }

    fn used_vars(instr: &MachineInstr) -> Vec<String> {
        match instr {
            MachineInstr::SetState { .. } => Vec::new(),
            MachineInstr::Mov { src, .. } => vec![src.clone()],
            MachineInstr::Not { src, .. } => vec![src.clone()],
            MachineInstr::And { lhs, rhs, .. } => vec![lhs.clone(), rhs.clone()],
            MachineInstr::Or { lhs, rhs, .. } => vec![lhs.clone(), rhs.clone()],
            MachineInstr::Xor { lhs, rhs, .. } => vec![lhs.clone(), rhs.clone()],
        }
    }

    #[inline]
    fn is_temp(name: &str) -> bool {
        name.starts_with("__t")
    }

    fn resolve_alias(name: &str, values: &HashMap<String, Value>) -> String {
        let mut cur = name.to_string();
        let mut steps = 0usize;
        while steps < 64 {
            match values.get(&cur) {
                Some(Value::Alias(next)) if next != &cur => {
                    cur = next.clone();
                    steps += 1;
                }
                _ => break,
            }
        }
        cur
    }

    fn read_value(name: &str, values: &HashMap<String, Value>) -> Value {
        match values.get(name) {
            Some(v) => v.clone(),
            None => Value::Unknown,
        }
    }

    fn track_result_value(values: &mut HashMap<String, Value>, instr: &MachineInstr) {
        match instr {
            MachineInstr::SetState { dst, state } => {
                values.insert(dst.clone(), Value::Const(*state));
            }
            MachineInstr::Mov { dst, src } => {
                values.insert(dst.clone(), Value::Alias(src.clone()));
            }
            MachineInstr::Not { dst, .. }
            | MachineInstr::And { dst, .. }
            | MachineInstr::Or { dst, .. }
            | MachineInstr::Xor { dst, .. } => {
                values.insert(dst.clone(), Value::Unknown);
            }
        }
    }

    fn simplify_binary(
        op: &str,
        dst: &str,
        lhs: &str,
        rhs: &str,
        lhs_v: &Value,
        rhs_v: &Value,
    ) -> MachineInstr {
        match (lhs_v, rhs_v) {
            (Value::Const(a), Value::Const(b)) => {
                return MachineInstr::SetState {
                    dst: dst.to_string(),
                    state: apply_state_binary(op, *a, *b),
                };
            }
            _ => {}
        }

        if lhs == rhs {
            return match op {
                "XOR" => MachineInstr::SetState {
                    dst: dst.to_string(),
                    state: N,
                },
                _ => MachineInstr::Mov {
                    dst: dst.to_string(),
                    src: lhs.to_string(),
                },
            };
        }

        match op {
            "AND" => {
                if is_const_state(lhs_v, N) || is_const_state(rhs_v, N) {
                    return MachineInstr::SetState {
                        dst: dst.to_string(),
                        state: N,
                    };
                }
                if is_const_state(lhs_v, S) {
                    return MachineInstr::Mov {
                        dst: dst.to_string(),
                        src: rhs.to_string(),
                    };
                }
                if is_const_state(rhs_v, S) {
                    return MachineInstr::Mov {
                        dst: dst.to_string(),
                        src: lhs.to_string(),
                    };
                }
                MachineInstr::And {
                    dst: dst.to_string(),
                    lhs: lhs.to_string(),
                    rhs: rhs.to_string(),
                }
            }
            "OR" => {
                if is_const_state(lhs_v, S) || is_const_state(rhs_v, S) {
                    return MachineInstr::SetState {
                        dst: dst.to_string(),
                        state: S,
                    };
                }
                if is_const_state(lhs_v, N) {
                    return MachineInstr::Mov {
                        dst: dst.to_string(),
                        src: rhs.to_string(),
                    };
                }
                if is_const_state(rhs_v, N) {
                    return MachineInstr::Mov {
                        dst: dst.to_string(),
                        src: lhs.to_string(),
                    };
                }
                MachineInstr::Or {
                    dst: dst.to_string(),
                    lhs: lhs.to_string(),
                    rhs: rhs.to_string(),
                }
            }
            "XOR" => {
                if is_const_state(lhs_v, N) {
                    return MachineInstr::Mov {
                        dst: dst.to_string(),
                        src: rhs.to_string(),
                    };
                }
                if is_const_state(rhs_v, N) {
                    return MachineInstr::Mov {
                        dst: dst.to_string(),
                        src: lhs.to_string(),
                    };
                }
                MachineInstr::Xor {
                    dst: dst.to_string(),
                    lhs: lhs.to_string(),
                    rhs: rhs.to_string(),
                }
            }
            _ => MachineInstr::Xor {
                dst: dst.to_string(),
                lhs: lhs.to_string(),
                rhs: rhs.to_string(),
            },
        }
    }

    #[inline]
    fn is_const_state(v: &Value, state: u8) -> bool {
        matches!(v, Value::Const(s) if *s == state)
    }

    #[inline]
    fn inverse_state(state: u8) -> u8 {
        ((state & 0b10) >> 1) | ((state & 0b01) << 1)
    }

    #[inline]
    fn apply_state_binary(op: &str, a: u8, b: u8) -> u8 {
        match op {
            "AND" => a & b,
            "OR" => a | b,
            "XOR" => a ^ b,
            _ => a ^ b,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use sm_profile::{train_profile, TrainingSample};

        #[test]
        fn compile_and_execute_human_program_with_profile() {
            let profile = train_profile(&[
                TrainingSample {
                    input: "x = TRUE",
                    target: "x = T",
                },
                TrainingSample {
                    input: "y = FALSE",
                    target: "y = F",
                },
                TrainingSample {
                    input: "out = x AND y",
                    target: "out = x & y",
                },
                TrainingSample {
                    input: "z = NOT out",
                    target: "z = ! out",
                },
            ]);

            let human = r#"
			x = TRUE
			y = FALSE
			out = x AND y
			z = NOT out
		"#;
            let program = compile_human_program(human, Some(&profile)).expect("compile");
            let mut env = HashMap::<String, QuadroReg>::new();
            execute_machine_program(&program, &mut env).expect("run");

            let expected_out = QuadroReg::from_raw(crate::MSB_MASK)
                .intersect(QuadroReg::from_raw(crate::LSB_MASK));
            let expected_z = expected_out.inverse();
            assert_eq!(env.get("z").copied(), Some(expected_z));
        }

        #[test]
        fn machine_text_parse_and_run() {
            let src = r#"
			SET a T
			SET b F
			AND out a b
			NOT inv out
		"#;
            let program = parse_machine_program(src).expect("parse");
            let mut env = HashMap::<String, QuadroReg>::new();
            execute_machine_program(&program, &mut env).expect("run");
            assert!(env.contains_key("inv"));
        }

        #[test]
        fn machine_parse_rejects_bad_instruction() {
            let err = parse_machine_program("BAD x y").expect_err("must fail");
            assert_eq!(err.line, 1);
            assert!(err.message.contains("unknown instruction"));
        }

        #[test]
        fn optimizer_folds_constants_and_eliminates_temps() {
            let profile = train_profile(&[
                TrainingSample {
                    input: "x = TRUE",
                    target: "x = T",
                },
                TrainingSample {
                    input: "y = FALSE",
                    target: "y = F",
                },
                TrainingSample {
                    input: "out = x AND y",
                    target: "out = x & y",
                },
            ]);
            let human = "x = TRUE\ny = FALSE\nout = x AND y\n";
            let program = compile_human_program(human, Some(&profile)).expect("compile");

            assert!(program
                .instructions
                .iter()
                .all(|i| !format!("{:?}", i).contains("__t")));
            assert!(program.instructions.len() <= 3);
        }

        #[test]
        fn optimizer_removes_redundant_moves() {
            let p = MachineProgram {
                instructions: vec![
                    MachineInstr::SetState {
                        dst: "a".to_string(),
                        state: T,
                    },
                    MachineInstr::Mov {
                        dst: "b".to_string(),
                        src: "a".to_string(),
                    },
                    MachineInstr::Mov {
                        dst: "c".to_string(),
                        src: "b".to_string(),
                    },
                ],
            };
            let o = optimize_machine_program(&p);
            assert_eq!(
                o.instructions,
                vec![
                    MachineInstr::SetState {
                        dst: "a".to_string(),
                        state: T,
                    },
                    MachineInstr::SetState {
                        dst: "b".to_string(),
                        state: T,
                    },
                    MachineInstr::SetState {
                        dst: "c".to_string(),
                        state: T,
                    },
                ]
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_alias_pair_ok() {
        let (a, b) = parse_alias_pair("AND=&").expect("must parse");
        assert_eq!(a, "AND");
        assert_eq!(b, "&");
    }

    #[test]
    fn parse_alias_pair_rejects_invalid() {
        assert!(parse_alias_pair("ANDEQ").is_err());
        assert!(parse_alias_pair("=x").is_err());
        assert!(parse_alias_pair("x=").is_err());
    }

    #[test]
    fn parse_seed_pair_ok() {
        let (k, v) = parse_seed_pair("a=T").expect("must parse");
        assert_eq!(k, "a");
        assert_eq!(v, T);
    }

    #[test]
    fn parse_seed_pair_rejects_invalid() {
        assert!(parse_seed_pair("a=").is_err());
        assert!(parse_seed_pair("=T").is_err());
        assert!(parse_seed_pair("a=X").is_err());
    }
}
