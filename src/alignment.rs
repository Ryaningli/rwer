/// Represents a single edit operation in the alignment between reference and hypothesis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditOp {
    /// Reference and hypothesis tokens match at this index.
    Equal {
        /// Index into the reference (and hypothesis) token list.
        index: usize,
    },
    /// Reference token was substituted by hypothesis token.
    Substitute {
        /// Index into the reference token list.
        ref_index: usize,
        /// Index into the hypothesis token list.
        hyp_index: usize,
    },
    /// Hypothesis token was inserted (not in reference).
    Insert {
        /// Index into the hypothesis token list.
        hyp_index: usize,
    },
    /// Reference token was deleted (not in hypothesis).
    Delete {
        /// Index into the reference token list.
        ref_index: usize,
    },
}

impl EditOp {
    /// Returns `true` if this is an equal (match) operation.
    #[must_use]
    pub fn is_equal(&self) -> bool {
        matches!(self, EditOp::Equal { .. })
    }

    /// Returns `true` if this is any error operation (substitution, insertion, or deletion).
    #[must_use]
    pub fn is_error(&self) -> bool {
        !self.is_equal()
    }

    /// Returns `true` if this is a substitution operation.
    #[must_use]
    pub fn is_substitute(&self) -> bool {
        matches!(self, EditOp::Substitute { .. })
    }

    /// Returns `true` if this is an insertion operation.
    #[must_use]
    pub fn is_insert(&self) -> bool {
        matches!(self, EditOp::Insert { .. })
    }

    /// Returns `true` if this is a deletion operation.
    #[must_use]
    pub fn is_delete(&self) -> bool {
        matches!(self, EditOp::Delete { .. })
    }
}

/// Counts of each edit operation type from an alignment.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OperationCounts {
    /// Number of matching tokens.
    pub hits: usize,
    /// Number of substitutions.
    pub substitutions: usize,
    /// Number of deletions.
    pub deletions: usize,
    /// Number of insertions.
    pub insertions: usize,
}

/// Count the occurrences of each operation type.
#[must_use]
pub fn count_operations(ops: &[EditOp]) -> OperationCounts {
    let mut counts = OperationCounts::default();
    for op in ops {
        match op {
            EditOp::Equal { .. } => counts.hits += 1,
            EditOp::Substitute { .. } => counts.substitutions += 1,
            EditOp::Insert { .. } => counts.insertions += 1,
            EditOp::Delete { .. } => counts.deletions += 1,
        }
    }
    counts
}

/// Compute the Levenshtein edit distance between two token sequences.
///
/// Uses single-row dynamic programming for O(min(M,N)) space complexity.
pub(crate) fn edit_distance<S: AsRef<str> + PartialEq>(reference: &[S], hypothesis: &[S]) -> usize {
    let m = reference.len();
    let n = hypothesis.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev_row: Vec<usize> = (0..=n).collect();
    let mut curr_row = vec![0; n + 1];

    for i in 1..=m {
        curr_row[0] = i;
        for j in 1..=n {
            let cost = usize::from(reference[i - 1] != hypothesis[j - 1]);
            curr_row[j] = (prev_row[j] + 1)
                .min(curr_row[j - 1] + 1)
                .min(prev_row[j - 1] + cost);
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[n]
}

/// Compute Levenshtein distance between two char sequences using rapidfuzz.
///
/// Uses Myers bit-parallel algorithm for O([K/64]×M) complexity where K is
/// the edit distance and M is the shorter sequence length.
pub(crate) fn rapidfuzz_char_distance(
    s1: impl IntoIterator<Item = char>,
    s2: impl IntoIterator<Item = char>,
) -> usize {
    let v1: Vec<char> = s1.into_iter().collect();
    let v2: Vec<char> = s2.into_iter().collect();
    rapidfuzz::distance::levenshtein::distance(v1.iter().copied(), v2.iter().copied())
}

/// Compute Levenshtein distance between two word sequences using rapidfuzz.
///
/// Maps words to `u64` IDs via a `HashMap`, then uses Myers bit-parallel
/// algorithm for O([K/64]×M) complexity where K is the edit distance
/// and M is the shorter sequence length.
#[allow(dead_code)]
pub(crate) fn rapidfuzz_word_distance<S: AsRef<str> + PartialEq>(
    reference: &[S],
    hypothesis: &[S],
) -> usize {
    let m = reference.len();
    let n = hypothesis.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut word_to_id = std::collections::HashMap::new();
    let mut next_id: u64 = 0;

    let ref_ids: Vec<u64> = reference
        .iter()
        .map(|w| {
            *word_to_id.entry(w.as_ref()).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            })
        })
        .collect();

    let hyp_ids: Vec<u64> = hypothesis
        .iter()
        .map(|w| {
            *word_to_id.entry(w.as_ref()).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            })
        })
        .collect();

    rapidfuzz::distance::levenshtein::distance(ref_ids.iter().copied(), hyp_ids.iter().copied())
}

