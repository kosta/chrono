use Datelike;
use date_ops::DateOp;

/// A Duration in years that can be added to Datelikes
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct YearDuration(pub i32);

impl <T: Datelike> DateOp<T> for YearDuration {
    fn times(&self, n: i32) -> Option<Self> {
        Some(YearDuration(try_opt!(self.0.checked_mul(n))))
    }

    fn add_to(&self, dt: &T) -> Option<T> {
        dt.with_year(try_opt!(dt.year().checked_add(self.0)))
    }
}