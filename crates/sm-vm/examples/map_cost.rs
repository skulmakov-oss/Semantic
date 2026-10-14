//! SEM-RVM-001 diagnostic benchmark. No production allocator or quota changes.
use sm_emit::{compile_program_to_semcode, emit_ir_to_semcode};
use sm_ir::{IrFunction, IrInstr};
use sm_vm::{run_verified_function_semcode_with_args, MapKey, Value};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::Instant;

struct CountingAllocator;
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(layout.size(), Relaxed);
            let live = LIVE.fetch_add(layout.size(), Relaxed) + layout.size();
            PEAK.fetch_max(live, Relaxed);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Relaxed);
        System.dealloc(ptr, layout);
    }
}
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

struct InputHost {
    input: String,
    output: String,
}
impl prom_abi::ApplicationHostAbi for InputHost {
    fn args_read(&mut self, index: u32) -> Result<String, prom_abi::AbiError> {
        assert_eq!(index, 0);
        Ok(self.input.clone())
    }
    fn stdout_write(&mut self, s: &str) -> Result<(), prom_abi::AbiError> {
        self.output.push_str(s);
        Ok(())
    }
    fn stdin_read_text(&mut self) -> Result<String, prom_abi::AbiError> {
        panic!("unexpected stdin")
    }
    fn stderr_write(&mut self, _: &str) -> Result<(), prom_abi::AbiError> {
        panic!("unexpected stderr")
    }
    fn path_inspect(&mut self, _: &str) -> Result<bool, prom_abi::AbiError> {
        panic!("unexpected path")
    }
    fn fs_read_text(&mut self, _: &str) -> Result<String, prom_abi::AbiError> {
        panic!("unexpected read")
    }
    fn fs_write_text(&mut self, _: &str, _: &str) -> Result<(), prom_abi::AbiError> {
        panic!("unexpected write")
    }
    fn time_duration_millis(&mut self) -> Result<u32, prom_abi::AbiError> {
        panic!("unexpected clock")
    }
}

fn save_artifact(name: &str, bytes: &[u8]) {
    if let Some(directory) = std::env::args().nth(2) {
        std::fs::write(
            std::path::Path::new(&directory).join(format!("{name}.smc")),
            bytes,
        )
        .unwrap();
    }
}

