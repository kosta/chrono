//! Exposes date iterators that can be used to express "start at time X and every hour afterwards"

use Datelike;
use date_ops::DateOp;

/// Iterator as returned by `date_iterator_from`
#[derive(Debug)]
pub struct OpenEndedDateIterator<Op: DateOp<T>, T: Datelike + Clone> {
    from: T,
    duration: Op,
    iterations: i32,
}

impl<Op: DateOp<T>, T: Datelike + Clone> OpenEndedDateIterator<Op, T> {
    /// return a new DateIterator that stops iteration when `to` is reached (`to` is not included)
    pub fn to(self, to: T) -> ClosedDateIterator<T, Self> {
        date_iterator_to(self, to)
    }

    /// needed here so that pairwise can work
    fn current(&self) -> Option<T> {
        self.duration.times(self.iterations)?.add_to(&self.from.clone())
    }

    /// returns a pairwise iterator of (next, after_next) dates. This is if you use the date iterator to
    /// e.g. slice a time range into Months. Note that it is not sufficient to take the date returned by
    /// `next()` and add `duration` as this can lead to overlapping slices.
    ///
    /// As an example e.g. if your starting date is
    /// e.g. January 31st and your duration is 1 month. pairwise iteration will yield (January 31st, Feb 28th),
    /// (Feb 28th, March 30th), (March 30th, April 31st), etc. This is different from if you simply used a
    /// date iterator (which would yield January 31st, Feb 28th, March 30th) and construct pairs by adding one
    /// month, which leads to errorneous (Feb 28th, March 28th) on the second iteration.
    pub fn pairwise(self) -> OpenEndedPairwiseDateIterator<Op, T> {
        OpenEndedPairwiseDateIterator { iter: self }
    }
}

/// As returned by [`OpenEndedDateIterator::pairwise()`] method. See comment there.
///
/// [`OpenEndedDateIterator::pairwise()`]: struct.OpenEndedDateIterator.html#method.pairwise
#[derive(Debug)]
pub struct OpenEndedPairwiseDateIterator<Op: DateOp<T>, T: Datelike + Clone> {
    iter: OpenEndedDateIterator<Op, T>,
}

/// Iterator that yields dates that until the given `to` date. (All dates are smaller than `to`).
/// TODO: Find a better name :)
/// TODO: Once impl Trait is stable, get rid of this struct and use `iterator.take_while()`
#[derive(Debug)]
pub struct ClosedDateIterator<T: Datelike, Iter: Iterator<Item = T>> {
    iter: Iter,
    to: T,
}

impl<Op: DateOp<T>, T: Datelike + Clone> ClosedDateIterator<T, OpenEndedDateIterator<Op, T>> {

    /// see comment on [`OpenEndedDateIterator::pairwise()`]
    ///
    ///[`OpenEndedDateIterator::pairwise()`]: struct.OpenEndedDateIterator.html#method.pairwise
    pub fn pairwise(self) -> ClosedPairwiseDateIterator<Op, T> {
        ClosedPairwiseDateIterator {
            iter: self.iter.pairwise(),
            to: self.to,
        }
    }
}

/// As returned by [`ClosedDateIterator::pairwise()`]. See comment on [`OpenEndedDateIterator::pairwise()`]
///
///[`ClosedDateIterator::pairwise()`]: struct.ClosedDateIterator.html#method.pairwise
///[`OpenEndedDateIterator::pairwise()`]: struct.OpenEndedDateIterator.html#method.pairwise
#[derive(Debug)]
pub struct ClosedPairwiseDateIterator<Op: DateOp<T>, T: Datelike + Clone> {
    iter: OpenEndedPairwiseDateIterator<Op, T>,
    to: T,
}

/// returns an open ended `Iterator`, that will first yield `dt`
pub fn date_iterator_from<Op: DateOp<T>, T: Datelike + Clone>(dt: T,
                                                      duration: Op)
                                                      -> OpenEndedDateIterator<Op, T> {
    OpenEndedDateIterator {
        from: dt,
        duration: duration,
        iterations: 0,
    }
}

/// return a new DateIterator that stops iteration when `to` is reached (`to` is not included)
pub fn date_iterator_to<T: Datelike, Iter: Iterator<Item = T>>
    (iter: Iter,
     to: T)
     -> ClosedDateIterator<T, Iter> {
    ClosedDateIterator { iter: iter, to: to }
}

