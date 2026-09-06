"""#1726 design-only model, NOT a compiler/decoder/VM implementation.

Run: python tests/borrow_site_design_model.py
Sites are attached at construction, never inferred from instruction positions.
Execution tests supply explicit successful/failed instruction traces; they do not
claim end-to-end execution of the proposed V20 by today's Semantic VM.
"""

from dataclasses import dataclass, replace
import struct
import unittest


@dataclass(frozen=True)
class Instr:
    op: str
    name: str = ""
    site: int | None = None


@dataclass(frozen=True)
class Event:
    mode: str  # site / frame / write
    root: str = "p"
    site: int | None = None
    component: tuple[int, int] = (0, 0)  # TupleIndex / FieldSymbol


class Sites:
    def __init__(self):
        self.next = 0

    def new(self):
        if self.next == 2**32:
            raise ValueError("site overflow")
        result = self.next
        self.next += 1
        return result


def pairs(instrs, events):
    stores, borrows = {}, {}
    for instr in instrs:
        if instr.site is not None:
            if instr.op != "store" or instr.site in stores:
                raise ValueError("duplicate or non-StoreVar site")
            stores[instr.site] = instr.name
    for event in events:
        if event.mode == "site":
            if event.site is None or event.site in borrows:
                raise ValueError("missing or duplicate Borrow site")
            borrows[event.site] = event
        elif event.mode not in ("frame", "write") or event.site is not None:
            raise ValueError("invalid activation mode")
    if stores.keys() != borrows.keys():
        raise ValueError("unpaired site")
    return {site: (stores[site], borrows[site]) for site in stores}


def check_transition(before, instrs, events, removed=frozenset()):
    after = pairs(instrs, events)
    if not removed <= before.keys():
        raise ValueError("invalid unreachable removal receipt")
    if after != {site: pair for site, pair in before.items() if site not in removed}:
        raise ValueError("unjustified ownership change")


def check_origins(before, after, origins, unreachable_indices=frozenset()):
    """Pass-local input indices recorded at emission, never inferred afterward.

    This reduced cleanup preserves every retained instruction verbatim. Other
    production passes must check their own allowed rewrite at that input origin.
    """
    if len(origins) != len(after) or origins != sorted(set(origins)):
        raise ValueError("invalid instruction origins")
    if set(origins) & unreachable_indices or set(origins) | unreachable_indices != set(range(len(before))):
        raise ValueError("unjustified instruction loss")
    if any(before[origin] != instr for origin, instr in zip(origins, after)):
        raise ValueError("instruction changed at its origin")


def cleanup(instrs, events):
    """Models only the existing Ret/Jmp-to-next-Label unreachable proof."""
    before = pairs(instrs, events)  # Reject malformed input BEFORE erasing anything.
    result, removed, unreachable = [], set(), False
    origins, unreachable_indices = [], set()
    for origin, instr in enumerate(instrs):
        if instr.op == "label":
            unreachable = False
            result.append(instr)
            origins.append(origin)
        elif unreachable:
            unreachable_indices.add(origin)
            if instr.site is not None:
                removed.add(instr.site)  # Evidence from the deleted instruction.
        else:
            result.append(instr)
            origins.append(origin)
            unreachable = instr.op in ("return", "jump")
    events = [e for e in events if not (e.mode == "site" and e.site in removed)]
    check_origins(instrs, result, origins, unreachable_indices)
    check_transition(before, result, events, removed)
    return result, events


