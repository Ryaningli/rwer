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

    // Use full WF when banded approach won't save work
    if dist >= m + n {
        return align_full(reference, hypothesis);
    }

    align_banded(reference, hypothesis, dist)
}

/// Full Wagner-Fischer alignment (fallback for edge cases).
fn align_full<S: AsRef<str> + PartialEq>(reference: &[S], hypothesis: &[S]) -> Vec<EditOp> {
    let m = reference.len();
    let n = hypothesis.len();

    let mut dist = vec![vec![0usize; n + 1]; m + 1];
    for (i, row) in dist.iter_mut().enumerate().take(m + 1) {
        row[0] = i;
    }
    for (j, cell) in dist[0].iter_mut().enumerate().take(n + 1) {
        *cell = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = usize::from(reference[i - 1] != hypothesis[j - 1]);
            dist[i][j] = (dist[i - 1][j] + 1)
                .min(dist[i][j - 1] + 1)
                .min(dist[i - 1][j - 1] + cost);
        }
    }

    let mut ops = Vec::with_capacity(m + n);
    let (mut i, mut j) = (m, n);
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && reference[i - 1] == hypothesis[j - 1] {
            ops.push(EditOp::Equal { index: i - 1 });
            i -= 1;
            j -= 1;
        } else if i > 0 && j > 0 && dist[i][j] == dist[i - 1][j - 1] + 1 {
            ops.push(EditOp::Substitute {
                ref_index: i - 1,
                hyp_index: j - 1,
            });
            i -= 1;
            j -= 1;
        } else if j > 0 && dist[i][j] == dist[i][j - 1] + 1 {
            ops.push(EditOp::Insert { hyp_index: j - 1 });
            j -= 1;
        } else {
            ops.push(EditOp::Delete { ref_index: i - 1 });
            i -= 1;
        }
    }
    ops.reverse();
    ops
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

            let diag = if hyp_idx > prev_lo && hyp_idx - 1 <= prev_hi {
                Some(rows[ref_idx - 1][hyp_idx - 1 - prev_lo])
            } else {
                None
            };

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
            let diag_val = diag.map(|d| d + cost);

            row[local_j] = up
                .into_iter()
                .chain(left)
                .chain(diag_val)
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
        let cur_val = if hyp_pos >= lo_val && hyp_pos <= hi(ref_pos) {
            Some(rows[ref_pos][hyp_pos - lo_val])
        } else {
            None
        };

        if ref_pos > 0 && hyp_pos > 0 && reference[ref_pos - 1] == hypothesis[hyp_pos - 1] {
            ops.push(EditOp::Equal { index: ref_pos - 1 });
            ref_pos -= 1;
            hyp_pos -= 1;
        } else if let Some(cv) = cur_val {
            let prev_lo = lo(ref_pos.saturating_sub(1));
            let prev_hi = if ref_pos > 0 { hi(ref_pos - 1) } else { 0 };

            let diag_ok = ref_pos > 0
                && hyp_pos > 0
                && hyp_pos > prev_lo
                && hyp_pos - 1 <= prev_hi
                && cv == rows[ref_pos - 1][hyp_pos - 1 - prev_lo] + 1;
            let left_ok =
                hyp_pos > lo_val && cv == rows[ref_pos][hyp_pos - 1 - lo_val] + 1;

            if diag_ok {
                ops.push(EditOp::Substitute {
                    ref_index: ref_pos - 1,
                    hyp_index: hyp_pos - 1,
                });
                ref_pos -= 1;
                hyp_pos -= 1;
            } else if left_ok {
                ops.push(EditOp::Insert { hyp_index: hyp_pos - 1 });
                hyp_pos -= 1;
            } else {
                ops.push(EditOp::Delete { ref_index: ref_pos - 1 });
                ref_pos -= 1;
            }
        } else {
            ops.push(EditOp::Delete { ref_index: ref_pos - 1 });
            ref_pos -= 1;
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
}
