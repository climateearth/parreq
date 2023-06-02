use std::iter::{Skip, StepBy};

/// Takes an input iterator and return an iterator of iterators in provided batches size
///
/// Example : input vector of 5 and split in 3 batches
///
/// ---------------------
/// | 1 | 2 | 3 | 4 | 5 |
/// ---------------------
///        |
///        |
///        V
/// -------------
/// | 1 | 2 | 3 |         batch 1:   1,4
/// -------------     ->  batch 2:   2,5
/// | 4 | 5 |             batch 3:   3
///
pub(crate) fn split<I>(iter: &I, n_batches: usize) -> Vec<StepBy<Skip<I>>>
where
    I: Iterator + Clone,
{
    let mut batches = vec![];
    for i in 0..n_batches {
        let iter = iter.clone().skip(i).step_by(n_batches);
        batches.push(iter);
    }
    batches
}

#[cfg(test)]
mod tests {
    use crate::batcher::split;

    #[test]
    fn it_should_return_() {
        let input = vec![1, 2, 3, 4, 5];
        let mut splits = split(&input.into_iter(), 3);

        let batch2: Vec<i32> = splits.pop().unwrap().collect();
        let batch1: Vec<i32> = splits.pop().unwrap().collect();
        let batch0: Vec<i32> = splits.pop().unwrap().collect();

        assert_eq!(vec![1, 4], batch0);
        assert_eq!(vec![2, 5], batch1);
        assert_eq!(vec![3], batch2);
    }

        #[test]
    fn it_should_run_2() {
        let input = vec![1, 2, 3, 4];
        let mut splits = split(&input.into_iter(), 2);

        let batch1: Vec<i32> = splits.pop().unwrap().collect();
        let batch0: Vec<i32> = splits.pop().unwrap().collect();

        assert_eq!(vec![1, 3], batch0);
        assert_eq!(vec![2, 4], batch1);
    }
}
