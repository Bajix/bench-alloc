#![allow(dead_code)]
use const_array_init::const_arr;
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    cell::UnsafeCell,
    future::Future,
    pin::Pin,
    ptr,
    sync::atomic::{AtomicPtr, AtomicU64},
};

struct TaskHandle {
    task: UnsafeCell<Option<Pin<Box<dyn Future<Output = ()> + 'static>>>>,
    next_enqueued: AtomicPtr<()>,
}

impl TaskHandle {
    const fn new() -> Self {
        TaskHandle {
            task: UnsafeCell::new(None),
            next_enqueued: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

#[repr(align(64))]
struct TaskArena {
    occupancy: AtomicU64,
    tasks: [TaskHandle; 64],
    next: AtomicPtr<()>,
}

impl TaskArena {
    const fn new() -> Self {
        TaskArena {
            occupancy: AtomicU64::new(0),
            #[allow(clippy::declare_interior_mutable_const)]
            tasks: const_arr!([TaskHandle; 64], |_| TaskHandle::new()),
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut benches = c.benchmark_group("Alloc");

    benches.bench_function("Boxed x 1", |bencher| {
        bencher.iter(|| {
            let slab = black_box(|| Box::new(TaskArena::new()))();

            drop(slab);
        })
    });

    benches.bench_function("Zeroed x 1", |bencher| {
        bencher.iter(|| {
            let layout: Layout = Layout::new::<TaskArena>();

            let ptr = black_box(|| unsafe { alloc_zeroed(layout) })();

            unsafe { dealloc(ptr, layout) };
        });
    });

    benches.bench_function("Boxed x 4", |bencher| {
        bencher.iter(|| {
            let slab: [Box<TaskArena>; 4] = black_box(|| {
                [
                    Box::new(TaskArena::new()),
                    Box::new(TaskArena::new()),
                    Box::new(TaskArena::new()),
                    Box::new(TaskArena::new()),
                ]
            })();

            drop(slab);
        })
    });

    benches.bench_function("Zeroed x 4", |bencher| {
        bencher.iter(|| {
            let layout: Layout = Layout::new::<[TaskArena; 4]>();

            let ptr = black_box(|| unsafe { alloc_zeroed(layout) })();

            unsafe { dealloc(ptr, layout) };
        });
    });

    benches.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
