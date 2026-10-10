use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

const MAXIMUM_AT_ONCE: usize = 16;

pub struct ParallelWork<'a, T, F> {
    pub items: &'a [T],
    pub work: F,
}

pub fn each_in_parallel<T: Sync, R: Send, F: Fn(&T) -> R + Sync>(
    parallel: ParallelWork<T, F>,
) -> Vec<R> {
    let ParallelWork { items, work } = parallel;
    let next = AtomicUsize::new(0);
    let workers = MAXIMUM_AT_ONCE.min(items.len());
    let mut done: Vec<(usize, R)> = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut finished = Vec::new();
                    loop {
                        let position = next.fetch_add(1, Ordering::Relaxed);
                        let Some(item) = items.get(position) else {
                            return finished;
                        };
                        finished.push((position, work(item)));
                    }
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap_or_default())
            .collect()
    });
    done.sort_by_key(|(position, _)| *position);
    done.into_iter().map(|(_, result)| result).collect()
}
