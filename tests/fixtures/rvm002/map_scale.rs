//! Experimental actual VM scaling probe; production quota defaults unchanged.
use sm_emit::compile_program_to_semcode;
use sm_runtime_core::{ExecutionConfig,ExecutionContext};
use std::alloc::{GlobalAlloc,Layout,System};
use std::sync::atomic::{AtomicUsize,Ordering::Relaxed};
use std::time::Instant;
struct Alloc;
static A:AtomicUsize=AtomicUsize::new(0);
static B:AtomicUsize=AtomicUsize::new(0);
static L:AtomicUsize=AtomicUsize::new(0);
static P:AtomicUsize=AtomicUsize::new(0);
unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self,l:Layout)->*mut u8 {let p=System.alloc(l);if !p.is_null(){A.fetch_add(1,Relaxed);B.fetch_add(l.size(),Relaxed);P.fetch_max(L.fetch_add(l.size(),Relaxed)+l.size(),Relaxed);}p}
    unsafe fn dealloc(&self,p:*mut u8,l:Layout){L.fetch_sub(l.size(),Relaxed);System.dealloc(p,l)}
}
#[global_allocator] static ALLOC:Alloc=Alloc;
struct Host;
impl prom_abi::ApplicationHostAbi for Host {
    fn args_read(&mut self,_:u32)->Result<String,prom_abi::AbiError>{panic!("effect")}
    fn stdout_write(&mut self,_:&str)->Result<(),prom_abi::AbiError>{panic!("effect")}
    fn stdin_read_text(&mut self)->Result<String,prom_abi::AbiError>{panic!("effect")}
    fn stderr_write(&mut self,_:&str)->Result<(),prom_abi::AbiError>{panic!("effect")}
    fn path_inspect(&mut self,_:&str)->Result<bool,prom_abi::AbiError>{panic!("effect")}
    fn fs_read_text(&mut self,_:&str)->Result<String,prom_abi::AbiError>{panic!("effect")}
    fn fs_write_text(&mut self,_:&str,_:&str)->Result<(),prom_abi::AbiError>{panic!("effect")}
    fn time_duration_millis(&mut self)->Result<u32,prom_abi::AbiError>{panic!("effect")}
}
fn main(){
    let args:Vec<String>=std::env::args().collect();
    let n:i32=args[1].parse().unwrap();let samples:usize=args[2].parse().unwrap();let kind=&args[3];
    let tail=if kind=="replace" {format!("i = 0; while i < {n} {{ m = map_set(m,i,i+1); i=i+1; }} assert(map_get(m,{},-1)=={n});",n-1)} else {format!("assert(map_get(m,{},-1)=={});",n-1,n-1)};
    let source=format!("fn main() {{ let m: Map(i32,i32)=map_empty(); let i:i32=0; while i < {n} {{ m=map_set(m,i,i); i=i+1; }} {tail} return; }}");
    let bytes=compile_program_to_semcode(&source).expect("compile");
    std::fs::write(format!("{}/{kind}-{n}.smc",args[4]),&bytes).unwrap();
    std::fs::write(format!("{}/{kind}-{n}.sm",args[4]),source).unwrap();
    let mut config=ExecutionConfig::for_context(ExecutionContext::VerifiedLocal);
    config.quotas.max_steps=2_000_000;
    let token=sm_verify::verify_semcode_token_with_quotas(&bytes,config.quotas).expect("admission");
    let entry=token.require_entry("main").unwrap();
    for sample in 0..samples {
        let live=L.load(Relaxed);P.store(live,Relaxed);let a=A.load(Relaxed);let b=B.load(Relaxed);let start=Instant::now();
        let run=sm_vm::run_verified_entry_semcode_with_application_host_and_capabilities_and_config_metered(&entry,&mut Host,&prom_cap::CapabilityManifest::new(),config);
        let elapsed=start.elapsed().as_nanos();assert!(run.result.is_ok(),"{run:?}");let c=run.counters.unwrap();
        println!("{kind},{n},{sample},{elapsed},{},{},{},{},{},{},OK",A.load(Relaxed)-a,B.load(Relaxed)-b,P.load(Relaxed)-live,c.steps,c.calls,c.peak_frames);
    }
}
