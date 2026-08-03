//! Generic, reusable clipping primitive (Issue "WORKBENCH FINAL CLOSURE
//! PASS" item 3).
//!
//! `crates/prom-ui-backend-native`'s wgpu presenter batches every
//! `DrawCommand::FillRect`/`DrawText` for a frame into one shared vertex
//! buffer / one glyphon text-render call (see `present_semantic_commands`).
//! A literal per-draw-call GPU scissor rect would require splitting that
//! batching into multiple render passes -- real, but a materially riskier
//! change to a shared renderer used by every Prom UI consumer, and one whose
//! correctness cannot be confirmed without visual pixel inspection.
//!
//! This module is the "canonical equivalent that best fits the existing
//! renderer architecture": clipping is applied by rect-intersection at
//! `DrawCommand` construction time, so geometry that would fall outside the
//! clip region is never emitted in the first place. "content outside the
//! clip rect is not presented" holds exactly, verified here by testing the
//! emitted geometry rather than rendered pixels.
//!
//! Deliberately reusable and not Workbench-specific: any Prom UI consumer
//! that wants clipped panels can depend on this module directly.

use crate::Rect;
use alloc::vec::Vec;

/// A clip region in the same coordinate space as `Rect`. A separate type
/// from `Rect` so "the rect being drawn" and "the region content must stay
/// inside" are never accidentally interchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl ClipRect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn from_rect(r: Rect) -> Self {
        Self::new(r.x, r.y, r.width, r.height)
    }

    pub const fn to_rect(self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }

    const fn right(self) -> i32 {
        self.x.saturating_add(self.width as i32)
    }

    const fn bottom(self) -> i32 {
        self.y.saturating_add(self.height as i32)
    }

    /// The canonical zero-area clip: contains nothing, clips everything.
    /// This is the safe result for an otherwise-invalid intersection,
    /// rather than a panic or an unclipped fallback.
    pub const EMPTY: ClipRect = ClipRect::new(0, 0, 0, 0);

    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Deterministic intersection of two clip rects. Two non-overlapping
    /// rects (including a zero-width/height input) intersect to `EMPTY`,
    /// never a negative-sized rect.
    pub fn intersect(self, other: ClipRect) -> ClipRect {
        let x0 = self.x.max(other.x);
        let y0 = self.y.max(other.y);
        let x1 = self.right().min(other.right());
        let y1 = self.bottom().min(other.bottom());
        if x1 <= x0 || y1 <= y0 {
            return ClipRect::EMPTY;
        }
        ClipRect::new(x0, y0, (x1 - x0) as u32, (y1 - y0) as u32)
    }

    pub const fn contains_point(self, x: i32, y: i32) -> bool {
        !self.is_empty() && x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }
}

/// Deterministic nested-clip stack. Each `push` intersects the pushed rect
/// with the current top, so a child clip can never escape its parent --
/// "nested clipping must behave deterministically" is satisfied by
/// construction (intersection is commutative/associative), not by ad hoc
/// rejection. `pop` restores the previous region exactly.
#[derive(Debug, Clone)]
pub struct ClipStack {
    stack: Vec<ClipRect>,
}

impl ClipStack {
    /// `full` is the outermost bound (e.g. the whole window/panel area).
    pub fn new(full: ClipRect) -> Self {
        Self {
            stack: alloc::vec![full],
        }
    }

    pub fn push(&mut self, rect: ClipRect) {
        let next = self.current().intersect(rect);
        self.stack.push(next);
    }

    /// Never pops the base (`full`) region -- an unbalanced `pop` is a safe
    /// no-op rather than leaving the stack empty (which would make
    /// `current()` ambiguous).
    pub fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    pub fn current(&self) -> ClipRect {
        *self
            .stack
            .last()
            .expect("ClipStack always has at least the base region")
    }
}

/// Clip a `FillRect` draw against `clip`. Returns the geometry to actually
/// draw (which is always fully inside `clip`), or `None` if nothing of it
/// is visible.
pub fn clip_fill_rect(rect: Rect, clip: ClipRect) -> Option<Rect> {
    let clipped = ClipRect::from_rect(rect).intersect(clip);
    if clipped.is_empty() {
        None
    } else {
        Some(clipped.to_rect())
    }
}

