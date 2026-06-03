use sm_emit::{compile_program_to_semcode, header_spec_from_magic};

fn check_emit_ok(src: &str) -> Vec<u8> {
    let bytes = compile_program_to_semcode(src).expect("semcode emission failed");
    assert!(
        bytes.len() >= 8,
        "SemCode must be at least 8 bytes for header"
    );
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("header magic not recognized");
    // Just sanity checking we got a spec
    assert!(spec.rev >= 1);
    bytes
}

#[test]
fn test_basic_emission() {
    check_emit_ok("fn main() {}");
    check_emit_ok("fn main() { return; }");
    check_emit_ok("fn main() { let x: i32 = 42; let y: bool = true; }");
}

#[test]
fn test_determinism() {
    let src = "fn main() { let x: i32 = 42; let y: bool = true; }";
    let bytes1 = check_emit_ok(src);
    let bytes2 = check_emit_ok(src);
    assert_eq!(bytes1, bytes2, "emission should be deterministic");
}

#[test]
fn test_header_sanity() {
    let src = "fn main() {}";
    let bytes = check_emit_ok(src);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("header magic not recognized");
    assert!(spec.rev >= 1);
}

#[test]
fn test_feature_emission() {
    let src = r#"
        record R { x: i32, y: bool }
        enum E { A, B(i32) }
        fn main() {
            let a = 1 + 2;
            let b = 3 - 1;
            let c = 2 * 3;
            let d = 4 / 2;
            let e = 5 % 2;
            let f = -a;
            
            if true {
                let z = 1;
            } else {
                let z = 2;
            }
            
            while false {
                break;
            }
            
            loop {
                continue;
            }
            
            let r = R { x: 1, y: true };
            let rx = r.x;
            
            let ev1 = E::A;
            let ev2 = E::B(10);
            
            let m = match ev2 {
                E::A => { 1 }
                E::B(x) => { x }
            };
        }
    "#;
    check_emit_ok(src);
}

#[test]
fn test_option_result_emission() {
    let src = r#"
        fn main() {
            let o: Option(i32) = Option::Some(1);
            let o2: Option(i32) = Option::None;
            let r: Result(i32, i32) = Result::Ok(1);
            let r2: Result(i32, i32) = Result::Err(2);
            let m1 = match o {
                Option::Some(x) => { x }
                Option::None => { 0 }
            };
            let m2 = match r {
                Result::Ok(x) => { x }
                Result::Err(e) => { e }
            };
        }
    "#;
    check_emit_ok(src);
}
