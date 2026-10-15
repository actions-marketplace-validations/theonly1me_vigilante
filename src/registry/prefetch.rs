use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use super::{Download, FetchRequest, PackageId, Registry};

const MAXIMUM_DOWNLOADS_AT_ONCE: usize = 16;

pub struct Prefetch {
    pub ecosystem: &'static str,
    pub name: String,
    pub version: String,
    pub download: Download,
    pub keep: fn(&str) -> bool,
}

impl Registry {
    pub fn prefetch(&self, items: &[Prefetch]) {
        let next = AtomicUsize::new(0);
        let workers = MAXIMUM_DOWNLOADS_AT_ONCE.min(items.len());
        thread::scope(|scope| {
            for _ in 0..workers {
                scope.spawn(|| {
                    while let Some(item) = items.get(next.fetch_add(1, Ordering::Relaxed)) {
                        let _ = self.fetch(FetchRequest {
                            package: PackageId {
                                ecosystem: item.ecosystem,
                                name: &item.name,
                                version: &item.version,
                            },
                            download: &item.download,
                            keep: &item.keep,
                        });
                    }
                });
            }
        });
    }
}
