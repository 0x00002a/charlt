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
        assert!(other.signum() == self.signum());
        let mut rs = other;
        while rs.abs() < self.abs() && self % other != 0.0 {
            rs += other;
        }
        rs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ceil_mul() {
        let muls = vec![10.0, -10.0, 5.0];
        let inputs = vec![9.0, -13.0, 12.0];
        let expected = vec![10.0, -20.0, 15.0];
        for i in 0..3 {
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