/// return a new DateIterator that starts at `from` and yields results for every added duration until `to` is reached (`to` is not included)
pub fn date_iterator_from_to<T: Datelike + Clone, Op: DateOp<T>>
    (from: T,
     duration: Op,
     to: T)
     -> ClosedDateIterator<T, OpenEndedDateIterator<Op, T>> {

    date_iterator_from(from, duration).to(to)
}

impl<Op: DateOp<T>, T: Datelike + Clone> Iterator for OpenEndedDateIterator<Op, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.current();
        self.iterations += 1;
        next
    }
}

impl<Op: DateOp<T>, T: Datelike + Clone> Iterator for OpenEndedPairwiseDateIterator<Op, T> {
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .and_then(|start| Some((start, try_opt!(self.iter.current()))))
    }
}

impl<T: Datelike + PartialOrd, Iter: Iterator<Item = T>> Iterator for ClosedDateIterator<T, Iter> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // this would be really cool if Option.filter() existed :)
        // -> exists since rust 1.27 (but chrono is on Rust 1.13?)
        self.iter
            .next()
            .and_then(|dt| if dt < self.to { Some(dt) } else { None })
    }
}

impl<Op: DateOp<T>, T: Datelike + Clone + PartialOrd> Iterator for ClosedPairwiseDateIterator<Op, T> {
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        // this would be really cool if Option.filter() existed :)
        self.iter
            .next()
            .and_then(|dts| if dts.0 < self.to { Some(dts) } else { None })
    }
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use DateTime;
    use Utc;
    use Duration as OldDuration;

    use date_ops::*;
    use date_ops::InvalidDateHandling::Previous;

    use super::*;

    #[test]
    pub fn test_date_iterator_from() {
        let input = "1996-12-25T16:39:57.123Z";
        let dt = DateTime::<Utc>::from_str(input).unwrap();
        assert_eq!(input, format!("{:?}", dt));

        let duration = DayDuration(2).
            and_then(OldDuration::minutes(4)).
            and_then(YearDuration(3)).
            and_then(MonthDuration(1, Previous));

        let iter = date_iterator_from(dt, duration);
        let expected = vec!["1996-12-25T16:39:57.123Z",
                            "2000-01-27T16:43:57.123Z",
                            "2003-02-28T16:47:57.123Z",
                            "2006-03-31T16:51:57.123Z"];

        assert_eq!(expected,
                   iter.take(4)
                       .map(|d| format!("{:?}", d))
                       .collect::<Vec<_>>());
    }

    #[test]
    pub fn test_date_iterator_from_to() {
        let from_str = "1996-12-25T16:39:57.123Z";
        let from_dt = DateTime::<Utc>::from_str(from_str).unwrap();
        assert_eq!(from_str, format!("{:?}", from_dt));

        let to_str = "2006-03-31T16:51:57.123Z";
        let to_dt = DateTime::<Utc>::from_str(to_str).unwrap();
        assert_eq!(to_str, format!("{:?}", to_dt));

        let duration = DayDuration(2).
            and_then(OldDuration::minutes(4)).
            and_then(YearDuration(3)).
            and_then(MonthDuration(1, Previous));

        let iter = date_iterator_from(from_dt, duration).to(to_dt);
        let expected = vec!["1996-12-25T16:39:57.123Z",
                            "2000-01-27T16:43:57.123Z",
                            "2003-02-28T16:47:57.123Z"];

        assert_eq!(expected,
                   iter.map(|d| format!("{:?}", d)).collect::<Vec<_>>());
    }

    #[test]
    pub fn test_date_iterator_from_to_pairwise() {
        let from_str = "1996-12-25T16:39:57.123Z";
        let from_dt = DateTime::<Utc>::from_str(from_str).unwrap();
        assert_eq!(from_str, format!("{:?}", from_dt));

        let to_str = "2006-03-31T16:51:57.123Z";
        let to_dt = DateTime::<Utc>::from_str(to_str).unwrap();
        assert_eq!(to_str, format!("{:?}", to_dt));

        let duration = DayDuration(2).
            and_then(OldDuration::minutes(4)).
            and_then(YearDuration(3)).
            and_then(MonthDuration(1, Previous));

        let iter = date_iterator_from(from_dt, duration)
            .to(to_dt)
            .pairwise();
        let expected = vec!["1996-12-25T16:39:57.123Z to 2000-01-27T16:43:57.123Z",
                            "2000-01-27T16:43:57.123Z to 2003-02-28T16:47:57.123Z",
                            "2003-02-28T16:47:57.123Z to 2006-03-31T16:51:57.123Z"];

        assert_eq!(expected,
                   iter.map(|d| format!("{:?} to {:?}", d.0, d.1))
                       .collect::<Vec<_>>());
    }

}
