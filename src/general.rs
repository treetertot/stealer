use std::sync::Mutex;
use std::iter::{ ExactSizeIterator, Enumerate };
use crossbeam_utils::thread::scope;
use arrayvec::ArrayVec;
use crate::vecbuilder::{ SharedVec, UnsafeVecBuilder };

struct ParIter<I: ExactSizeIterator>(Mutex<Enumerate<I>>);
impl<I: ExactSizeIterator> ParIter<I> {
    fn next_n(&self) -> ArrayVec::<[(usize, I::Item); 32]> {
        let mut out = ArrayVec::new();
        let mut iter = self.0.lock().unwrap();
        for _ in 0..out.capacity() {
            if let Some(val) = iter.next() {
                out.push(val);
            } else {
                return out;
            }
        }
        out
    }
}

fn worker<I: ExactSizeIterator, O, F: Fn(I::Item) -> O>(iter: &ParIter<I>, mut output: UnsafeVecBuilder<O>, func: F) {
    loop {
        let buffer = iter.next_n();
        if buffer.len() == 0 {
            break;
        }
        for (i, item) in buffer {
            unsafe {
                output.insert(i, func(item));
            }
        }
    }
}

pub fn run<I: ExactSizeIterator + Send, O: Send, F: Fn(I::Item) -> O + Sync>(iter: I, func: F) -> Option<Vec<O>> {
    let mut out = SharedVec::new(iter.len());
    let iter = ParIter(Mutex::new(iter.enumerate()));
    let res = scope(|s| {
        for _ in 0..num_cpus::get() {
            let builder = out.builder();
            s.spawn(|_| {worker(&iter, builder, &func)});
        }
        worker(&iter, out.builder(), &func);
    });
    match res {
        Ok(_) => (),
        Err(_) => return None,
    }
    out.collapse()
}
