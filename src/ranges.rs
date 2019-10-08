use std::sync::{atomic::{Ordering, AtomicU64, AtomicBool}, Arc, RwLock};
use std::ops::Range;
use std::{thread, time};
use crate::vecbuilder::{SharedVec, UnsafeVecBuilder};

pub struct MonoRange {
    counter: AtomicU64,
    offset: u64,
    end: u64,
}
impl MonoRange {
    pub fn new(start: u64, end: u64) -> MonoRange {
        MonoRange {
            counter: AtomicU64::new(0),
            offset: start,
            end: end - start,
        }
    }
    pub fn slice(&self, size: u64) -> Option<MonoSlice> {
        let n = self.counter.fetch_add(size, Ordering::SeqCst);
        if n > self.end {
            return None;
        }
        let end = n+size;
        if end > self.end {
            let end = self.end;
            return Some(MonoSlice {
                range: n..end,
                offset: self.offset,
                terminal: true,
            })
        }
        Some(MonoSlice {
            range: n..end,
            offset: self.offset,
            terminal: false,
        })
    }
}
pub struct MonoSlice {
    range: Range<u64>,
    offset: u64,
    terminal: bool,
}
impl Iterator for MonoSlice {
    type Item = (usize, u64);
    fn next(&mut self) -> Option<(usize, u64)> {
        let n = self.range.next()?;
        Some((n as usize, n+self.offset))
    }
}

pub struct ExecUnit<F: Fn(u64) -> T, T> {
    exp: F,
    output: UnsafeVecBuilder<T>,
    iter: MonoRange,
    complete: AtomicBool,
}
impl<F: Fn(u64) -> T + Send + Sync, T: Send> Unit for ExecUnit<F, T> {
    fn run(&self) {
        let mut out = self.output.clone();
        while let Some(iter) = self.iter.slice(32) {
            let end = iter.terminal;
            for (index, n) in iter {
                unsafe {
                    out.insert(index, (self.exp)(n));
                }
            }
            if end {
                self.complete.store(true, Ordering::Relaxed);
            }
        }
        let time = time::Duration::from_millis(10);
        while !self.complete.load(Ordering::Acquire) {thread::sleep(time)};
    }
}

trait Unit: Send + Sync {
    fn run(&self);
}

struct RunnerCore{
    unit: RwLock<Option<Box<dyn Unit>>>,
    kill: AtomicBool,
}

pub struct RangeRunner {
    core: Arc<RunnerCore>,
}
impl RangeRunner {
    pub fn new(threads: usize) -> Self {
        let core = Arc::new(RunnerCore{
            unit: RwLock::new(None),
            kill: AtomicBool::new(false),
        });
        for _ in 0..threads {
            let core = core.clone();
            thread::spawn(move || {
                let time = time::Duration::from_millis(10);
                while !core.kill.load(Ordering::Acquire) {
                    match &*core.unit.read().unwrap() {
                        Some(execunit) => execunit.run(),
                        None => thread::sleep(time),
                    }
                }

            });
        }
        RangeRunner {
            core: core,
        }
    }
    pub fn run<F: Fn(u64) -> T + Send + Sync + 'static, T: Send + 'static>(&self, start: u64, end: u64, func: F) -> Option<Vec<T>> {
        let mut output = SharedVec::new((end-start) as usize);
        let unit: Box<dyn Unit> = Box::new(ExecUnit {
            exp: func,
            output: output.builder(),
            iter: MonoRange::new(start, end),
            complete: AtomicBool::new(false),
        });
        *self.core.unit.write().unwrap() = Some(unit);
        self.core.unit.read().unwrap().as_ref().unwrap().run();
        output.collapse()
    }
}
impl Drop for RangeRunner {
    fn drop(&mut self) {
        self.core.kill.store(true, Ordering::Relaxed);
    }
}
