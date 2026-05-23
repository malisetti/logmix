use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::record::Record;

struct HeapEntry {
    record: Record,
    source_idx: usize,
    ts: Option<i64>,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for HeapEntry {}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.ts, other.ts) {
            (Some(a), Some(b)) => match a.cmp(&b) {
                Ordering::Equal => other.source_idx.cmp(&self.source_idx),
                ord => ord.reverse(),
            },
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => other.source_idx.cmp(&self.source_idx),
        }
    }
}

struct SourceIter<I> {
    iter: I,
}

struct MergeState<I>
where
    I: Iterator<Item = Record>,
{
    heap: BinaryHeap<HeapEntry>,
    sources: Vec<SourceIter<I>>,
}

impl<I> MergeState<I>
where
    I: Iterator<Item = Record>,
{
    fn push_next(&mut self, source_idx: usize) {
        if let Some(record) = self.sources[source_idx].iter.next() {
            self.heap.push(HeapEntry {
                ts: record.ts,
                record,
                source_idx,
            });
        }
    }
}

struct MergeIter<I>
where
    I: Iterator<Item = Record>,
{
    state: MergeState<I>,
}

impl<I> Iterator for MergeIter<I>
where
    I: Iterator<Item = Record>,
{
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.state.heap.pop()?;
        let source_idx = entry.source_idx;
        let record = entry.record;
        self.state.push_next(source_idx);
        Some(record)
    }
}

pub fn merge<I>(sources: Vec<I>) -> impl Iterator<Item = Record>
where
    I: Iterator<Item = Record>,
{
    let mut state = MergeState {
        heap: BinaryHeap::new(),
        sources: sources
            .into_iter()
            .map(|iter| SourceIter { iter })
            .collect(),
    };
    for source_idx in 0..state.sources.len() {
        state.push_next(source_idx);
    }
    MergeIter { state }
}
