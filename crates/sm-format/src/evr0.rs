//! Q3.10-B2-1: untrusted, revision-gated EVR0 structural decoder.
//! No issuer authentication, type checking, or proof authority is conferred here.
use crate::semcode_decode::DecodeError;

pub(crate) const MAX_PAYLOAD: usize = 4 * 1024 * 1024;
const MAX_TEXT: usize = 4096;
const MAX_PROPOSITIONS: usize = 65_536;
const MAX_PROOFS: usize = 65_536;
const MAX_WITNESSES: usize = 262_144;
const MAX_PARENTS: usize = 262_144;
const MAX_DEPTH: u16 = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedEvidenceSection {
    pub source_set: [u8; 32],
    pub compiler_contract: [u8; 32],
    pub epoch: u16,
    pub policy_namespace: String,
    pub issuer_namespace: String,
    pub proposition_count: u32,
    pub proof_count: u32,
    pub witness_count: u32,
    pub parent_count: u32,
    pub payload_len: usize,
}

#[derive(Clone, Copy)]
struct Proof {
    proposition: u32,
    authority: u32,
    witness_first: usize,
    witness_count: usize,
    parent_first: usize,
    parent_count: usize,
}

struct Reader<'a> {
    input: &'a [u8],
    offset: usize,
    base: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        let end = self
            .offset
            .checked_add(n)
            .ok_or_else(|| self.bad("length overflow"))?;
        if end > self.input.len() {
            return Err(self.bad("truncated evidence field"));
        }
        let start = self.offset;
        self.offset = end;
        Ok(&self.input[start..end])
    }
    fn bad(&self, msg: &'static str) -> DecodeError {
        DecodeError::InvalidEvidenceSection {
            offset: self.base.saturating_add(self.offset),
            msg,
        }
    }
    fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("exact width"),
        ))
    }
    fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("exact width"),
        ))
    }
    fn digest(&mut self) -> Result<[u8; 32], DecodeError> {
        Ok(self.take(32)?.try_into().expect("exact width"))
    }
    fn text(&mut self) -> Result<String, DecodeError> {
        let len = self.u16()? as usize;
        if len == 0 || len > MAX_TEXT {
            return Err(self.bad("invalid evidence string length"));
        }
        let bytes = self.take(len)?;
        let text = std::str::from_utf8(bytes).map_err(|_| self.bad("invalid evidence UTF-8"))?;
        Ok(text.to_owned())
    }
    fn reserved(&mut self) -> Result<(), DecodeError> {
        if self.take(3)? != [0, 0, 0] {
            return Err(self.bad("nonzero evidence reserved bytes"));
        }
        Ok(())
    }
}

