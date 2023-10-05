use crate::algorithms::approx::ApproxPoint;

/// Points of a curve approximation
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct CurveApproxPoints {
    /// Points of a curve approximation
    pub inner: Vec<ApproxPoint<1>>,
}

impl CurveApproxPoints {
    /// Reverse the orientation of the approximation
    pub fn reverse(&mut self) -> &mut Self {
        self.inner.reverse();
        self
    }
}