def emit(instrs, events):
    pairs(instrs, events)
    strings = list(dict.fromkeys(i.name for i in instrs if i.op == "store"))
    code, relocation, store_visits = bytearray(), {}, []
    # This reduced emitter supports the instruction widths used by the proofs.
    # Jump operands are placeholders: execution is exercised through traces.
    for instr in instrs:
        if instr.op == "label":
            continue
        pc = len(code)  # Authoritative emitted length, not a predicted offset.
        if instr.op == "store":
            sid = strings.index(instr.name)
            code.extend(struct.pack("<BHH", 0x05, sid, 0))
            if instr.site is not None:
                relocation[instr.site] = (pc, sid)
                store_visits.append((instr.site, pc))
        elif instr.op == "load":
            code.extend(struct.pack("<BHi", 0x03, 0, 1))
        elif instr.op == "return":
            code.extend(b"\x41\x00")
        elif instr.op == "jump":
            code.extend(struct.pack("<BI", 0x30, 0))
        else:
            raise ValueError("unsupported model instruction")
    records, event_index = [], {}
    for index, event in enumerate(events):
        root = strings.index(event.root)  # Strict lookup, no interning metadata.
        if event.mode == "site":
            pc, target = relocation[event.site]
            records.append((event.mode, pc, target, root, event.component))
            event_index[event.site] = index
        else:
            records.append((event.mode, None, None, root, event.component))
    # Reverse anchor records originate from instruction visits, not OWN0 PCs.
    anchors = [(event_index[site], pc) for site, pc in store_visits]
    wire = bytearray(b"OWN0") + struct.pack("<H", len(records))
    for mode, pc, target, root, (tag, component) in records:
        wire.append(1 if mode == "write" else 0)
        if mode != "write":
            wire.append(1 if mode == "site" else 0)
            if mode == "site":
                wire.extend(struct.pack("<II", pc, target))
        wire.extend(struct.pack("<IH", root, 1))
        wire.append(tag)
        wire.extend(struct.pack("<H" if tag == 0 else "<I", component))
    wire.extend(struct.pack("<H", len(anchors)))
    for index, pc in anchors:
        wire.extend(struct.pack("<HI", index, pc))
    return bytes(code), bytes(wire), records, anchors, strings, relocation


def decode_own0(wire):
    cursor = 0

    def read(fmt):
        nonlocal cursor
        size = struct.calcsize(fmt)
        if cursor + size > len(wire):
            raise ValueError("truncated OWN0")
        value = struct.unpack_from(fmt, wire, cursor)
        cursor += size
        return value[0] if len(value) == 1 else value

    if read("<4s") != b"OWN0":
        raise ValueError("missing OWN0")
    records = []
    for _ in range(read("<H")):
        kind = read("<B")
        if kind not in (0, 1):
            raise ValueError("unknown event kind")
        pc = target = None
        mode = "write"
        if kind == 0:
            marker = read("<B")
            if marker not in (0, 1):
                raise ValueError("unknown activation mode")
            mode = "site" if marker else "frame"
            if marker:
                pc, target = read("<II")
        root, count = read("<IH")
        if count != 1:
            raise ValueError("model expects one component")
        tag = read("<B")
        if tag not in (0, 1):
            raise ValueError("unsupported model path component")
        component = read("<H" if tag == 0 else "<I")
        records.append((mode, pc, target, root, (tag, component)))
    anchors = [read("<HI") for _ in range(read("<H"))]
    if cursor != len(wire):
        raise ValueError("trailing OWN0 model bytes")
    return records, anchors


def admit(code, records, anchors, strings):
    stores, pc = {}, 0
    widths = {0x05: 5, 0x03: 7, 0x41: 2, 0x30: 5}
    while pc < len(code):
        opcode = code[pc]
        if opcode not in widths or pc + widths[opcode] > len(code):
            raise ValueError("invalid model code")
        if opcode == 0x05:
            stores[pc] = struct.unpack_from("<H", code, pc + 1)[0]
        pc += widths[opcode]
    anchor_map, seen_pcs = {}, set()
    for index, pc in anchors:
        if index in anchor_map or pc in seen_pcs or not 0 <= index < len(records):
            raise ValueError("duplicate or out-of-range anchor")
        anchor_map[index] = pc
        seen_pcs.add(pc)
    for index, (mode, pc, target, root, _) in enumerate(records):
        if not 0 <= root < len(strings):
            raise ValueError("invalid root")
        if mode == "site":
            if target is None or not 0 <= target < len(strings):
                raise ValueError("invalid target")
            if pc not in stores or stores[pc] != target:
                raise ValueError("anchor is not matching StoreVar boundary")
            if anchor_map.pop(index, None) != pc:
                raise ValueError("stale or missing reverse anchor")
        elif pc is not None or target is not None or mode not in ("frame", "write"):
            raise ValueError("invalid activation record")
    if anchor_map:
        raise ValueError("anchor for FrameEntry or Write")


class Frame:
    def __init__(self, records):
        self.active = {i for i, r in enumerate(records) if r[0] == "frame"}
        self.pending = {r[1]: i for i, r in enumerate(records) if r[0] == "site"}

    def successful_store(self, pc):
        if pc in self.pending:
            self.active.add(self.pending.pop(pc))


