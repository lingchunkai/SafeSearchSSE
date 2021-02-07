/// Bounds attached to either sequences or infosets. Use `None` if non existent.
#[derive(Debug, Clone, Copy)]
pub enum ValueBound {
    UpperBound(f64),
    LowerBound(f64),
    None,
}
