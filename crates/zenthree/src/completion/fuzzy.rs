/// Evaluates whether `candidate` matches `query` and computes a relevance score.
/// Higher score means better match. Returns `None` if the candidate does not match.
pub fn fuzzy_match_score(query: &str, candidate: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(100);
    }
    if candidate.is_empty() {
        return None;
    }

    // 1. Exact match (case-sensitive or insensitive)
    if candidate == query {
        return Some(2000);
    }
    if candidate.eq_ignore_ascii_case(query) {
        return Some(1800);
    }

    // 2. Exact case-sensitive prefix match (highest real-world relevance)
    if candidate.starts_with(query) {
        return Some(1000 + (100 - (candidate.len() - query.len()) as i32).max(0));
    }

    // 3. Case-insensitive prefix match
    let q_lower = query.to_lowercase();
    let c_lower = candidate.to_lowercase();
    if c_lower.starts_with(&q_lower) {
        return Some(850 + (100 - (candidate.len() - query.len()) as i32).max(0));
    }

    // 4. Word-boundary matches: e.g. 'str' matches 'my_string', 'read_stream', 'io.stream'
    if let Some(boundary_score) = match_word_boundary(&q_lower, candidate) {
        return Some(boundary_score);
    }

    // 5. JetBrains CamelHump Matching:
    // e.g. 'HM' matches 'HashMap', 'tb' or 'TB' matches 'total_bytes'
    if let Some(hump_score) = match_camel_hump(query, candidate) {
        return Some(hump_score);
    }

    // For 1-character queries, only match exact prefix, word boundary, or camel-hump.
    // Do NOT allow scattered internal matches (e.g. 'h' must never match 'whilelet' or 'match').
    if query.len() == 1 {
        return None;
    }

    // 6. General Subsequence Fuzzy Match with contiguous streak bonus and distance penalty
    let mut q_chars = query.chars().peekable();
    let mut score = 400;
    let mut consecutive = 0;
    let mut first_match_idx: Option<usize> = None;
    let mut last_match_idx: usize = 0;

    for (c_idx, c) in candidate.chars().enumerate() {
        if let Some(&next_q) = q_chars.peek() {
            if c.eq_ignore_ascii_case(&next_q) {
                q_chars.next();
                if first_match_idx.is_none() {
                    first_match_idx = Some(c_idx);
                } else {
                    let distance = c_idx.saturating_sub(last_match_idx);
                    if distance > 1 {
                        score -= (distance as i32 - 1).min(20);
                    }
                }
                last_match_idx = c_idx;
                consecutive += 1;
                score += 20 * consecutive; // Reward contiguous streaks
            } else {
                consecutive = 0;
            }
        }
    }

    // All query characters matched
    if q_chars.peek().is_none() {
        let first_idx = first_match_idx.unwrap_or(0) as i32;
        let len_penalty = (candidate.len() as i32 - query.len() as i32).min(50);
        return Some((score - first_idx * 5 - len_penalty).max(100));
    }

    None
}

/// Matches word boundaries after '_', '.', '-', '/', or uppercase transitions.
fn match_word_boundary(query_lower: &str, candidate: &str) -> Option<i32> {
    let mut pos = 0;
    let bytes = candidate.as_bytes();
    while pos < bytes.len() {
        let is_boundary = pos == 0
            || bytes[pos - 1] == b'_'
            || bytes[pos - 1] == b'.'
            || bytes[pos - 1] == b'-'
            || bytes[pos - 1] == b'/'
            || (bytes[pos].is_ascii_uppercase() && !bytes[pos - 1].is_ascii_uppercase());

        if is_boundary {
            let slice = &candidate[pos..];
            if slice.len() >= query_lower.len()
                && slice[..query_lower.len()].eq_ignore_ascii_case(query_lower)
            {
                let start_penalty = (pos as i32).min(100);
                return Some(750 - start_penalty);
            }
        }
        pos += 1;
    }
    None
}

/// Matches JetBrains CamelHump abbreviations:
/// E.g. "HM" -> "HashMap", "FSR" -> "FileStreamReader", "pb" -> "point_buffer"
fn match_camel_hump(query: &str, candidate: &str) -> Option<i32> {
    let mut humps = Vec::new();
    let chars: Vec<char> = candidate.chars().collect();

    for i in 0..chars.len() {
        let c = chars[i];
        if i == 0 || c.is_uppercase() || (i > 0 && chars[i - 1] == '_') {
            if c != '_' {
                humps.push(c);
            }
        }
    }

    let q_chars: Vec<char> = query.chars().collect();
    if q_chars.len() <= humps.len() {
        let mut q_idx = 0;
        for &h in &humps {
            if q_idx < q_chars.len() && h.eq_ignore_ascii_case(&q_chars[q_idx]) {
                q_idx += 1;
            }
        }
        if q_idx == q_chars.len() {
            return Some(700 + (100 - (humps.len() - q_chars.len()) as i32).max(0));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_matches() {
        assert!(fuzzy_match_score("to_", "to_string").unwrap() > 900);
        assert!(fuzzy_match_score("to_", "to_lowercase").unwrap() > 900);
    }

    #[test]
    fn test_camel_hump() {
        assert!(fuzzy_match_score("HM", "HashMap").is_some());
        assert!(fuzzy_match_score("tb", "total_bytes").is_some());
    }

    #[test]
    fn test_fuzzy_subsequence() {
        assert!(fuzzy_match_score("fnd", "find_node_depth").is_some());
        assert!(fuzzy_match_score("xyz", "find_node_depth").is_none());
    }

    #[test]
    fn test_word_boundary_matches() {
        assert!(fuzzy_match_score("str", "my_string").is_some());
        assert!(fuzzy_match_score("stream", "read_stream").is_some());
        assert!(fuzzy_match_score("path", "io.path.resolve").is_some());
    }
}