/// Compute the Levenshtein alignment between two token sequences.
///
/// Uses a two-phase approach for performance:
/// 1. Compute total edit distance using single-row DP
/// 2. Run banded Wagner-Fischer (diagonal ± distance band) for traceback
///
/// The band width is derived from the precomputed distance, making this
/// much faster than full O(M×N) Wagner-Fischer when the distance is small
/// relative to the sequence lengths.
///
/// # Examples
///
/// ```
/// use rwer::alignment::align;
///
/// let ref_tokens = vec!["hello", "world"];
/// let hyp_tokens = vec!["hello", "earth"];
/// let ops = align(&ref_tokens, &hyp_tokens);
/// assert_eq!(ops.len(), 2);
/// ```
#[must_use]
pub fn align<S: AsRef<str> + PartialEq>(reference: &[S], hypothesis: &[S]) -> Vec<EditOp> {
    let m = reference.len();
    let n = hypothesis.len();

    if m == 0 {
        return (0..n).map(|i| EditOp::Insert { hyp_index: i }).collect();
    }
    if n == 0 {
        return (0..m).map(|i| EditOp::Delete { ref_index: i }).collect();
    }

    let dist = edit_distance(reference, hypothesis);

    if dist == 0 {
        return (0..m).map(|i| EditOp::Equal { index: i }).collect();
    }

    align_banded(reference, hypothesis, dist)
}

/// Banded Wagner-Fischer alignment.
///
/// Only computes DP cells within a band of width `(2 * dist + 1)` centered
/// on the main diagonal. This reduces time and space from O(M×N) to O(M×D)
/// where D is the edit distance.
fn align_banded<S: AsRef<str> + PartialEq>(
    reference: &[S],
    hypothesis: &[S],
    dist: usize,
) -> Vec<EditOp> {
    let ref_len = reference.len();
    let hyp_len = hypothesis.len();

    let band = dist;
    let lo = |row: usize| row.saturating_sub(band);
    let hi = |row: usize| std::cmp::min(hyp_len, row + band);

    let rows = build_banded_dp(reference, hypothesis, ref_len, &lo, &hi);
    backtrack_banded(reference, hypothesis, ref_len, hyp_len, &rows, &lo, &hi)
}

/// Build the banded DP table for Wagner-Fischer.
fn build_banded_dp<S: AsRef<str> + PartialEq>(
    reference: &[S],
    hypothesis: &[S],
    ref_len: usize,
    lo: &dyn Fn(usize) -> usize,
    hi: &dyn Fn(usize) -> usize,
) -> Vec<Vec<usize>> {
    let mut rows: Vec<Vec<usize>> = Vec::with_capacity(ref_len + 1);

    // Row 0
    {
        let lo_val = lo(0);
        let hi_val = hi(0);
        let mut row = vec![0; hi_val - lo_val + 1];
        for (idx, val) in row.iter_mut().enumerate() {
            *val = lo_val + idx;
        }
        rows.push(row);
    }

    // Rows 1..=ref_len
    for ref_idx in 1..=ref_len {
        let lo_val = lo(ref_idx);
        let hi_val = hi(ref_idx);
        let width = hi_val - lo_val + 1;
        let mut row = vec![0; width];

        let prev_lo = lo(ref_idx - 1);
        let prev_hi = hi(ref_idx - 1);

        for hyp_idx in lo_val..=hi_val {
            let local_j = hyp_idx - lo_val;

            if hyp_idx == 0 {
                row[local_j] = ref_idx;
                continue;
            }

            // SAFETY: Diagonal neighbor is always within the band when band = dist,
            // since the band width equals the maximum possible off-diagonal
            // displacement in an optimal alignment.
            let diag = rows[ref_idx - 1][hyp_idx - 1 - prev_lo];

            let up = if hyp_idx >= prev_lo && hyp_idx <= prev_hi {
                Some(rows[ref_idx - 1][hyp_idx - prev_lo] + 1)
            } else {
                None
            };

            let left = if hyp_idx > lo_val {
                Some(row[local_j - 1] + 1)
            } else {
                None
            };

            let cost = usize::from(reference[ref_idx - 1] != hypothesis[hyp_idx - 1]);
            let diag_val = diag + cost;

            row[local_j] = up
                .into_iter()
                .chain(left)
                .chain(Some(diag_val))
                .min()
                .unwrap_or(ref_idx + hyp_idx);
        }

        rows.push(row);
    }

    rows
}

