use crate::interaction::ElementId;

/// A pure interface for resolving geometric coordinates back into logical elements
/// based on the deterministic layout projection.
///
/// Implementors of this trait (e.g. a LayoutGeometryModel solver) can definitively
/// state which element, if any, is physically located at the given `(x, y)` coordinates.
pub trait UiHitTest {
    fn hit_test(&self, x: f64, y: f64) -> Option<ElementId>;
}
