use std::fmt::Debug;

use super::DateOp;

use Datelike;
use last_day_of_month_0;

/// InvalidDateHandling specifies what operations that result in invalid dates should return instead
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvalidDateHandling {
    /// Return `None` instead of an invalid date
    Reject,
    /// Return the closest previous valid date instead of an invalid one (e.g. April 30th instead of April 31st)
    Previous,
    /// Return the closest subsequent valid date instead of an invalid one (e.g. May 1st instead of April 31st)
    Next,
}

/// A Duration in months that can be added to Datelikes
/// Note: Adding months can be a bit weird, because they have varying length. While
/// you can always add a month to e.g. 2017-05-01, you cannot "sanely" add a month
/// to e.g. 2017-01-30, as the 30th February doesn't exist. Still, such operations
/// can make sense, e.g. in the context of a date iterator.
/// In order to resolve this issue, this struct requires an `InvalidDateHandling`
/// to either skip or select the next closest date for invalid results.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonthDuration(pub i32, pub InvalidDateHandling);

//TODO: Remove debug again?
impl <T: Datelike + Debug> DateOp<T> for MonthDuration {
    fn times(&self, n: i32) -> Option<Self> {
        Some(MonthDuration(try_opt!(self.0.checked_mul(n)), self.1))
    }

    fn add_to(&self, dt: &T) -> Option<T> {
        let next_month_0 = try_opt!((dt.month0() as i64).checked_add(self.0 as i64));
        let additional_years = next_month_0 / 12;
        let mut next_month_0 = (next_month_0 % 12) as u32;
        let additional_years = if additional_years >= (i32::max_value() as i64) {
            return None;
        } else {
            additional_years as i32
        };
        let mut next_year = try_opt!(dt.year().checked_add(additional_years));
        let last_day = last_day_of_month_0(next_year, next_month_0);
        let mut next_day = dt.day();
        // check for invalid date
        if next_day > last_day {
            use date_ops::month_duration::InvalidDateHandling::*;
            match self.1 {
                Reject => return None,
                Previous => next_day = last_day,
                Next => {
                    next_day = 1;
                    next_month_0 = (next_month_0 + 1) % 12;
                    if next_month_0 == 0 {
                        next_year = try_opt!(next_year.checked_add(1))
                    };
                },
            }
        };
        // TODO: Is there a better way?
        dt.with_year(next_year).
            // set days to 1 so that month is always valid
            and_then(|dt| dt.with_day(1)).
            and_then(|dt| dt.with_month(next_month_0 + 1)).
            and_then(|dt| dt.with_day(next_day))
    }
}