fn main() {
    println!("case,n,sample,ns,allocations,allocated_bytes,peak_extra_live_bytes,outcome");
    for n in [16, 128, 1024] {
        for operation in ["get", "contains", "insert", "replace", "clone"] {
            let mut instrs = vec![
                IrInstr::LoadI32 {
                    dst: 1,
                    val: if operation == "insert" { n } else { n - 1 },
                },
                IrInstr::LoadI32 { dst: 2, val: -1 },
            ];
            for _ in 0..100 {
                instrs.push(match operation {
                    "get" => IrInstr::MapGet {
                        dst: 3,
                        map: 0,
                        key: 1,
                        default_val: 2,
                    },
                    "contains" => IrInstr::MapContains {
                        dst: 3,
                        map: 0,
                        key: 1,
                    },
                    "insert" | "replace" => IrInstr::MapSet {
                        dst: 3,
                        map: 0,
                        key: 1,
                        val: 2,
                    },
                    _ => IrInstr::MapContains {
                        dst: 3,
                        map: 0,
                        key: 1,
                    },
                });
            }
            instrs.push(IrInstr::Ret { src: None });
            let bytes = emit_ir_to_semcode(
                &[IrFunction {
                    name: "main".into(),
                    instrs,
                    ownership_events: vec![],
                    params: vec![sm_emit::CallableValueFamily::Map],
                }],
                false,
            )
            .unwrap();
            save_artifact(&format!("{operation}-{n}"), &bytes);
            let token = sm_verify::verify_semcode_token(&bytes).expect("admission");
            let entry = token.require_entry("main").unwrap();
            let map = Value::Map(
                (0..n)
                    .map(|key| (MapKey::I32(key), Value::Text("x".repeat(256))))
                    .collect(),
            );
            for sample in 0..9 {
                let start_live = LIVE.load(Relaxed);
                PEAK.store(start_live, Relaxed);
                let a = ALLOCS.load(Relaxed);
                let b = BYTES.load(Relaxed);
                let start = Instant::now();
                if operation == "clone" {
                    for _ in 0..100 {
                        std::hint::black_box(map.clone());
                    }
                } else {
                    let result = run_verified_function_semcode_with_args(&entry, vec![map.clone()]);
                    assert_eq!(result, Ok(Value::Unit));
                }
                let ns = start.elapsed().as_nanos();
                println!(
                    "{operation},{n},{sample},{ns},{},{},{},OK",
                    ALLOCS.load(Relaxed) - a,
                    BYTES.load(Relaxed) - b,
                    PEAK.load(Relaxed).saturating_sub(start_live)
                );
            }
        }
    }
    // Compiler-generated loop, including locals, dispatch and persistent construction.
    for n in [16, 128, 512] {
        let source = format!("fn main() {{ let m: Map(i32, i32) = map_empty(); let i: i32 = 0; while i < {n} {{ m = map_set(m, i, i); i = i + 1; }} assert(map_get(m, {}, -1) == {}); return; }}", n-1, n-1);
        let bytes = compile_program_to_semcode(&source).expect("compile source");
        save_artifact(&format!("source-build-{n}"), &bytes);
        let token = sm_verify::verify_semcode_token(&bytes).expect("source admission");
        let entry = token.require_entry("main").unwrap();
        for sample in 0..9 {
            let start_live = LIVE.load(Relaxed);
            PEAK.store(start_live, Relaxed);
            let a = ALLOCS.load(Relaxed);
            let b = BYTES.load(Relaxed);
            let start = Instant::now();
            assert_eq!(
                run_verified_function_semcode_with_args(&entry, vec![]),
                Ok(Value::Unit)
            );
            println!(
                "source_build,{n},{sample},{},{},{},{},OK",
                start.elapsed().as_nanos(),
                ALLOCS.load(Relaxed) - a,
                BYTES.load(Relaxed) - b,
                PEAK.load(Relaxed).saturating_sub(start_live)
            );
        }
    }
    // Optional actual Semantic-written compiler/checker/interpreter artifact.
    if let Some(path) = std::env::args().nth(1) {
        let bytes = std::fs::read(path).unwrap();
        let config = sm_runtime_core::ExecutionConfig::for_context(
            sm_runtime_core::ExecutionContext::VerifiedLocal,
        );
        let token = sm_verify::verify_semcode_token_with_quotas(&bytes, config.quotas)
            .expect("SemSeed admission");
        let entry = token.require_entry("main").unwrap();
        let mut caps = prom_cap::CapabilityManifest::new();
        caps.allow(prom_cap::CapabilityKind::ArgsRead);
        caps.allow(prom_cap::CapabilityKind::StdoutWrite);
        for sample in 0..9 {
            let mut host = InputHost {
                input: "let x:int = 10; x = x - 3; x = x * 2; return x;".into(),
                output: String::new(),
            };
            let live = LIVE.load(Relaxed);
            PEAK.store(live, Relaxed);
            let a = ALLOCS.load(Relaxed);
            let b = BYTES.load(Relaxed);
            let start = Instant::now();
            let run=sm_vm::run_verified_entry_semcode_with_application_host_and_capabilities_and_config_metered(&entry,&mut host,&caps,config);
            let ns = start.elapsed().as_nanos();
            assert!(run.result.is_ok(), "{run:?}");
            assert!(host.output.contains("ok:int:14"), "{}", host.output);
            let counters = run.counters.expect("counters");
            println!(
                "semseed,1,{sample},{ns},{},{},{},OK:s{}:c{}:f{}:e{}",
                ALLOCS.load(Relaxed) - a,
                BYTES.load(Relaxed) - b,
                PEAK.load(Relaxed).saturating_sub(live),
                counters.steps,
                counters.calls,
                counters.peak_frames,
                counters.effect_calls
            );
        }
    }
}
