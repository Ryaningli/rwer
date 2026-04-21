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

/// Compute the Levenshtein alignment between two token sequences using the Wagner-Fischer algorithm.
///
/// Returns a vector of [`EditOp`] representing the optimal alignment.
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

    // DP table: dist[i][j] = edit distance between ref[..i] and hyp[..j]
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

    // Backtrack to reconstruct alignment
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
        assert!(ops[0].is_equal()); // a
        assert!(ops[1].is_substitute()); // b → x
        assert!(ops[2].is_equal()); // c
        assert!(ops[3].is_equal()); // d
        assert!(ops[4].is_insert()); // e
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
}
