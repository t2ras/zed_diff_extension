fn backtrack_changes(
    lcs_matrix: &[Vec<usize>],
    original_lines: &[String],
    modified_lines: &[String],
    original_hashes: &[u64],
    modified_hashes: &[u64],
) -> Vec<LineChange> {
    let mut changes = Vec::new();
    let mut i = original_hashes.len();
    let mut j = modified_hashes.len();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && original_hashes[i - 1] == modified_hashes[j - 1] {
            // Lines match, no change
            i -= 1;
            j -= 1;
        } else if i > 0 && (j == 0 || lcs_matrix[i][j] == lcs_matrix[i - 1][j]) {
            // Deletion
            changes.push(LineChange {
                original_start: i - 1,
                original_end: i,
                modified_start: j,
                modified_end: j,
                change_type: ChangeType::Deleted,
                char_changes: None,
            });
            i -= 1;
        } else if j > 0 {
            // Insertion
            changes.push(LineChange {
                original_start: i,
                original_end: i,
                modified_start: j - 1,
                modified_end: j,
                change_type: ChangeType::Added,
                char_changes: None,
            });
            j -= 1;
        }
    }

    changes.reverse();
    merge_adjacent_changes(changes)
}

fn merge_adjacent_changes(changes: Vec<LineChange>) -> Vec<LineChange> {
    if changes.is_empty() {
        return changes;
    }

    let mut merged = Vec::new();
    let mut current = changes[0].clone();

    for change in changes.into_iter().skip(1) {
        if should_merge(&current, &change) {
            current.original_end = change.original_end;
            current.modified_end = change.modified_end;
            if current.change_type == ChangeType::Deleted 
                && change.change_type == ChangeType::Added {
                current.change_type = ChangeType::Modified;
            }
        } else {
            merged.push(current);
            current = change;
        }
    }
    merged.push(current);
    merged
}

fn should_merge(a: &LineChange, b: &LineChange) -> bool {
    // Merge adjacent deletions and insertions into modifications
    (a.change_type == ChangeType::Deleted && b.change_type == ChangeType::Added)
        || (a.change_type == b.change_type 
            && a.original_end == b.original_start 
            && a.modified_end == b.modified_start)
}

fn compute_character_changes(
    mut changes: Vec<LineChange>,
    original_lines: &[String],
    modified_lines: &[String],
) -> Vec<LineChange> {
    for change in &mut changes {
        if change.change_type == ChangeType::Modified {
            // Compute character-level diff for modified lines
            let orig_text = get_line_range(original_lines, change.original_start, change.original_end);
            let mod_text = get_line_range(modified_lines, change.modified_start, change.modified_end);

            change.char_changes = Some(compute_char_diff(&orig_text, &mod_text));
        }
    }
    changes
}

fn get_line_range(lines: &[String], start: usize, end: usize) -> String {
    lines[start..end].join("\n")
}

fn compute_char_diff(original: &str, modified: &str) -> Vec<CharChange> {
    // Simplified character-level diff
    let orig_chars: Vec<char> = original.chars().collect();
    let mod_chars: Vec<char> = modified.chars().collect();

    let m = orig_chars.len();
    let n = mod_chars.len();

    if m == 0 && n == 0 {
        return Vec::new();
    }

    // Simple character-level LCS
    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if orig_chars[i - 1] == mod_chars[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack to find character changes
    let mut char_changes = Vec::new();
    let mut i = m;
    let mut j = n;
    let mut del_start = None;
    let mut ins_start = None;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && orig_chars[i - 1] == mod_chars[j - 1] {
            // Flush pending changes
            if let (Some(ds), Some(is)) = (del_start, ins_start) {
                char_changes.push(CharChange {
                    original_start: ds,
                    original_length: i - ds,
                    modified_start: is,
                    modified_length: j - is,
                });
                del_start = None;
                ins_start = None;
            }
            i -= 1;
            j -= 1;
        } else if i > 0 && (j == 0 || dp[i][j] == dp[i - 1][j]) {
            if del_start.is_none() {
                del_start = Some(i - 1);
            }
            i -= 1;
        } else {
            if ins_start.is_none() {
                ins_start = Some(j - 1);
            }
            j -= 1;
        }
    }

    if let (Some(ds), Some(is)) = (del_start, ins_start) {
        char_changes.push(CharChange {
            original_start: ds,
            original_length: i - ds,
            modified_start: is,
            modified_length: j - is,
        });
    }

    char_changes.reverse();
    char_changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_files() {
        let lines1 = vec!["line1".to_string(), "line2".to_string()];
        let lines2 = vec!["line1".to_string(), "line2".to_string()];
        let options = DiffOptions {
            ignore_whitespace: false,
            ignore_case: false,
            max_computation_time_ms: 5000,
            compute_char_changes: false,
        };

        let changes = compute_diff(&lines1, &lines2, options);
        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn test_simple_addition() {
        let lines1 = vec!["line1".to_string()];
        let lines2 = vec!["line1".to_string(), "line2".to_string()];
        let options = DiffOptions {
            ignore_whitespace: false,
            ignore_case: false,
            max_computation_time_ms: 5000,
            compute_char_changes: false,
        };

        let changes = compute_diff(&lines1, &lines2, options);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::Added);
    }

    #[test]
    fn test_simple_deletion() {
        let lines1 = vec!["line1".to_string(), "line2".to_string()];
        let lines2 = vec!["line1".to_string()];
        let options = DiffOptions {
            ignore_whitespace: false,
            ignore_case: false,
            max_computation_time_ms: 5000,
            compute_char_changes: false,
        };

        let changes = compute_diff(&lines1, &lines2, options);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::Deleted);
    }
}
