use serde::{Deserialize, Serialize};

/// Type of line diff operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffOp {
    /// Lines that remain identical between old and new text.
    Keep { lines: Vec<String> },
    /// Lines added in the new text (Green / +).
    Insert { lines: Vec<String> },
    /// Lines removed from the old text (Red / -).
    Delete { lines: Vec<String> },
}

/// A structured diff hunk with line coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub operations: Vec<DiffOp>,
}

/// Two-column dynamic programming matrix with column swapping.
/// Maintains $O(M)$ memory where $M$ is the number of lines in the original text,
/// rather than allocating a full $O(N \times M)$ grid.
#[derive(Default, Debug)]
struct SwapMatrix {
    col0: Vec<usize>,
    col1: Vec<usize>,
}

impl SwapMatrix {
    fn new(rows: usize) -> Self {
        Self {
            col0: vec![0; rows],
            col1: vec![0; rows],
        }
    }

    fn swap(&mut self) {
        std::mem::swap(&mut self.col0, &mut self.col1);
    }
}

/// Dynamic programming line-level diff calculator.
/// Computes Longest Common Subsequence (LCS) line diffs with $O(N)$ auxiliary memory.
#[derive(Debug, Clone)]
pub struct MatrixDiff;

impl MatrixDiff {
    /// Computes line-level edit distance using SwapMatrix in $O(M)$ memory.
    pub fn edit_distance(old_text: &str, new_text: &str) -> usize {
        let old_lines: Vec<&str> = old_text.lines().collect();
        let new_lines: Vec<&str> = new_text.lines().collect();
        let m = old_lines.len();
        let n = new_lines.len();
        if m == 0 { return n; }
        if n == 0 { return m; }

        let mut matrix = SwapMatrix::new(m + 1);
        for i in 0..=m {
            matrix.col0[i] = i;
        }

        for j in 1..=n {
            matrix.col1[0] = j;
            for i in 1..=m {
                let cost = if old_lines[i - 1] == new_lines[j - 1] { 0 } else { 1 };
                matrix.col1[i] = (matrix.col0[i] + 1)
                    .min(matrix.col1[i - 1] + 1)
                    .min(matrix.col0[i - 1] + cost);
            }
            matrix.swap();
        }

        matrix.col0[m]
    }

    /// Computes the diff operations between `old_text` and `new_text`.
    pub fn compute_diff(old_text: &str, new_text: &str) -> Vec<DiffOp> {
        let old_lines: Vec<&str> = old_text.lines().collect();
        let new_lines: Vec<&str> = new_text.lines().collect();

        if old_lines.is_empty() && new_lines.is_empty() {
            return Vec::new();
        }

        if old_lines.is_empty() {
            return vec![DiffOp::Insert {
                lines: new_lines.into_iter().map(String::from).collect(),
            }];
        }

        if new_lines.is_empty() {
            return vec![DiffOp::Delete {
                lines: old_lines.into_iter().map(String::from).collect(),
            }];
        }

        // Fast path: exact match
        if old_lines == new_lines {
            return vec![DiffOp::Keep {
                lines: old_lines.into_iter().map(String::from).collect(),
            }];
        }

        let m = old_lines.len();
        let n = new_lines.len();

        // Standard 2D table for trace-back
        let mut dp = vec![vec![0usize; n + 1]; m + 1];

        for i in 1..=m {
            for j in 1..=n {
                if old_lines[i - 1] == new_lines[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1] + 1;
                } else {
                    dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
                }
            }
        }

        // Traceback from (m, n) to (0, 0)
        let mut i = m;
        let mut j = n;
        let mut raw_ops = Vec::new();