class SiteDesignProof(unittest.TestCase):
    def example(self):
        ids = Sites()
        site = ids.new()
        instrs = [Instr("store", "p"), Instr("store", "x", site),
                  Instr("store", "x"), Instr("store", "x"), Instr("return")]
        return instrs, [Event("site", site=site)]

    def test_introduction_not_reassignments_and_loop_idempotence(self):
        code, wire, records, anchors, strings, relocation = emit(*self.example())
        self.assertEqual(decode_own0(wire), (records, anchors))
        admit(code, records, anchors, strings)
        frame = Frame(records)
        for pc in [10, 15, 10, 15]:  # Two other StoreVars with the SAME target.
            frame.successful_store(pc)
        self.assertFalse(frame.active)
        for _ in range(5):  # One static site executed on repeated loop visits.
            frame.successful_store(relocation[0][0])
        self.assertEqual(frame.active, {0})
        self.assertFalse(frame.pending)

    def test_deduplicated_first_site_is_required_for_every_later_ref(self):
        # Lowering emits one dynamic-root event for a sequential pattern such
        # as (ref a, ref b).  The later StoreVar is reachable only after the
        # first one in the same successful bind pass; no trace may execute b
        # while skipping a.  A split trace is rejected as an invalid lowering.
        first_pc, later_pc = 5, 10
        self.assertFalse(Frame([("site", first_pc, None, "p", (0, 0))]).active)
        frame = Frame([("site", first_pc, None, "p", (0, 0))])
        frame.successful_store(later_pc)
        self.assertFalse(frame.active)
        frame.successful_store(first_pc)
        self.assertEqual(frame.active, {0})

    def test_branch_traces_failure_and_frame_lifetime(self):
        records = emit(*self.example())[2]
        untaken, taken = Frame(records), Frame(records)
        # An untaken site or a failing StoreVar does not call successful_store.
        untaken.successful_store(15)
        self.assertFalse(untaken.active)
        taken.successful_store(5)
        taken.successful_store(15)
        self.assertEqual(taken.active, {0})
        # Leaving a lexical block does nothing; a new frame starts independently.
        self.assertEqual(Frame(records).active, set())

    def test_shadowing_repeated_paths_and_reordered_events(self):
        instrs = [Instr("store", "p"), Instr("store", "__local_1_x", 7),
                  Instr("store", "__local_2_x", 9), Instr("return")]
        events = [Event("site", site=9), Event("site", site=7)]
        code, wire, records, anchors, strings, relocation = emit(instrs, events)
        admit(code, *decode_own0(wire), strings)
        frame = Frame(records)
        frame.successful_store(relocation[7][0])
        self.assertEqual(frame.active, {1})
        frame.successful_store(relocation[9][0])
        self.assertEqual(frame.active, {0, 1})

    def test_deferred_record_bind_and_final_pc_relocation(self):
        ids = Sites()
        outer, inner = ids.new(), ids.new()
        # Outer metadata can precede else-return metadata; the outer StoreVar
        # is attached later. The identities were already reserved explicitly.
        events = [Event("site", site=outer, component=(1, 4)), Event("site", site=inner)]
        instrs = [Instr("store", "p"), Instr("jump"), Instr("load"),
                  Instr("label"), Instr("store", "inner", inner), Instr("return"),
                  Instr("label"), Instr("store", "outer", outer), Instr("return")]
        old = emit(instrs, events)
        optimized = cleanup(instrs, events)
        new = emit(*optimized)
        self.assertEqual(pairs(instrs, events), pairs(*optimized))
        self.assertEqual(old[5][outer][0] - new[5][outer][0], 7)
        admit(new[0], new[2], new[3], new[4])
        frame = Frame(new[2])
        frame.successful_store(new[5][inner][0])
        self.assertEqual(frame.active, {1})

    def test_unreachable_introduction_o0_valid_o1_removes_exact_pair(self):
        instrs = [Instr("store", "p"), Instr("return"), Instr("store", "x", 0)]
        events = [Event("site", site=0), Event("frame")]
        o0 = emit(instrs, events)
        admit(o0[0], o0[2], o0[3], o0[4])
        self.assertEqual(len(o0[3]), 1)
        optimized = cleanup(instrs, events)
        self.assertEqual(optimized, (instrs[:2], [events[1]]))
        o1 = emit(*optimized)
        admit(o1[0], o1[2], o1[3], o1[4])
        self.assertFalse(o1[3])

    def test_dead_reassignment_does_not_remove_live_borrow(self):
        instrs, events = self.example()
        instrs.insert(2, Instr("return"))
        optimized = cleanup(instrs, events)
        self.assertEqual(pairs(instrs, events), pairs(*optimized))
        self.assertEqual(len(optimized[1]), 1)

    def test_missing_duplicate_or_wrong_kind_ir_sites_fail_before_cleanup(self):
        instrs, events = self.example()
        malformed = [instrs[:1] + instrs[2:], instrs + [instrs[1]],
                     [replace(i, op="load") if i.site is not None else i for i in instrs]]
        for candidate in malformed:
            with self.assertRaises(ValueError):
                cleanup(candidate, events)
        for bad_events in [[], events * 2, [replace(events[0], site=99)], [Event("frame")]]:
            with self.assertRaises(ValueError):
                emit(instrs, bad_events)

    def test_silent_pair_loss_without_unreachable_proof_is_rejected(self):
        instrs, events = self.example()
        before = pairs(instrs, events)
        with self.assertRaisesRegex(ValueError, "unjustified"):
            check_transition(before, [i for i in instrs if i.site is None], [])

    def test_marker_transfer_to_same_target_reassignment_is_rejected(self):
        instrs, events = self.example()
        moved = list(instrs)
        moved[1] = replace(moved[1], site=None)
        moved[2] = replace(moved[2], site=0)
        # Bijection/conservation alone cannot detect this mutation.
        check_transition(pairs(instrs, events), moved, events)
        with self.assertRaisesRegex(ValueError, "origin"):
            check_origins(instrs, moved, list(range(len(instrs))))
        with self.assertRaisesRegex(ValueError, "origin"):
            check_origins(instrs, moved, [0, 2, 1, 3, 4])

    def test_wire_corruption_and_one_sided_stale_same_target_pc(self):
        code, wire, records, anchors, strings, _ = emit(*self.example())
        for truncated in range(len(wire)):
            with self.assertRaises(ValueError):
                decode_own0(wire[:truncated])
        bad = bytearray(wire)
        bad[7] = 2
        with self.assertRaises(ValueError):
            decode_own0(bad)
        for pc, target in [(6, 1), (999, 1), (20, 1), (0, 1), (5, 99), (10, 1)]:
            bad_records = [replace_record(records[0], pc=pc, target=target)]
            with self.assertRaises(ValueError):
                admit(code, bad_records, anchors, strings)
        for bad_anchors in [[], anchors * 2, [(0, 10)], [(9, 5)]]:
            with self.assertRaises(ValueError):
                admit(code, records, bad_anchors, strings)

    def test_coordinated_reauthoring_is_not_source_authentication(self):
        code, _, records, _, strings, _ = emit(*self.example())
        changed = [replace_record(records[0], pc=10)]
        # Explicit limitation: both records now consistently declare another
        # exact anchor. Neither unsigned tables nor opcodes authenticate source.
        admit(code, changed, [(0, 10)], strings)

    def test_frameentry_and_write_have_no_execution_anchors(self):
        instrs = [Instr("store", "p"), Instr("return")]
        code, wire, records, anchors, strings, _ = emit(instrs, [Event("frame"), Event("write")])
        admit(code, *decode_own0(wire), strings)
        self.assertFalse(anchors)
        self.assertEqual(Frame(records).active, {0})
        with self.assertRaises(ValueError):
            admit(code, records, [(0, 0)], strings)

    def test_site_allocation_overflow_fails_closed(self):
        ids = Sites()
        ids.next = 2**32
        with self.assertRaises(ValueError):
            ids.new()

    def test_semantic_mutants_are_killed_and_restored(self):
        original_init = Frame.__init__
        original_store = Frame.successful_store

        def eager(frame, records):
            original_init(frame, records)
            frame.active.update(frame.pending.values())
            frame.pending.clear()

        def wrong_pc(frame, pc):
            if frame.pending:
                frame.active.add(frame.pending.pop(next(iter(frame.pending))))

        mutants = [
            ("eager frame-entry activation", "__init__", eager),
            ("removed StoreVar activation", "successful_store", lambda frame, pc: None),
            ("activation at an unrelated StoreVar PC", "successful_store", wrong_pc),
        ]
        for name, method, replacement in mutants:
            try:
                setattr(Frame, method, replacement)
                result = unittest.TestResult()
                SiteDesignProof("test_introduction_not_reassignments_and_loop_idempotence").run(result)
                self.assertFalse(result.errors, name)  # A semantic assertion, not a harness error.
                self.assertTrue(result.failures, f"surviving mutant: {name}")
            finally:
                Frame.__init__ = original_init
                Frame.successful_store = original_store


def replace_record(record, **changes):
    names = ("mode", "pc", "target", "root", "component")
    return tuple(changes.get(name, value) for name, value in zip(names, record))


if __name__ == "__main__":
    unittest.main(verbosity=2)