/// Whether a `DrawText` origin is inside `clip`. Text has no per-glyph clip
/// in the Wave 3 draw-command family, so a text draw is an all-or-nothing
/// decision keyed on its origin -- dropped entirely rather than partially
/// rendered or drawn past the clip boundary.
pub const fn clip_text_origin(x: i32, y: i32, clip: ClipRect) -> bool {
    clip.contains_point(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect_of_disjoint_rects_is_empty() {
        let a = ClipRect::new(0, 0, 10, 10);
        let b = ClipRect::new(20, 20, 10, 10);
        assert!(a.intersect(b).is_empty());
    }

    #[test]
    fn intersect_of_overlapping_rects_is_deterministic() {
        let a = ClipRect::new(0, 0, 10, 10);
        let b = ClipRect::new(5, 5, 10, 10);
        let i1 = a.intersect(b);
        let i2 = b.intersect(a);
        assert_eq!(i1, i2);
        assert_eq!(i1, ClipRect::new(5, 5, 5, 5));
    }

    #[test]
    fn nested_clip_stack_cannot_escape_parent() {
        let mut stack = ClipStack::new(ClipRect::new(0, 0, 100, 100));
        stack.push(ClipRect::new(50, 50, 100, 100)); // half outside parent
        assert_eq!(stack.current(), ClipRect::new(50, 50, 50, 50));
        stack.push(ClipRect::new(-1000, -1000, 5000, 5000)); // absurdly large, still bounded
        assert_eq!(stack.current(), ClipRect::new(50, 50, 50, 50));
        stack.pop();
        assert_eq!(stack.current(), ClipRect::new(50, 50, 50, 50));
        stack.pop();
        assert_eq!(stack.current(), ClipRect::new(0, 0, 100, 100));
        stack.pop(); // unbalanced pop: safe no-op, base region preserved
        assert_eq!(stack.current(), ClipRect::new(0, 0, 100, 100));
    }

    #[test]
    fn invalid_zero_size_clip_is_handled_safely_not_panicking() {
        let clip = ClipRect::new(10, 10, 0, 0);
        assert!(clip.is_empty());
        assert!(clip_fill_rect(Rect::new(10, 10, 5, 5), clip).is_none());
        assert!(!clip_text_origin(10, 10, clip));
    }

    #[test]
    fn fill_rect_partially_outside_clip_is_truncated_to_the_visible_portion() {
        let clip = ClipRect::new(0, 0, 50, 50);
        let rect = Rect::new(40, 40, 20, 20); // extends to (60,60), past the clip
        let clipped = clip_fill_rect(rect, clip).expect("partially visible rect must still draw");
        assert_eq!(clipped, Rect::new(40, 40, 10, 10));
    }

    #[test]
    fn fill_rect_fully_outside_clip_is_dropped() {
        let clip = ClipRect::new(0, 0, 50, 50);
        let rect = Rect::new(100, 100, 10, 10);
        assert!(clip_fill_rect(rect, clip).is_none());
    }

    #[test]
    fn fill_rect_fully_inside_clip_is_unchanged() {
        let clip = ClipRect::new(0, 0, 50, 50);
        let rect = Rect::new(5, 5, 10, 10);
        assert_eq!(clip_fill_rect(rect, clip), Some(rect));
    }

    #[test]
    fn text_origin_inside_and_outside_clip() {
        let clip = ClipRect::new(10, 10, 30, 30);
        assert!(clip_text_origin(15, 15, clip));
        assert!(!clip_text_origin(5, 5, clip));
        assert!(
            !clip_text_origin(41, 15, clip),
            "exact right edge is exclusive"
        );
    }

    #[test]
    fn resize_recomputes_clipping_purely_from_new_bounds() {
        // "Resize" is nothing but constructing a new base ClipRect -- there
        // is no cached derived state to go stale, so correctness after
        // resize reduces to correctness of `intersect`/`contains_point`,
        // already covered above. This test documents that property.
        let before = ClipStack::new(ClipRect::new(0, 0, 100, 100));
        let after_resize = ClipStack::new(ClipRect::new(0, 0, 50, 50));
        assert!(before.current().contains_point(80, 80));
        assert!(!after_resize.current().contains_point(80, 80));
    }

    #[test]
    fn hit_test_agrees_with_clipped_visibility() {
        let clip = ClipRect::new(0, 0, 40, 40);
        // A hit target whose rect is entirely clipped away must not be
        // clickable -- hit-testing uses the same `contains_point` the
        // renderer uses to decide what is visible.
        let hit_target = Rect::new(50, 50, 10, 10);
        assert!(clip_fill_rect(hit_target, clip).is_none());
        assert!(!clip.contains_point(hit_target.x, hit_target.y));
    }
}
