#!/usr/bin/env bash
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

echo "========================================="
echo "  7hell PCC Qualification Mini-Runner    "
echo "========================================="
echo ""

assert_success() {
    if [ $? -ne 0 ]; then
        echo -e "${RED}FAIL: $1${NC}"
        exit 1
    fi
}

# -----------------------------------------------------------------------------
# Hell 1
echo -e "${CYAN}[ Hell 1 ] Workspace Health...${NC}"
cargo fmt --check
assert_success "cargo fmt failed"
cargo check --workspace --all-features
assert_success "cargo check failed"
cargo test --workspace --all-features
assert_success "cargo test failed"
echo -e "${GREEN}PASS: Hell 1${NC}"

# -----------------------------------------------------------------------------
# Hell 2
echo -e "\n${CYAN}[ Hell 2 ] Trust Boundary Guards...${NC}"
cargo test -p semantic_language --test trust_boundary_guards
assert_success "Trust boundary guards test failed"

if cargo tree --edges normal -p sm-vm 2>&1 | grep -E "sm-ir|sm-emit|prom-ui" > /dev/null; then
    echo -e "${RED}FAIL: sm-vm depends on higher level crates (sm-ir, sm-emit, or prom-ui)${NC}"
    exit 1
fi

if cargo tree --edges normal -p sm-verify 2>&1 | grep -E "sm-ir|sm-emit" > /dev/null; then
    echo -e "${RED}FAIL: sm-verify depends on higher level crates (sm-ir or sm-emit)${NC}"
    exit 1
fi

if cargo tree --edges normal -p sm-format 2>&1 | grep -E "sm-ir" > /dev/null; then
    echo -e "${RED}FAIL: sm-format depends on sm-ir${NC}"
    exit 1
fi
echo -e "${GREEN}PASS: Hell 2${NC}"

# -----------------------------------------------------------------------------
# Hell 3
echo -e "\n${CYAN}[ Hell 3 ] SemCode Format Authority...${NC}"
cargo test -p sm-format --all-features
assert_success "sm-format tests failed"

if grep -rE "sm_ir::semcode_decode|sm_ir::semcode_format" crates > /dev/null 2>&1; then
    echo -e "${RED}FAIL: Leakage of sm_ir decoding/formatting found in crates!${NC}"
    exit 1
fi
if grep -rE "sm_emit::semcode_format" crates > /dev/null 2>&1; then
    echo -e "${RED}FAIL: Leakage of sm_emit formatting found in crates!${NC}"
    exit 1
fi
echo -e "${GREEN}PASS: Hell 3${NC}"

# -----------------------------------------------------------------------------
# Hell 4
echo -e "\n${CYAN}[ Hell 4 ] Verifier Negative Corpus...${NC}"
cargo test -p sm-verify --all-features --features sm-ir/profile-rust
assert_success "sm-verify tests failed"
echo -e "${GREEN}PASS: Hell 4${NC}"

# -----------------------------------------------------------------------------
# Hell 5
echo -e "\n${CYAN}[ Hell 5 ] VM Ownership Semantics...${NC}"
cargo test -p sm-vm --all-features
assert_success "sm-vm tests failed"
echo -e "${GREEN}PASS: Hell 5${NC}"

# -----------------------------------------------------------------------------
# Hell 6
echo -e "\n${CYAN}[ Hell 6 ] Source to SemCode Smoke...${NC}"
mkdir -p target/7hell
cargo run --bin smc -- compile crates/sm-front/tests/adt_match_local.sm -o target/7hell/adt_match_local.smc
assert_success "Source compilation smoke test failed"
cargo run --bin smc -- compile tests/fixtures/pcc6_option_result_diagnostics/positive_option_result_ownership.sm -o target/7hell/positive_option_result_ownership.smc
assert_success "Option/Result compilation smoke test failed"
cargo run --bin smc -- compile tests/fixtures/pcc4_records/positive_record_field_ownership.sm -o target/7hell/positive_record_field_ownership.smc
assert_success "Record field compilation smoke test failed"
cargo run --bin smc -- compile tests/fixtures/pcc_tuple_ownership/positive_tuple_ownership.sm -o target/7hell/positive_tuple_ownership.smc
assert_success "Tuple ownership compilation smoke test failed"
cargo test --test cli_public_smoke_matrix
assert_success "Public CLI smoke matrix test failed"
rm -f target/7hell/adt_match_local.smc
rm -f target/7hell/positive_option_result_ownership.smc
rm -f target/7hell/positive_record_field_ownership.smc
rm -f target/7hell/positive_tuple_ownership.smc
echo -e "${GREEN}PASS: Hell 6${NC}"

# -----------------------------------------------------------------------------
# Hell 7
echo -e "\n${CYAN}[ Hell 7 ] PCC Documentation Integrity...${NC}"
for doc in "docs/roadmap/pcc/adt_payload_ownership_matrix.md" \
           "docs/roadmap/pcc/adt_payload_ownership_slice_closeout.md" \
           "docs/architecture/adt_payload_ownership_paths.md"; do
    if [ ! -f "$doc" ]; then
        echo -e "${RED}FAIL: Missing documentation file: $doc${NC}"
        exit 1
    fi
done
echo -e "${GREEN}PASS: Hell 7${NC}"

echo -e "\n========================================="
echo -e "  ${GREEN}ALL 7 GATES PASSED!${NC}                    "
echo -e "========================================="
