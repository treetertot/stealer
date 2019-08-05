#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let mut nums = Vec::with_capacity(10000);
        for i in 0..10000 {
            nums.push(i);
        }
        let fizzy = crate::run(nums.iter(), |n| {
            if n % 3 == 0 {
                if n % 5 == 0 {
                    String::from("FizzBuzz")                    
                } else {
                    String::from("Fizz")
                }
            } else if n % 5 == 0 {
                String::from("Buzz")
            } else {
                format!("{}", n)
            }
        });
        if let Some(val) = fizzy {
            for i in val {
                println!("{}", i);
            }
        } else {
            panic!("AAAAAAAAAA");
        }
    }
}

use std::ptr::NonNull;
use std::sync::Mutex;
use std::iter::{ ExactSizeIterator, Enumerate };
use crossbeam_utils::thread::scope;
use arrayvec::ArrayVec;

struct UnsafeVecBuilder<T> {
    target: NonNull<Vec<Option<T>>>,
}
unsafe impl<T> Send for UnsafeVecBuilder<T> where T: Send {}
impl<T> UnsafeVecBuilder<T> {
    unsafe fn insert(&mut self, index: usize, item: T) {
        let list = self.target.as_mut();
        list[index] = Some(item);
    }
}

struct SharedVec<T> (Vec<Option<T>>);
impl<T> SharedVec<T> {
    fn new(cap: usize) -> Self {
        let mut base = Vec::with_capacity(cap);
        for _ in 0..cap {
            base.push(None);
        }
        SharedVec(base)
    }
    fn builder(&mut self) -> UnsafeVecBuilder<T> {
        let ptr = (&mut self.0) as *mut Vec<Option<T>>;
        unsafe {
            UnsafeVecBuilder {
                target: NonNull::new_unchecked(ptr)
            }
        }
    }
    fn collapse(self) -> Option<Vec<T>> {
        let mut out = Vec::with_capacity(self.0.len());
        for item in self.0 {
            out.push(item?);
        }
        Some(out)
    }
}

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
    let _res = scope(|s| {
        for _ in 0..num_cpus::get() {
            let builder = out.builder();
            s.spawn(|_| {worker(&iter, builder, &func)});
        }
        worker(&iter, out.builder(), &func);
    });
    out.collapse()
}
