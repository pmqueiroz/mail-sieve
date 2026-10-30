use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UidSet {
    ranges: Vec<(u32, u32)>,
}

impl UidSet {
    pub fn from_uids(uids: impl IntoIterator<Item = u32>) -> Self {
        let mut sorted: Vec<u32> = uids.into_iter().filter(|&uid| uid > 0).collect();
        sorted.sort_unstable();
        sorted.dedup();
        let mut ranges: Vec<(u32, u32)> = Vec::new();
        for uid in sorted {
            match ranges.last_mut() {
                Some((_, end)) if *end + 1 == uid => *end = uid,
                _ => ranges.push((uid, uid)),
            }
        }
        Self { ranges }
    }

    pub fn range(start: u32, end: u32) -> Self {
        if start == 0 || end < start {
            return Self::default();
        }
        Self { ranges: vec![(start, end)] }
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ranges.iter().map(|(start, end)| (end - start + 1) as usize).sum()
    }

    pub fn ranges(&self) -> &[(u32, u32)] {
        &self.ranges
    }

    pub fn chunks_by_count(&self, max_uids: usize) -> Vec<UidSet> {
        let max_uids = max_uids.max(1) as u32;
        let mut chunks = Vec::new();
        let mut current = Vec::new();
        let mut current_len = 0u32;
        for &(start, end) in &self.ranges {
            let mut cursor = start;
            while cursor <= end {
                let room = max_uids - current_len;
                let take_end = end.min(cursor.saturating_add(room - 1));
                current.push((cursor, take_end));
                current_len += take_end - cursor + 1;
                if current_len == max_uids {
                    chunks.push(UidSet { ranges: std::mem::take(&mut current) });
                    current_len = 0;
                }
                if take_end == u32::MAX {
                    break;
                }
                cursor = take_end + 1;
            }
        }
        if !current.is_empty() {
            chunks.push(UidSet { ranges: current });
        }
        chunks
    }

    pub fn chunks_by_encoded_len(&self, max_bytes: usize) -> Vec<UidSet> {
        let mut chunks = Vec::new();
        let mut current = Vec::new();
        let mut current_bytes = 0usize;
        for &range in &self.ranges {
            let encoded = encoded_range_len(range) + usize::from(!current.is_empty());
            if !current.is_empty() && current_bytes + encoded > max_bytes {
                chunks.push(UidSet { ranges: std::mem::take(&mut current) });
                current_bytes = 0;
            }
            current_bytes += encoded_range_len(range) + usize::from(!current.is_empty());
            current.push(range);
        }
        if !current.is_empty() {
            chunks.push(UidSet { ranges: current });
        }
        chunks
    }

    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.ranges.iter().flat_map(|&(start, end)| start..=end)
    }
}

fn digits(value: u32) -> usize {
    value.checked_ilog10().map_or(1, |log| log as usize + 1)
}

fn encoded_range_len((start, end): (u32, u32)) -> usize {
    if start == end {
        digits(start)
    } else {
        digits(start) + 1 + digits(end)
    }
}

impl fmt::Display for UidSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, &(start, end)) in self.ranges.iter().enumerate() {
            if index > 0 {
                f.write_str(",")?;
            }
            if start == end {
                write!(f, "{start}")?;
            } else {
                write!(f, "{start}:{end}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compresses_contiguous_runs() {
        let set = UidSet::from_uids([5, 1, 2, 3, 9, 10, 3, 0]);
        assert_eq!(set.to_string(), "1:3,5,9:10");
        assert_eq!(set.len(), 6);
    }

    #[test]
    fn empty_set_renders_empty() {
        assert!(UidSet::from_uids([]).is_empty());
        assert_eq!(UidSet::range(5, 4).to_string(), "");
    }

    #[test]
    fn chunks_by_count_splits_inside_ranges() {
        let set = UidSet::from_uids((1..=10).chain([20, 21]));
        let chunks: Vec<String> = set.chunks_by_count(4).iter().map(ToString::to_string).collect();
        assert_eq!(chunks, ["1:4", "5:8", "9:10,20:21"]);
    }

    #[test]
    fn chunks_by_encoded_len_respects_limit() {
        let set = UidSet::from_uids((1..=2000).step_by(2));
        for chunk in set.chunks_by_encoded_len(100) {
            assert!(chunk.to_string().len() <= 100);
        }
        let total: usize = set.chunks_by_encoded_len(100).iter().map(UidSet::len).sum();
        assert_eq!(total, 1000);
    }

    #[test]
    fn iter_round_trips() {
        let uids = vec![1, 2, 3, 7, 8, 100];
        assert_eq!(UidSet::from_uids(uids.clone()).iter().collect::<Vec<_>>(), uids);
    }
}
