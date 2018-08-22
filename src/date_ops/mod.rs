//! The date_ops module defines operations that can be added to Dates, e.g.
//! adding "one month and two hours" to a given Date.
//! TODO: Expand docs

mod year_duration;
pub use self::year_duration::YearDuration;
mod day_duration;
pub use self::day_duration::DayDuration;
mod month_duration;
pub use self::month_duration::{MonthDuration, InvalidDateHandling};
mod and_then;
pub use self::and_then::AndThen;

use std::fmt::Debug;
use std::marker::PhantomData;

use Datelike;
use Duration as OldDuration;

/// A DateOp is something that you can add to a Date<Tz>, DateTime<Tz>, NaiveDate or NaiveDateTime
pub trait DateOp<T: Datelike> : Sized + Debug {

    /// Multiplies the given DateOp by n (which can fail if e.g. n overflows)
    /// This is needed because adding one month to Jan 31st is impossible (Feb 31st doesn't exist),
    /// but adding two months works.
    /// TODO: Is i32 big enough? Someone might want to iterator over nanoseconds or something...
    fn times(&self, n: i32) -> Option<Self>;

    /// Returns a n T with self added to it; can fail e.g. because of overflow or because of
    /// invalid date operations (e.g. adding one month to Jan 31st)
    fn add_to(&self, &T) -> Option<T>;

    /// Chain the other DateOp to be applied after this one
    fn and_then<O: DateOp<T>>(self, other: O) -> AndThen<Self, O, T> where T: Debug {
        AndThen(self, other, PhantomData)
    }
}

impl <T: Datelike + Clone> DateOp<T> for OldDuration {
    fn times(&self, n: i32) -> Option<Self> {
        // TODO: checked_mul for OldDuration?
        Some(*self * n)
    }

    fn add_to(&self, t: &T) -> Option<T> {
        t.clone().checked_add(self.clone())
    }
}

// TODO: Add EndOfYear/Month/Day/Hour/Minute/Second DateOps?

#[cfg(test)]
mod tests {

    use std::str::FromStr;
    use ::Utc;
    use ::DateTime;

    use super::*;

    #[test]
    pub fn add_simple() {
        let input = "1996-12-19T16:39:57.123Z";
        let dt = DateTime::<Utc>::from_str(input).unwrap();
        assert_eq!(input, format!("{:?}", dt));

        let duration = DayDuration(3).
            and_then(OldDuration::hours(1)).
            and_then(MonthDuration(5, InvalidDateHandling::Previous)).
            and_then(YearDuration(1));

        let result = duration.add_to(&dt);
        assert_eq!("1998-05-22T17:39:57.123Z", format!("{:?}", result.unwrap()));
    }

    #[test]
    pub fn add_overflow_checked() {
        let input = "1996-12-19T16:39:57.123Z";
        let dt = DateTime::<Utc>::from_str(input).unwrap();
        assert_eq!(input, format!("{:?}", dt));

        let duration = YearDuration(300_000);
        assert_eq!(None, duration.add_to(&dt));
    }

    #[test]
    pub fn add_adjusted() {
        let input = "1996-12-31T16:39:57.123Z";
        let dt = DateTime::<Utc>::from_str(input).unwrap();
        assert_eq!(input, format!("{:?}", dt));

        let duration = MonthDuration(2, InvalidDateHandling::Previous);
        let result = duration.add_to(&dt);
        //Note how february doesn't have a 31st day...
        assert_eq!("Some(1997-02-28T16:39:57.123Z)", format!("{:?}", result));

        let duration = MonthDuration(4, InvalidDateHandling::Previous);
        let result = duration.add_to(&dt);
        //...and neither has april
        assert_eq!("Some(1997-04-30T16:39:57.123Z)", format!("{:?}", result));

        let duration = MonthDuration(5, InvalidDateHandling::Previous);
        let result = duration.add_to(&dt);
        //But May is ok
        assert_eq!("Some(1997-05-31T16:39:57.123Z)", format!("{:?}", result));
    }
}