/// Backtrack through the banded DP table to reconstruct the alignment.
fn backtrack_banded<S: AsRef<str> + PartialEq>(
    reference: &[S],
    hypothesis: &[S],
    ref_len: usize,
    hyp_len: usize,
    rows: &[Vec<usize>],
    lo: &dyn Fn(usize) -> usize,
    hi: &dyn Fn(usize) -> usize,
) -> Vec<EditOp> {
    let mut ops = Vec::with_capacity(ref_len + hyp_len);
    let (mut ref_pos, mut hyp_pos) = (ref_len, hyp_len);

    while ref_pos > 0 || hyp_pos > 0 {
        let lo_val = lo(ref_pos);
        // The optimal alignment path is always within the band when band = dist,
        // so hyp_pos is guaranteed to be within [lo_val, hi(ref_pos)].
        let cv = rows[ref_pos][hyp_pos - lo_val];

        if ref_pos > 0 && hyp_pos > 0 && reference[ref_pos - 1] == hypothesis[hyp_pos - 1] {
            ops.push(EditOp::Equal { index: ref_pos - 1 });
            ref_pos -= 1;
            hyp_pos -= 1;
        } else {
            let prev_lo = lo(ref_pos.saturating_sub(1));
            let prev_hi = if ref_pos > 0 { hi(ref_pos - 1) } else { 0 };

            let diag_ok = ref_pos > 0
                && hyp_pos > 0
                && hyp_pos > prev_lo
                && hyp_pos - 1 <= prev_hi
                && cv == rows[ref_pos - 1][hyp_pos - 1 - prev_lo] + 1;
            let left_ok = hyp_pos > lo_val && cv == rows[ref_pos][hyp_pos - 1 - lo_val] + 1;

            if diag_ok {
                ops.push(EditOp::Substitute {
                    ref_index: ref_pos - 1,
                    hyp_index: hyp_pos - 1,
                });
                ref_pos -= 1;
                hyp_pos -= 1;
            } else if left_ok {
                ops.push(EditOp::Insert {
                    hyp_index: hyp_pos - 1,
                });
                hyp_pos -= 1;
            } else {
                ops.push(EditOp::Delete {
                    ref_index: ref_pos - 1,
                });
                ref_pos -= 1;
            }
        }
    }
    ops.reverse();
    ops
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_operation() {
        let op = EditOp::Equal { index: 0 };
        assert!(op.is_equal());
        assert!(!op.is_error());
    }

    #[test]
    fn equal_operation_not_substitute_or_insert_or_delete() {
        let op = EditOp::Equal { index: 0 };
        assert!(!op.is_substitute());
        assert!(!op.is_insert());
        assert!(!op.is_delete());
    }

    #[test]
    fn substitute_operation() {
        let op = EditOp::Substitute {
            ref_index: 0,
            hyp_index: 0,
        };
        assert!(!op.is_equal());
        assert!(op.is_error());
        assert!(op.is_substitute());
        assert!(!op.is_insert());
        assert!(!op.is_delete());
    }

    #[test]
    fn insert_operation() {
        let op = EditOp::Insert { hyp_index: 0 };
        assert!(op.is_error());
        assert!(op.is_insert());
        assert!(!op.is_substitute());
        assert!(!op.is_delete());
    }

    #[test]
    fn delete_operation() {
        let op = EditOp::Delete { ref_index: 0 };
        assert!(op.is_error());
        assert!(op.is_delete());
        assert!(!op.is_substitute());
        assert!(!op.is_insert());
    }

    #[test]
    fn align_identical_sequences() {
        let ref_tokens = vec!["hello", "world"];
        let hyp_tokens = vec!["hello", "world"];
        let ops = align(&ref_tokens, &hyp_tokens);
        assert_eq!(ops.len(), 2);
        assert!(ops.iter().all(EditOp::is_equal));
    }

    #[test]
    fn align_empty_sequences() {
        let ops = align::<&str>(&[], &[]);
        assert!(ops.is_empty());
    }

    #[test]
    fn align_empty_reference() {
        let ref_tokens: Vec<&str> = vec![];
        let hyp_tokens = vec!["hello"];
        let ops = align(&ref_tokens, &hyp_tokens);
        assert_eq!(ops.len(), 1);
        assert!(ops[0].is_insert());
        assert_eq!(ops[0], EditOp::Insert { hyp_index: 0 });
    }

    #[test]
    fn align_empty_reference_multiple() {
        let ref_tokens: Vec<&str> = vec![];
        let hyp_tokens = vec!["hello", "world"];
        let ops = align(&ref_tokens, &hyp_tokens);
        assert_eq!(ops.len(), 2);
        assert!(ops[0].is_insert());
        assert!(ops[1].is_insert());
    }

    #[test]
    fn align_empty_hypothesis() {
        let ref_tokens = vec!["hello"];
        let hyp_tokens: Vec<&str> = vec![];
        let ops = align(&ref_tokens, &hyp_tokens);
        assert_eq!(ops.len(), 1);
        assert!(ops[0].is_delete());
        assert_eq!(ops[0], EditOp::Delete { ref_index: 0 });
    }

    #[test]
    fn align_empty_hypothesis_multiple() {
        let ref_tokens = vec!["hello", "world"];
        let hyp_tokens: Vec<&str> = vec![];
        let ops = align(&ref_tokens, &hyp_tokens);
        assert_eq!(ops.len(), 2);
        assert!(ops[0].is_delete());
        assert!(ops[1].is_delete());
    }

    #[test]
    fn align_with_substitution() {
        let ref_tokens = vec!["hello", "world"];
        let hyp_tokens = vec!["hello", "earth"];
        let ops = align(&ref_tokens, &hyp_tokens);
        assert_eq!(ops.len(), 2);
        assert!(ops[0].is_equal());
        assert!(ops[1].is_substitute());
        assert_eq!(
            ops[1],
            EditOp::Substitute {
                ref_index: 1,
                hyp_index: 1
            }
        );
    }

    #[test]
    fn align_with_deletion() {
        let ref_tokens = vec!["hello", "world", "foo"];
        let hyp_tokens = vec!["hello", "foo"];
        let ops = align(&ref_tokens, &hyp_tokens);
        assert_eq!(ops.len(), 3);
        assert!(ops[0].is_equal());
        assert!(ops[1].is_delete());
        assert!(ops[2].is_equal());
    }

    #[test]
    fn align_with_insertion() {
        let ref_tokens = vec!["hello", "foo"];
        let hyp_tokens = vec!["hello", "world", "foo"];
        let ops = align(&ref_tokens, &hyp_tokens);
        assert_eq!(ops.len(), 3);
        assert!(ops[0].is_equal());
        assert!(ops[1].is_insert());
        assert!(ops[2].is_equal());
    }

    #[test]
    fn align_complex_case() {
        let ref_tokens = vec!["the", "cat", "sat", "on", "the", "mat"];
        let hyp_tokens = vec!["the", "cat", "on", "the", "mat"];
        let ops = align(&ref_tokens, &hyp_tokens);
        let errors: Vec<_> = ops.iter().filter(|op| op.is_error()).collect();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].is_delete());
    }

    #[test]
    fn align_multiple_substitutions() {
        let ref_tokens = vec!["a", "b", "c"];
        let hyp_tokens = vec!["x", "y", "z"];
        let ops = align(&ref_tokens, &hyp_tokens);
        assert_eq!(ops.len(), 3);
        assert!(ops.iter().all(EditOp::is_substitute));
    }

    #[test]
    fn align_mixed_operations() {
        let ref_tokens = vec!["a", "b", "c", "d"];
        let hyp_tokens = vec!["a", "x", "c", "d", "e"];
        let ops = align(&ref_tokens, &hyp_tokens);
        assert_eq!(ops.len(), 5);
        assert!(ops[0].is_equal());
        assert!(ops[1].is_substitute());
        assert!(ops[2].is_equal());
        assert!(ops[3].is_equal());
        assert!(ops[4].is_insert());
    }

    #[test]
    fn alignment_counts() {
        let ref_tokens = vec!["a", "b", "c"];
        let hyp_tokens = vec!["a", "x", "c", "d"];
        let ops = align(&ref_tokens, &hyp_tokens);
        let counts = count_operations(&ops);
        assert_eq!(counts.hits, 2);
        assert_eq!(counts.substitutions, 1);
        assert_eq!(counts.deletions, 0);
        assert_eq!(counts.insertions, 1);
    }

    #[test]
    fn alignment_counts_empty() {
        let ops: Vec<EditOp> = vec![];
        let counts = count_operations(&ops);
        assert_eq!(counts.hits, 0);
        assert_eq!(counts.substitutions, 0);
        assert_eq!(counts.deletions, 0);
        assert_eq!(counts.insertions, 0);
    }

    #[test]
    fn alignment_counts_all_equal() {
        let ref_tokens = vec!["a", "b", "c"];
        let hyp_tokens = vec!["a", "b", "c"];
        let ops = align(&ref_tokens, &hyp_tokens);
        let counts = count_operations(&ops);
        assert_eq!(counts.hits, 3);
        assert_eq!(counts.substitutions, 0);
        assert_eq!(counts.deletions, 0);
        assert_eq!(counts.insertions, 0);
    }

    #[test]
    fn alignment_counts_all_insertions() {
        let ref_tokens: Vec<&str> = vec![];
        let hyp_tokens = vec!["a", "b"];
        let ops = align(&ref_tokens, &hyp_tokens);
        let counts = count_operations(&ops);
        assert_eq!(counts.hits, 0);
        assert_eq!(counts.insertions, 2);
    }

    #[test]
    fn alignment_counts_all_deletions() {
        let ref_tokens = vec!["a", "b"];
        let hyp_tokens: Vec<&str> = vec![];
        let ops = align(&ref_tokens, &hyp_tokens);
        let counts = count_operations(&ops);
        assert_eq!(counts.hits, 0);
        assert_eq!(counts.deletions, 2);
    }

    #[test]
    fn alignment_counts_all_substitutions() {
        let ref_tokens = vec!["a", "b"];
        let hyp_tokens = vec!["x", "y"];
        let ops = align(&ref_tokens, &hyp_tokens);
        let counts = count_operations(&ops);
        assert_eq!(counts.hits, 0);
        assert_eq!(counts.substitutions, 2);
    }

    #[test]
    fn operation_counts_default() {
        let counts = OperationCounts::default();
        assert_eq!(counts.hits, 0);
        assert_eq!(counts.substitutions, 0);
        assert_eq!(counts.deletions, 0);
        assert_eq!(counts.insertions, 0);
    }

    #[test]
    fn edit_op_equality() {
        let op1 = EditOp::Equal { index: 5 };
        let op2 = EditOp::Equal { index: 5 };
        assert_eq!(op1, op2);
    }

    #[test]
    fn edit_op_inequality() {
        let op1 = EditOp::Equal { index: 0 };
        let op2 = EditOp::Equal { index: 1 };
        assert_ne!(op1, op2);
    }

    #[test]
    fn edit_op_clone() {
        let op = EditOp::Substitute {
            ref_index: 3,
            hyp_index: 4,
        };
        let cloned = op.clone();
        assert_eq!(op, cloned);
    }

    #[test]
    fn align_with_string_types() {
        let ref_tokens = vec![String::from("hello"), String::from("world")];
        let hyp_tokens = vec![String::from("hello"), String::from("world")];
        let ops = align(&ref_tokens, &hyp_tokens);
        assert_eq!(ops.len(), 2);
        assert!(ops.iter().all(EditOp::is_equal));
    }

    // --- New tests for banded alignment ---

    #[test]
    fn edit_distance_basic() {
        let ref_tokens = vec!["hello", "world"];
        let hyp_tokens = vec!["hello", "earth"];
        assert_eq!(edit_distance(&ref_tokens, &hyp_tokens), 1);
    }

    #[test]
    fn edit_distance_identical() {
        let ref_tokens = vec!["a", "b", "c"];
        assert_eq!(edit_distance(&ref_tokens, &ref_tokens), 0);
    }

    #[test]
    fn edit_distance_empty() {
        assert_eq!(edit_distance::<&str>(&[], &[]), 0);
        assert_eq!(edit_distance::<&str>(&[], &["a"]), 1);
        assert_eq!(edit_distance::<&str>(&["a"], &[]), 1);
    }

    #[test]
    fn align_large_banded() {
        // Test with enough tokens that banded path is taken
        let ref_tokens: Vec<String> = (0..100).map(|i| format!("word{i}")).collect();
        let mut hyp_tokens = ref_tokens.clone();
        hyp_tokens[50] = "changed".to_string();
        let ops = align(&ref_tokens, &hyp_tokens);
        let counts = count_operations(&ops);
        assert_eq!(counts.hits, 99);
        assert_eq!(counts.substitutions, 1);
    }

    #[test]
    fn align_banded_many_deletions() {
        // ref much longer than hyp — exercises deletion-heavy banded path
        // and the diag=None / cur_val=None branches at band edges
        let ref_tokens: Vec<String> = (0..50).map(|i| format!("w{i}")).collect();
        let hyp_tokens: Vec<String> = (0..25).map(|i| format!("w{i}")).collect();
        let ops = align(&ref_tokens, &hyp_tokens);
        let counts = count_operations(&ops);
        assert_eq!(counts.hits, 25);
        assert_eq!(counts.deletions, 25);
        assert_eq!(counts.insertions, 0);
        assert_eq!(counts.substitutions, 0);
    }

    #[test]
    fn align_banded_many_insertions() {
        // hyp much longer than ref — exercises insertion-heavy banded path
        let ref_tokens: Vec<String> = (0..25).map(|i| format!("w{i}")).collect();
        let hyp_tokens: Vec<String> = (0..50).map(|i| format!("w{i}")).collect();
        let ops = align(&ref_tokens, &hyp_tokens);
        let counts = count_operations(&ops);
        assert_eq!(counts.hits, 25);
        assert_eq!(counts.insertions, 25);
        assert_eq!(counts.deletions, 0);
        assert_eq!(counts.substitutions, 0);
    }

    #[test]
    fn align_banded_deletions_at_start() {
        // Deletions at the start of reference
        let ref_tokens: Vec<&str> = vec!["a", "b", "c", "d"];
        let hyp_tokens: Vec<&str> = vec!["c", "d"];
        let ops = align(&ref_tokens, &hyp_tokens);
        let counts = count_operations(&ops);
        assert_eq!(counts.hits, 2);
        assert_eq!(counts.deletions, 2);
    }

    #[test]
    fn align_banded_insertions_at_end() {
        // Insertions at the end of hypothesis
        let ref_tokens: Vec<&str> = vec!["a", "b"];
        let hyp_tokens: Vec<&str> = vec!["a", "b", "c", "d"];
        let ops = align(&ref_tokens, &hyp_tokens);
        let counts = count_operations(&ops);
        assert_eq!(counts.hits, 2);
        assert_eq!(counts.insertions, 2);
    }

    #[test]
    fn align_banded_long_with_offset_substitution() {
        // Substitution far from diagonal — exercises band boundary branches
        let ref_tokens: Vec<String> = (0..200)
            .map(|i| {
                if i == 180 {
                    "wrong".to_string()
                } else {
                    format!("w{i}")
                }
            })
            .collect();
        let hyp_tokens: Vec<String> = (0..200).map(|i| format!("w{i}")).collect();
        let ops = align(&ref_tokens, &hyp_tokens);
        let counts = count_operations(&ops);
        assert_eq!(counts.hits, 199);
        assert_eq!(counts.substitutions, 1);
    }

    #[test]
    fn align_banded_mixed_operations_long() {
        // Long sequence with mixed ops to exercise all banded branches
        let ref_tokens: Vec<String> = (0..200)
            .map(|i| {
                if i == 100 {
                    "changed".to_string()
                } else {
                    format!("w{i}")
                }
            })
            .collect();
        let mut hyp_tokens: Vec<String> = (0..200).map(|i| format!("w{i}")).collect();
        hyp_tokens.insert(150, "extra".to_string());
        let ops = align(&ref_tokens, &hyp_tokens);
        let counts = count_operations(&ops);
        assert_eq!(counts.substitutions, 1);
        assert_eq!(counts.insertions, 1);
        assert_eq!(counts.hits, 199);
    }

    #[test]
    fn edit_distance_with_mixed_operations() {
        let ref_tokens = vec!["a", "b", "c", "d", "e"];
        let hyp_tokens = vec!["a", "x", "c", "e"];
        assert_eq!(edit_distance(&ref_tokens, &hyp_tokens), 2);
    }

    #[test]
    fn rapidfuzz_char_distance_identical() {
        assert_eq!(rapidfuzz_char_distance("hello".chars(), "hello".chars()), 0);
    }

    #[test]
    fn rapidfuzz_char_distance_substitution() {
        assert_eq!(rapidfuzz_char_distance("hello".chars(), "hallo".chars()), 1);
    }

    #[test]
    fn rapidfuzz_char_distance_empty_both() {
        assert_eq!(rapidfuzz_char_distance("".chars(), "".chars()), 0);
    }

    #[test]
    fn rapidfuzz_char_distance_empty_one() {
        assert_eq!(rapidfuzz_char_distance("abc".chars(), "".chars()), 3);
        assert_eq!(rapidfuzz_char_distance("".chars(), "abc".chars()), 3);
    }

    #[test]
    fn rapidfuzz_char_distance_all_different() {
        assert_eq!(rapidfuzz_char_distance("abc".chars(), "xyz".chars()), 3);
    }

    #[test]
    fn rapidfuzz_char_distance_unicode() {
        assert_eq!(rapidfuzz_char_distance("你好".chars(), "你好".chars()), 0);
        assert_eq!(rapidfuzz_char_distance("你好".chars(), "你们".chars()), 1);
    }

    #[test]
    fn rapidfuzz_char_distance_insertion() {
        assert_eq!(rapidfuzz_char_distance("ac".chars(), "abc".chars()), 1);
    }

    #[test]
    fn rapidfuzz_char_distance_deletion() {
        assert_eq!(rapidfuzz_char_distance("abc".chars(), "ac".chars()), 1);
    }

    // --- rapidfuzz_word_distance tests ---

    #[test]
    fn rapidfuzz_word_distance_identical() {
        let words = vec!["hello", "world"];
        assert_eq!(rapidfuzz_word_distance(&words, &words), 0);
    }

    #[test]
    fn rapidfuzz_word_distance_substitution() {
        let ref_words = vec!["hello", "world"];
        let hyp_words = vec!["hello", "earth"];
        assert_eq!(rapidfuzz_word_distance(&ref_words, &hyp_words), 1);
    }

    #[test]
    fn rapidfuzz_word_distance_empty() {
        assert_eq!(rapidfuzz_word_distance::<&str>(&[], &[]), 0);
        assert_eq!(rapidfuzz_word_distance::<&str>(&[], &["a"]), 1);
        assert_eq!(rapidfuzz_word_distance::<&str>(&["a"], &[]), 1);
    }

    #[test]
    fn rapidfuzz_word_distance_mixed_operations() {
        let ref_words = vec!["a", "b", "c", "d", "e"];
        let hyp_words = vec!["a", "x", "c", "e"];
        assert_eq!(rapidfuzz_word_distance(&ref_words, &hyp_words), 2);
    }

    #[test]
    fn rapidfuzz_word_distance_matches_edit_distance() {
        let ref_words: Vec<String> = (0..100).map(|i| format!("word{i}")).collect();
        let mut hyp_words = ref_words.clone();
        hyp_words[10] = "changed".to_string();
        hyp_words.remove(50);
        hyp_words.insert(75, "extra".to_string());
        assert_eq!(
            edit_distance(&ref_words, &hyp_words),
            rapidfuzz_word_distance(&ref_words, &hyp_words)
        );
    }

    #[test]
    fn rapidfuzz_word_distance_large() {
        let ref_words: Vec<String> = (0..1000).map(|i| format!("w{i}")).collect();
        let mut hyp_words = ref_words.clone();
        hyp_words[500] = "changed".to_string();
        assert_eq!(rapidfuzz_word_distance(&ref_words, &hyp_words), 1);
    }
}
