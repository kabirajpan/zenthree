/// Truncates large tool outputs to prevent prompt blowup and token exhaustion
pub fn truncate_output(text: &str, max_lines: usize, max_chars: usize) -> String {
    if text.len() <= max_chars && text.lines().count() <= max_lines {
        return text.to_string();
    }

    let mut result = Vec::new();
    let mut current_chars = 0;
    let mut line_count = 0;

    for line in text.lines() {
        if line_count >= max_lines || current_chars + line.len() > max_chars {
            let remaining_lines = text.lines().count() - line_count;
            result.push(format!(
                "... [{} additional lines truncated ({}) - refine query or read line range] ...",
                remaining_lines,
                if text.len() > max_chars { format!("{} total chars", text.len()) } else { "".to_string() }
            ));
            break;
        }
        result.push(line.to_string());
        current_chars += line.len() + 1;
        line_count += 1;
    }

    result.join("\n")
}