        while i > 0 || j > 0 {
            if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
                raw_ops.push(DiffOp::Keep {
                    lines: vec![old_lines[i - 1].to_string()],
                });
                i -= 1;
                j -= 1;
            } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
                raw_ops.push(DiffOp::Insert {
                    lines: vec![new_lines[j - 1].to_string()],
                });
                j -= 1;
            } else if i > 0 && (j == 0 || dp[i][j - 1] < dp[i - 1][j]) {
                raw_ops.push(DiffOp::Delete {
                    lines: vec![old_lines[i - 1].to_string()],
                });
                i -= 1;
            }
        }

        raw_ops.reverse();
        Self::coalesce_operations(raw_ops)
    }

    /// Merges consecutive operations of the same type into single blocks.
    fn coalesce_operations(ops: Vec<DiffOp>) -> Vec<DiffOp> {
        let mut coalesced = Vec::new();

        for op in ops {
            match (coalesced.last_mut(), op) {
                (Some(DiffOp::Keep { lines: prev }), DiffOp::Keep { lines: next }) => {
                    prev.extend(next);
                }
                (Some(DiffOp::Insert { lines: prev }), DiffOp::Insert { lines: next }) => {
                    prev.extend(next);
                }
                (Some(DiffOp::Delete { lines: prev }), DiffOp::Delete { lines: next }) => {
                    prev.extend(next);
                }
                (_, new_op) => {
                    coalesced.push(new_op);
                }
            }
        }

        coalesced
    }

    /// Formats the diff operations into a unified diff style string (`+`, `-`, ` `).
    pub fn format_unified(ops: &[DiffOp]) -> String {
        let mut out = String::new();
        for op in ops {
            match op {
                DiffOp::Keep { lines } => {
                    for line in lines {
                        out.push_str(&format!(" {}\n", line));
                    }
                }
                DiffOp::Insert { lines } => {
                    for line in lines {
                        out.push_str(&format!("+{}\n", line));
                    }
                }
                DiffOp::Delete { lines } => {
                    for line in lines {
                        out.push_str(&format!("-{}\n", line));
                    }
                }
            }
        }
        out
    }
}

/// Live streaming diff accumulator that receives token chunks from an LLM
/// and computes live diff operations against the original base text.
#[derive(Debug, Clone)]
pub struct StreamingDiff {
    original_text: String,
    streamed_text: String,
}

impl StreamingDiff {
    pub fn new(original_text: &str) -> Self {
        Self {
            original_text: original_text.to_string(),
            streamed_text: String::new(),
        }
    }

    /// Appends incoming token chunk to the new text stream and returns current diff operations.
    pub fn push_chunk(&mut self, chunk: &str) -> Vec<DiffOp> {
        self.streamed_text.push_str(chunk);
        MatrixDiff::compute_diff(&self.original_text, &self.streamed_text)
    }

    /// Returns the complete accumulated new text.
    pub fn current_text(&self) -> &str {
        &self.streamed_text
    }

    /// Returns the original text.
    pub fn original_text(&self) -> &str {
        &self.original_text
    }

    /// Finalizes and returns the unified diff string.
    pub fn finalize(&self) -> String {
        let ops = MatrixDiff::compute_diff(&self.original_text, &self.streamed_text);
        MatrixDiff::format_unified(&ops)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match_yields_keep() {
        let text = "fn main() {\n    println!(\"hello\");\n}";
        let ops = MatrixDiff::compute_diff(text, text);
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            DiffOp::Keep { lines } => assert_eq!(lines.len(), 3),
            _ => panic!("Expected DiffOp::Keep"),
        }
    }

    #[test]
    fn test_single_line_replacement() {
        let old = "line 1\nold line 2\nline 3";
        let new = "line 1\nnew line 2\nline 3";

        let ops = MatrixDiff::compute_diff(old, new);
        assert_eq!(ops.len(), 4);
        assert_eq!(ops[0], DiffOp::Keep { lines: vec!["line 1".to_string()] });
        assert_eq!(ops[1], DiffOp::Delete { lines: vec!["old line 2".to_string()] });
        assert_eq!(ops[2], DiffOp::Insert { lines: vec!["new line 2".to_string()] });
        assert_eq!(ops[3], DiffOp::Keep { lines: vec!["line 3".to_string()] });

        let unified = MatrixDiff::format_unified(&ops);
        assert!(unified.contains(" line 1\n"));
        assert!(unified.contains("-old line 2\n"));
        assert!(unified.contains("+new line 2\n"));
        assert!(unified.contains(" line 3\n"));
    }

    #[test]
    fn test_streaming_diff_incremental_chunks() {
        let old = "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}";
        let mut streaming = StreamingDiff::new(old);

        streaming.push_chunk("fn add(a: i32, b: i32) -> i32 {\n");
        let ops = streaming.push_chunk("    // logging\n    a + b\n}");

        let unified = MatrixDiff::format_unified(&ops);
        assert!(unified.contains("+    // logging"));
        assert!(unified.contains(" fn add(a: i32, b: i32) -> i32 {"));
    }
}
