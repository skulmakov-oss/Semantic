use sm_ir::compile_program_to_ir;

fn check_lower_ok(src: &str) {
    let ir = compile_program_to_ir(src).expect("lowering failed");
    assert!(!ir.is_empty(), "IR should be non-empty");
}

#[test]
fn test_primitive_lowering() {
    let src = r#"
        fn get_quad() -> quad { return get_quad(); }
        fn get_f64() -> f64 { return get_f64(); }
        fn main() {
            let q: quad = get_quad();
            let b: bool = true;
            let i: i32 = 42;
            let f: f64 = get_f64();
            return;
        }
    "#;
    check_lower_ok(src);
}

#[test]
fn test_arithmetic_lowering() {
    let src = r#"
        fn main() {
            let x: i32 = 10;
            let a = x + 1;
            let b = x - 1;
            let c = x * 2;
            let d = x / 2;
            let e = x % 3;
            let f = -x;
        }
    "#;
    check_lower_ok(src);
}

#[test]
fn test_control_flow_lowering() {
    let src = r#"
        fn main() {
            let x = true;
            if x {
                let y = 1;
            } else {
                let y = 2;
            }
            while x {
                break;
            }
            loop {
                continue;
            }
        }
    "#;
    check_lower_ok(src);
}

#[test]
fn test_record_lowering() {
    let src = r#"
        record R { x: i32, y: bool }
        fn main() {
            let r = R { x: 1, y: true };
            let a = r.x;
            let b = r.y;
        }
    "#;
    check_lower_ok(src);
}

#[test]
fn test_adt_match_lowering() {
    let src = r#"
        enum E { A, B(i32) }
        fn main() {
            let e1 = E::A;
            let e2 = E::B(42);
            let m = match e2 {
                E::A => { 1 }
                E::B(x) => { x }
            };
        }
    "#;
    check_lower_ok(src);
}

#[test]
fn test_option_result_lowering() {
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
    check_lower_ok(src);
}
