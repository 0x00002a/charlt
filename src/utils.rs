use std::ops::{Deref, DerefMut};

pub trait Holds {
    type Item;
    fn map<O, T, F: FnOnce(Self::Item) -> T>(self, f: F) -> O
    where
        O: Holds<Item = T>;
    fn into_value(self) -> Self::Item;
}

pub trait RoundMul<T> {
    fn ceil_mul(self, other: T) -> Self;
}
impl RoundMul<f64> for f64 {
    fn ceil_mul(self, other: f64) -> Self {
        if self % other == 0.0 {
            self
        } else if self.abs() > other.abs() {
            (other * (self / other).floor()) + (other % self)
        } else {
            self + (other % self)
        }
    }
}

struct BezWrapper(kurbo::BezPath);

impl From<BezWrapper> for kurbo::BezPath {
    fn from(r: BezWrapper) -> Self {
        r.0
    }
}
impl From<kurbo::BezPath> for BezWrapper {
    fn from(p: kurbo::BezPath) -> Self {
        Self(p)
    }
}
impl Deref for BezWrapper {
    type Target = kurbo::BezPath;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BezWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<kurbo::Point> for BezWrapper {
    fn from_iter<T: IntoIterator<Item = kurbo::Point>>(iter: T) -> Self {
        BezWrapper(
            iter.into_iter()
                .fold(kurbo::BezPath::default(), |mut path, p| {
                    if path.elements().len() == 0 {
                        path.move_to(p);
                    } else {
                        path.line_to(p);
                    }
                    path
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ceil_mul() {
        let muls = vec![10.0, -10.0, 5.0, 1.0];
        let inputs = vec![9.0, -13.0, 12.0, 10.0];
        let expected = vec![10.0, -20.0, 15.0, 10.0];
        for i in 0..expected.len() {
            assert_eq!(
                inputs[i].ceil_mul(muls[i]),
                expected[i],
                "({} -> {}) should be {}",
                inputs[i],
                muls[i],
                expected[i]
            );
        }
    }
}