pub(crate) fn decode(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<DecodedEvidenceSection, DecodeError> {
    let start = *cursor;
    if bytes.get(start..start.saturating_add(4)) != Some(b"EVR0".as_slice()) {
        return Err(DecodeError::InvalidEvidenceSection {
            offset: start,
            msg: "missing mandatory EVR0 tag",
        });
    }
    let len_bytes: [u8; 4] = bytes
        .get(start + 4..start + 8)
        .ok_or(DecodeError::InvalidEvidenceSection {
            offset: start + 4,
            msg: "truncated EVR0 length",
        })?
        .try_into()
        .expect("exact width");
    let len = u32::from_le_bytes(len_bytes) as usize;
    if len == 0 || len > MAX_PAYLOAD {
        return Err(DecodeError::InvalidEvidenceSection {
            offset: start + 4,
            msg: "EVR0 payload exceeds limit or is empty",
        });
    }
    let base = start
        .checked_add(8)
        .ok_or(DecodeError::InvalidEvidenceSection {
            offset: start,
            msg: "EVR0 length overflow",
        })?;
    let end = base
        .checked_add(len)
        .ok_or(DecodeError::InvalidEvidenceSection {
            offset: base,
            msg: "EVR0 length overflow",
        })?;
    let payload = bytes
        .get(base..end)
        .ok_or(DecodeError::InvalidEvidenceSection {
            offset: base,
            msg: "EVR0 payload truncated",
        })?;
    let mut r = Reader {
        input: payload,
        offset: 0,
        base,
    };

    if r.u16()? != 1 || r.u16()? != 0 {
        return Err(r.bad("unsupported EVR0 schema or flags"));
    }
    let source_set = r.digest()?;
    let compiler_contract = r.digest()?;
    let epoch = r.u16()?;
    let policy_namespace = r.text()?;
    let issuer_namespace = r.text()?;
    let counts = [r.u32()?, r.u32()?, r.u32()?, r.u32()?];
    let [nprop, nproof, nwitness, nparent] = counts.map(|v| v as usize);
    if nprop > MAX_PROPOSITIONS
        || nproof > MAX_PROOFS
        || nwitness > MAX_WITNESSES
        || nparent > MAX_PARENTS
    {
        return Err(r.bad("EVR0 table count exceeds limit"));
    }
    // Reject impossible counts before reserving memory. The 14- and 34-byte
    // minima include the required nonempty Bytes16 length prefix.
    let minimum = nprop
        .checked_mul(14)
        .and_then(|a| nproof.checked_mul(34).and_then(|b| a.checked_add(b)))
        .and_then(|a| nwitness.checked_mul(84).and_then(|b| a.checked_add(b)))
        .and_then(|a| nparent.checked_mul(4).and_then(|b| a.checked_add(b)))
        .ok_or_else(|| r.bad("EVR0 count arithmetic overflow"))?;
    if minimum > payload.len() - r.offset {
        return Err(r.bad("EVR0 counts exceed remaining payload"));
    }

    let mut propositions = Vec::with_capacity(nprop);
    for _ in 0..nprop {
        let kind = r.text()?;
        let tuple = (kind, r.u32()?, r.u32()?, r.u32()?);
        if propositions.last().is_some_and(|prev| prev >= &tuple) {
            return Err(r.bad("EVR0 proposition order or duplicate"));
        }
        propositions.push(tuple);
    }
    let mut proofs = Vec::with_capacity(nproof);
    let mut witness_cursor = 0usize;
    let mut parent_cursor = 0usize;
    for i in 0..nproof {
        if r.u32()? != i as u32 {
            return Err(r.bad("EVR0 proof IDs must be contiguous"));
        }
        let proposition = r.u32()?;
        if proposition as usize >= nprop {
            return Err(r.bad("EVR0 unknown proof proposition"));
        }
        let _rule_id = r.text()?;
        let authority = r.u32()?;
        let witness_first = r.u32()? as usize;
        let witness_count = r.u32()? as usize;
        let parent_first = r.u32()? as usize;
        let parent_count = r.u32()? as usize;
        let operation = r.u8()?;
        r.reserved()?;
        if operation > 1 {
            return Err(r.bad("EVR0 unknown structural operation"));
        }
        if witness_first != witness_cursor || parent_first != parent_cursor {
            return Err(r.bad("EVR0 noncanonical proof table ranges"));
        }
        witness_cursor = witness_cursor
            .checked_add(witness_count)
            .ok_or_else(|| r.bad("EVR0 witness range overflow"))?;
        parent_cursor = parent_cursor
            .checked_add(parent_count)
            .ok_or_else(|| r.bad("EVR0 parent range overflow"))?;
        if witness_cursor > nwitness || parent_cursor > nparent {
            return Err(r.bad("EVR0 proof range out of bounds"));
        }
        proofs.push(Proof {
            proposition,
            authority,
            witness_first,
            witness_count,
            parent_first,
            parent_count,
        });
    }
    if witness_cursor != nwitness || parent_cursor != nparent {
        return Err(r.bad("EVR0 unowned witness or parent"));
    }
    for i in 0..nwitness {
        if r.u32()? != i as u32 {
            return Err(r.bad("EVR0 witness IDs must be contiguous"));
        }
        let proposition = r.u32()?;
        if proposition as usize >= nprop {
            return Err(r.bad("EVR0 witness proposition out of bounds"));
        }
        let polarity = r.u8()?;
        if polarity > 1 {
            return Err(r.bad("EVR0 witness polarity out of range"));
        }
        r.reserved()?;
        let _file = r.digest()?;
        let source_start = r.u32()?;
        let source_end = r.u32()?;
        let _origin = r.digest()?;
        if source_start > source_end {
            return Err(r.bad("EVR0 reversed witness span"));
        }
        // Source span digest membership must be evaluated by the trusted issuer,
        // NEVER inferred from these serialized bytes.
        let owner = proofs.binary_search_by(|p| {
            let end = p.witness_first + p.witness_count;
            if i < p.witness_first {
                std::cmp::Ordering::Greater
            } else if i >= end {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        });
        let owner = owner.map_err(|_| r.bad("EVR0 unowned witness"))?;
        if proofs[owner].proposition != proposition {
            return Err(r.bad("EVR0 witness/proof identity mismatch"));
        }
    }
    let mut depths = vec![0u16; nproof];
    for (i, p) in proofs.iter().enumerate() {
        let mut previous: Option<u32> = None;
        let mut max_parent_depth = 0u16;
        for _ in 0..p.parent_count {
            let parent = r.u32()?;
            if parent as usize >= i || previous.is_some_and(|prev| parent <= prev) {
                return Err(r.bad("EVR0 invalid parent ordering or cycle"));
            }
            let ancestor = proofs[parent as usize];
            if ancestor.proposition != p.proposition || ancestor.authority != p.authority {
                return Err(r.bad("EVR0 cross-proposition/authority parent"));
            }
            max_parent_depth = max_parent_depth.max(depths[parent as usize]);
            previous = Some(parent);
        }
        // Parent ownership is the contiguous range checked above. The current
        // reader position is guaranteed to match it (no index-based seeking).
        if p.parent_first + p.parent_count > nparent {
            return Err(r.bad("EVR0 parent range out of bounds"));
        }
        let depth = max_parent_depth
            .checked_add(1)
            .ok_or_else(|| r.bad("EVR0 depth overflow"))?;
        if depth > MAX_DEPTH {
            return Err(r.bad("EVR0 proof depth exceeded"));
        }
        depths[i] = depth;
    }
    if r.offset != payload.len() {
        return Err(r.bad("EVR0 trailing bytes"));
    }
    *cursor = end;
    Ok(DecodedEvidenceSection {
        source_set,
        compiler_contract,
        epoch,
        policy_namespace,
        issuer_namespace,
        proposition_count: counts[0],
        proof_count: counts[1],
        witness_count: counts[2],
        parent_count: counts[3],
        payload_len: len,
    })
}
