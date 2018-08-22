use Datelike;
use NaiveDate;
use Duration as OldDuration;
use date_ops::DateOp;

/// A Duration in days that can be added to Datelikes
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DayDuration(pub i32);

impl <T: Datelike> DateOp<T> for DayDuration {
    fn times(&self, n: i32) -> Option<Self> {
        Some(DayDuration(try_opt!(self.0.checked_mul(n))))
    }

    fn add_to(&self, dt: &T) -> Option<T> {
        // TODO: Is there a better way?
        let naive = try_opt!(NaiveDate::from_ymd(dt.year(), dt.month(), dt.day()).
            checked_add_signed(OldDuration::days(self.0.into())));
        dt.with_year(naive.year()).
            // set days to 1 so that month is always valid
            and_then(|dt| dt.with_day(1)).
            and_then(|dt| dt.with_month(naive.month())).
            and_then(|dt| dt.with_day(naive.day()))
    }
}