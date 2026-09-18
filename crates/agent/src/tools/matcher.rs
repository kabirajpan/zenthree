use super::patch::SearchReplaceBlock;

/// Executes the 3-Tier Matcher:
/// - Tier 1: Exact Substring Match (zero-allocation fast path)
/// - Tier 2: CRLF / LF line-ending normalization
/// - Tier 3: Line-trimmed & Indentation-tolerant matching with automatic re-indentation
pub fn match_and_replace(
    original: &str,
    blocks: &[SearchReplaceBlock],
) -> Result<String, String> {
    if blocks.is_empty() {
        return Ok(original.to_string());
    }

    let mut current_content = original.to_string();

    for (idx, block) in blocks.iter().enumerate() {
        let search = &block.search;
        let replace = &block.replace;

        if search.is_empty() {
            if current_content.trim().is_empty() {
                current_content = replace.clone();
                continue;
            }
            return Err(format!("Block #{}: Search block cannot be empty", idx + 1));
        }

        // Tier 1: Exact Substring Match
        if let Some(pos) = current_content.find(search) {
            let before = &current_content[..pos];
            let after = &current_content[pos + search.len()..];
            current_content = format!("{}{}{}", before, replace, after);
            continue;
        }

        // Tier 2: CRLF / LF line-ending normalization
        let norm_current = current_content.replace("\r\n", "\n");
        let norm_search = search.replace("\r\n", "\n");
        let norm_replace = replace.replace("\r\n", "\n");

        if let Some(pos) = norm_current.find(&norm_search) {
            let before = &norm_current[..pos];
            let after = &norm_current[pos + norm_search.len()..];
            current_content = format!("{}{}{}", before, norm_replace, after);
            continue;
        }

        // Tier 3: Line-trimmed / Indentation-tolerant matching
        let cur_lines: Vec<&str> = norm_current.lines().collect();
        let search_lines: Vec<&str> = norm_search.lines().collect();
        let search_len = search_lines.len();

        if search_len > 0 && cur_lines.len() >= search_len {
            let mut matched_line_idx = None;

            for i in 0..=(cur_lines.len() - search_len) {
                let mut matches = true;
                for j in 0..search_len {
                    if cur_lines[i + j].trim() != search_lines[j].trim() {
                        matches = false;
                        break;
                    }
                }
                if matches {
                    matched_line_idx = Some(i);
                    break;
                }
            }

            if let Some(start_idx) = matched_line_idx {
                let orig_indent = cur_lines[start_idx].len() - cur_lines[start_idx].trim_start().len();
                let search_indent = search_lines
                    .iter()
                    .find(|l| !l.trim().is_empty())
                    .map(|l| l.len() - l.trim_start().len())
                    .unwrap_or(0);
                let delta = orig_indent as isize - search_indent as isize;

                let mut adjusted_replace_lines = Vec::new();
                for r_line in norm_replace.lines() {
                    if r_line.trim().is_empty() {
                        adjusted_replace_lines.push(String::new());
                    } else if delta > 0 {
                        adjusted_replace_lines.push(format!("{}{}", " ".repeat(delta as usize), r_line));
                    } else if delta < 0 {
                        let to_strip = (-delta) as usize;
                        let cur_r_indent = r_line.len() - r_line.trim_start().len();
                        let strip = to_strip.min(cur_r_indent);
                        adjusted_replace_lines.push(r_line[strip..].to_string());
                    } else {
                        adjusted_replace_lines.push(r_line.to_string());
                    }
                }

                let mut new_lines = Vec::new();
                new_lines.extend_from_slice(&cur_lines[..start_idx]);
                for r in &adjusted_replace_lines {
                    new_lines.push(r.as_str());
                }
                new_lines.extend_from_slice(&cur_lines[start_idx + search_len..]);

                let trailing_newline = norm_current.ends_with('\n');
                let mut joined = new_lines.join("\n");
                if trailing_newline && !joined.ends_with('\n') {
                    joined.push('\n');
                }

                current_content = joined;
                continue;
            }
        }

        return Err(format!(
            "Failed to find search pattern (block #{}):\n```\n{}\n```",
            idx + 1,
            search
        ));
    }

    Ok(current_content)
}
