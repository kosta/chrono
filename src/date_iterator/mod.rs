//! Exposes CalendarDuration and DateIterators

use super::{Datelike, NaiveDate};

pub mod calendar_duration;
pub mod date_iterators;

pub use self::calendar_duration::{CalendarDuration, add, checked_add};
pub use self::date_iterators::{OpenEndedDateIterator, ClosedDateIterator, date_iterator_from,
                         date_iterator_to, date_iterator_from_to};

//from https://github.com/chronotope/chrono/issues/29

/// return whether the given year is a leap year
pub fn is_leap_year(year: i32) -> bool {
    NaiveDate::from_ymd_opt(year, 2, 29).is_some()
}

/// return the last day of a given month/year, where month is zero-based (January = 0, February = 1, etc)
pub fn last_day_of_month_0(year: i32, month_0: u32) -> u32 {
    last_day_of_month(year, month_0 + 1)
}

/// return the last day of a given month/year, where month is one-based (January = 1, February = 2, etc)
pub fn last_day_of_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd(year + 1, 1, 1))
        .pred()
        .day()
}
