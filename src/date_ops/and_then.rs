use std::fmt::Debug;
use std::marker::PhantomData;

use date_ops::DateOp;
use Datelike;

/// AndThen represents the chaining of two DateOps, applied consecutively
/// TODO: Get rid of PhantomData?
#[derive(Debug)]
pub struct AndThen<Op1: DateOp<T>, Op2: DateOp<T>, T: Datelike + Debug>(pub Op1, pub Op2, pub PhantomData<T>);

impl <Op1: DateOp<T>,
      Op2: DateOp<T>,
      T: Datelike + Debug>
DateOp<T> for AndThen<Op1, Op2, T> {

    fn times(&self, n: i32) -> Option<Self> {
        Some(AndThen(
            try_opt!(self.0.times(n)),
            try_opt!(self.1.times(n)),
            PhantomData,
        ))
    }

    fn add_to(&self, dt: &T) -> Option<T> {
        self.0.add_to(dt).and_then(|dt| self.1.add_to(&dt))
    }

}