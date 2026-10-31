//! EVR0 structural decoder (Q3.10 B2-1). This is untrusted data, NOT proof admission.
//! All counts are checked before allocation, all byte reads are bounded by the
//! declared section payload, and no issuer/authority is minted by parsing.

pub const MAX_EVR0_PAYLOAD: usize = 4 * 1024 * 1024;
pub const MAX_EVR0_STRING_BYTES: usize = 4096;
pub const MAX_EVR0_PROPOSITIONS: usize = 65_536;
pub const MAX_EVR0_PROOFS: usize = 65_536;
pub const MAX_EVR0_WITNESSES: usize = 262_144;
pub const MAX_EVR0_PARENTS: usize = 262_144;
pub const MAX_EVR0_DEPTH: usize = 256;
/// Conservative logical allocation budget for decoded fixed-size records and
/// retained text. It does not count the input artifact (borrowed from caller).
pub const MAX_EVR0_LOGICAL_ALLOC: usize = 16 * 1024 * 1024;
/// Deterministic work budget: one unit per input payload byte and per
/// fixed-size record examined or graph edge traversed.
pub const MAX_EVR0_WORK_UNITS: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Evr0ErrorKind {
    Invalid,
    ResourceLimit,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evr0Error {
    pub kind: Evr0ErrorKind,
    pub offset: usize,
    pub message: &'static str,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposition {
    pub kind: String,
    pub ast_owner_id: u32,
    pub semantic_scope_id: u32,
    pub type_obligation_id: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    pub proof_id: u32,
    pub proposition_index: u32,
    pub rule_id: String,
    pub authority_context: u32,
    pub witness_first: u32,
    pub witness_count: u32,
    pub parent_first: u32,
    pub parent_count: u32,
    pub authorized_operation: u8,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness {
    pub witness_id: u32,
    pub proposition_index: u32,
    pub polarity: u8,
    pub file_digest: [u8; 32],
    pub source_start: u32,
    pub source_end: u32,
    pub origin_digest: [u8; 32],
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedEvr0 {
    pub source_set: [u8; 32],
    pub compiler_contract: [u8; 32],
    pub epoch: u16,
    pub policy_namespace: String,
    pub issuer_namespace: String,
    pub propositions: Vec<Proposition>,
    pub proofs: Vec<Proof>,
    pub witnesses: Vec<Witness>,
    pub parents: Vec<u32>,
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    base: usize,
}
impl<'a> Reader<'a> {
    fn fail(&self, message: &'static str) -> Evr0Error {
        Evr0Error {
            kind: Evr0ErrorKind::Invalid,
            offset: self.base + self.pos,
            message,
        }
    }
    fn limit(&self, message: &'static str) -> Evr0Error {
        Evr0Error {
            kind: Evr0ErrorKind::ResourceLimit,
            offset: self.base + self.pos,
            message,
        }
    }
    fn fail_at(&self, position: usize, message: &'static str) -> Evr0Error {
        Evr0Error {
            kind: Evr0ErrorKind::Invalid,
            offset: self.base + position,
            message,
        }
    }
    fn limit_at(&self, position: usize, message: &'static str) -> Evr0Error {
        Evr0Error {
            kind: Evr0ErrorKind::ResourceLimit,
            offset: self.base + position,
            message,
        }
    }
    fn take(&mut self, length: usize) -> Result<&'a [u8], Evr0Error> {
        let end = self
            .pos
            .checked_add(length)
            .ok_or_else(|| self.fail("EVR0 offset overflow"))?;
        if end > self.data.len() {
            return Err(self.fail("EVR0 truncated field"));
        }
        let start = self.pos;
        self.pos = end;
        Ok(&self.data[start..end])
    }
    fn u8(&mut self) -> Result<u8, Evr0Error> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, Evr0Error> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("fixed read"),
        ))
    }
    fn u32(&mut self) -> Result<u32, Evr0Error> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("fixed read"),
        ))
    }
    fn digest(&mut self) -> Result<[u8; 32], Evr0Error> {
        Ok(self.take(32)?.try_into().expect("fixed read"))
    }
    fn text(&mut self) -> Result<String, Evr0Error> {
        let length = self.u16()? as usize;
        if length == 0 || length > MAX_EVR0_STRING_BYTES {
            return Err(self.fail("EVR0 empty or overlong text field"));
        }
        let data = self.take(length)?;
        let value = core::str::from_utf8(data).map_err(|_| self.fail("EVR0 invalid UTF-8"))?;
        if value.chars().any(char::is_control) {
            return Err(self.fail("EVR0 control character in text field"));
        }
        let mut owned = String::new();
        owned
            .try_reserve_exact(length)
            .map_err(|_| self.limit("EVR0 text reservation failed"))?;
        owned.push_str(value);
        Ok(owned)
    }
    fn zeros(&mut self) -> Result<(), Evr0Error> {
        if self.take(3)? != [0, 0, 0] {
            return Err(self.fail("EVR0 nonzero reserved bytes"));
        }
        Ok(())
    }
}

fn tuple_order(left: &Proposition, right: &Proposition) -> core::cmp::Ordering {
    (
        &left.kind,
        left.ast_owner_id,
        left.semantic_scope_id,
        left.type_obligation_id,
    )
        .cmp(&(
            &right.kind,
            right.ast_owner_id,
            right.semantic_scope_id,
            right.type_obligation_id,
        ))
}

/// Decode exactly one mandatory EVR0 section at *cursor. Structural success
/// never authenticates the source, compiler contract, issuer, or witnesses.
pub fn decode_evr0_section(bytes: &[u8], cursor: &mut usize) -> Result<DecodedEvr0, Evr0Error> {
    let start = *cursor;
    let invalid = |at: usize, message| Evr0Error {
        kind: Evr0ErrorKind::Invalid,
        offset: at,
        message,
    };
    if bytes.get(start..start.saturating_add(4)) != Some(b"EVR0".as_slice()) {
        return Err(invalid(start, "missing mandatory EVR0 section"));
    }
    let len_at = start
        .checked_add(4)
        .ok_or_else(|| invalid(start, "EVR0 header overflow"))?;
    let header_end = len_at
        .checked_add(4)
        .ok_or_else(|| invalid(start, "EVR0 header overflow"))?;
    let len_bytes = bytes
        .get(len_at..header_end)
        .ok_or_else(|| invalid(len_at, "EVR0 length truncated"))?;
    let payload_len = u32::from_le_bytes(len_bytes.try_into().expect("fixed read")) as usize;
    if payload_len == 0 || payload_len > MAX_EVR0_PAYLOAD {
        return Err(Evr0Error {
            kind: Evr0ErrorKind::ResourceLimit,
            offset: len_at,
            message: "EVR0 payload exceeds bounded section budget or is empty",
        });
    }
    let payload_end = header_end
        .checked_add(payload_len)
        .ok_or_else(|| invalid(len_at, "EVR0 payload end overflow"))?;
    let payload = bytes
        .get(header_end..payload_end)
        .ok_or_else(|| invalid(header_end, "EVR0 payload truncated"))?;
    let mut r = Reader {
        data: payload,
        pos: 0,
        base: header_end,
    };
    if r.u16()? != 1 {
        return Err(r.fail("unsupported EVR0 schema"));
    }
    if r.u16()? != 0 {
        return Err(r.fail("EVR0 flags must be zero"));
    }
    let source_set = r.digest()?;
    let compiler_contract = r.digest()?;
    let epoch = r.u16()?;
    let policy_namespace = r.text()?;
    let issuer_namespace = r.text()?;
    let counts_start = r.pos;
    let counts = [
        r.u32()? as usize,
        r.u32()? as usize,
        r.u32()? as usize,
        r.u32()? as usize,
    ];
    for (index, (count, limit)) in counts
        .iter()
        .zip([
            MAX_EVR0_PROPOSITIONS,
            MAX_EVR0_PROOFS,
            MAX_EVR0_WITNESSES,
            MAX_EVR0_PARENTS,
        ])
        .enumerate()
    {
        if *count > limit {
            return Err(r.limit_at(counts_start + index * 4, "EVR0 table count exceeds limit"));
        }
    }
    // Reject impossible counts before allocating ANY vector.
    let min_bytes = [15usize, 35, 84, 4];
    let mut minimum = 0usize;
    for (count, size) in counts.iter().zip(min_bytes) {
        minimum = minimum
            .checked_add(
                count
                    .checked_mul(size)
                    .ok_or_else(|| r.limit("EVR0 table size overflow"))?,
            )
            .ok_or_else(|| r.limit("EVR0 table sum overflow"))?;
    }
    if minimum > payload.len() - r.pos {
        return Err(r.fail("EVR0 count exceeds remaining payload"));
    }
    let sizes = [
        core::mem::size_of::<Proposition>(),
        core::mem::size_of::<Proof>(),
        core::mem::size_of::<Witness>(),
        core::mem::size_of::<u32>(),
    ];
    // Depth tracking is allocated separately; include it in the same budget.
    let depth_bytes = counts[1]
        .checked_mul(core::mem::size_of::<usize>())
        .ok_or_else(|| r.limit("EVR0 depth tracking size overflow"))?;
    let mut allocation = payload_len
        .checked_add(depth_bytes)
        .ok_or_else(|| r.limit("EVR0 depth allocation budget overflow"))?;
    for (count, size) in counts.iter().zip(sizes) {
        allocation = allocation
            .checked_add(
                count
                    .checked_mul(size)
                    .ok_or_else(|| r.limit("EVR0 allocation size overflow"))?,
            )
            .ok_or_else(|| r.limit("EVR0 allocation sum overflow"))?;
    }
    if allocation > MAX_EVR0_LOGICAL_ALLOC {
        return Err(r.limit("EVR0 allocation budget exceeded"));
    }
    let work = payload_len
        .checked_add(counts.iter().sum::<usize>())
        .ok_or_else(|| r.limit("EVR0 work units overflow"))?;
    if work > MAX_EVR0_WORK_UNITS {
        return Err(r.limit("EVR0 work budget exceeded"));
    }

    // Resource limits bound requests but cannot guarantee that the host has
    // memory. Fallible reservations report a resource rejection when possible.
    let mut propositions = Vec::new();
    propositions
        .try_reserve_exact(counts[0])
        .map_err(|_| r.limit("EVR0 proposition reservation failed"))?;
    for _ in 0..counts[0] {
        let record_start = r.pos;
        let item = Proposition {
            kind: r.text()?,
            ast_owner_id: r.u32()?,
            semantic_scope_id: r.u32()?,
            type_obligation_id: r.u32()?,
        };
        if propositions
            .last()
            .is_some_and(|last| tuple_order(last, &item) != core::cmp::Ordering::Less)
        {
            return Err(r.fail_at(record_start, "EVR0 proposition table not strictly ordered"));
        }
        propositions.push(item);
    }
    let mut proofs = Vec::new();
    proofs
        .try_reserve_exact(counts[1])
        .map_err(|_| r.limit("EVR0 proof reservation failed"))?;
    let mut witness_cursor = 0usize;
    let mut parent_cursor = 0usize;
    for id in 0..counts[1] {
        let record_start = r.pos;
        let item = Proof {
            proof_id: r.u32()?,
            proposition_index: r.u32()?,
            rule_id: r.text()?,
            authority_context: r.u32()?,
            witness_first: r.u32()?,
            witness_count: r.u32()?,
            parent_first: r.u32()?,
            parent_count: r.u32()?,
            authorized_operation: r.u8()?,
        };
        r.zeros()?;
        if item.proof_id as usize != id || item.proposition_index as usize >= counts[0] {
            return Err(r.fail_at(record_start, "EVR0 proof identity or order invalid"));
        }
        if item.witness_first as usize != witness_cursor
            || item.parent_first as usize != parent_cursor
        {
            return Err(r.fail_at(
                record_start,
                "EVR0 witness or parent ranges are not contiguous",
            ));
        }
        witness_cursor = witness_cursor
            .checked_add(item.witness_count as usize)
            .ok_or_else(|| r.limit("EVR0 witness range overflow"))?;
        parent_cursor = parent_cursor
            .checked_add(item.parent_count as usize)
            .ok_or_else(|| r.limit("EVR0 parent range overflow"))?;
        if witness_cursor > counts[2] || parent_cursor > counts[3] {
            return Err(r.fail_at(record_start, "EVR0 proof range exceeds table"));
        }
        let shape = match item.rule_id.as_str() {
            "POS-WITNESS-1" | "NEG-WITNESS-1" => (1, 0, 0),
            "JOIN-SAME-P-1" => (0, 2, 0),
            "OP-T-ONLY-1" => (0, 1, 1),
            _ => return Err(r.fail_at(record_start, "EVR0 unknown proof rule")),
        };
        if (
            item.witness_count,
            item.parent_count,
            item.authorized_operation,
        ) != shape
        {
            return Err(r.fail_at(record_start, "EVR0 invalid proof rule shape"));
        }
        proofs.push(item);
    }
    if witness_cursor != counts[2] || parent_cursor != counts[3] {
        return Err(r.fail("EVR0 witness or parent table not fully owned"));
    }
    let mut witnesses = Vec::new();
    witnesses
        .try_reserve_exact(counts[2])
        .map_err(|_| r.limit("EVR0 witness reservation failed"))?;
    for id in 0..counts[2] {
        let record_start = r.pos;
        let item = Witness {
            witness_id: r.u32()?,
            proposition_index: r.u32()?,
            polarity: r.u8()?,
            file_digest: {
                r.zeros()?;
                r.digest()?
            },
            source_start: r.u32()?,
            source_end: r.u32()?,
            origin_digest: r.digest()?,
        };
        if item.witness_id as usize != id
            || item.proposition_index as usize >= counts[0]
            || item.polarity > 1
            || item.source_start > item.source_end
        {
            return Err(r.fail_at(
                record_start,
                "EVR0 witness identity, polarity or source span invalid",
            ));
        }
        witnesses.push(item);
    }
    let parent_table_start = r.pos;
    let mut parents = Vec::new();
    parents
        .try_reserve_exact(counts[3])
        .map_err(|_| r.limit("EVR0 parent reservation failed"))?;
    for _ in 0..counts[3] {
        parents.push(r.u32()?);
    }
    if r.pos != payload.len() {
        return Err(r.fail("EVR0 payload has trailing bytes"));
    }

    // Deterministic single forward pass: every parent must precede its child.
    let mut depth = Vec::new();
    depth
        .try_reserve_exact(proofs.len())
        .map_err(|_| r.limit("EVR0 depth reservation failed"))?;
    depth.resize(proofs.len(), 0usize);
    for proof in &proofs {
        let id = proof.proof_id as usize;
        let mut max_parent_depth = 0usize;
        let begin = proof.parent_first as usize;
        let end = begin + proof.parent_count as usize; // already checked <= counts[3]
        let mut previous = None;
        for (index, &parent_id) in parents[begin..end].iter().enumerate() {
            let parent = parent_id as usize;
            if parent >= id || previous.is_some_and(|p| p >= parent_id) {
                return Err(Evr0Error {
                    kind: Evr0ErrorKind::Invalid,
                    offset: r.base + parent_table_start + (begin + index) * 4,
                    message: "EVR0 invalid or noncanonical parent reference",
                });
            }
            if proofs[parent].proposition_index != proof.proposition_index
                || proofs[parent].authority_context != proof.authority_context
            {
                return Err(Evr0Error {
                    kind: Evr0ErrorKind::Invalid,
                    offset: r.base + parent_table_start + (begin + index) * 4,
                    message: "EVR0 parent identity or authority mismatch",
                });
            }
            max_parent_depth = max_parent_depth.max(depth[parent]);
            previous = Some(parent_id);
        }
        depth[id] = max_parent_depth + 1;
        if depth[id] > MAX_EVR0_DEPTH {
            return Err(Evr0Error {
                kind: Evr0ErrorKind::ResourceLimit,
                offset: header_end,
                message: "EVR0 derivation depth limit exceeded",
            });
        }
        let start_wit = proof.witness_first as usize;
        let end_wit = start_wit + proof.witness_count as usize;
        for witness in &witnesses[start_wit..end_wit] {
            if witness.proposition_index != proof.proposition_index {
                return Err(Evr0Error {
                    kind: Evr0ErrorKind::Invalid,
                    offset: header_end,
                    message: "EVR0 witness and proof proposition mismatch",
                });
            }
            if (proof.rule_id == "POS-WITNESS-1" && witness.polarity != 1)
                || (proof.rule_id == "NEG-WITNESS-1" && witness.polarity != 0)
            {
                return Err(Evr0Error {
                    kind: Evr0ErrorKind::Invalid,
                    offset: header_end,
                    message: "EVR0 witness polarity contradicts rule",
                });
            }
        }
    }
    *cursor = payload_end;
    Ok(DecodedEvr0 {
        source_set,
        compiler_contract,
        epoch,
        policy_namespace,
        issuer_namespace,
        propositions,
        proofs,
        witnesses,
        parents,
    })
}
