use crate::diff_core::{LineChange, ChangeType};

pub fn format_unified_diff(
    file1_path: &str,
    file2_path: &str,
    changes: &[LineChange],
) -> String {
    let mut output = String::new();

    output.push_str(&format!("--- {}\n", file1_path));
    output.push_str(&format!("+++ {}\n", file2_path));

    if changes.is_empty() {
        output.push_str("\nFiles are identical\n");
        return output;
    }

    for change in changes {
        let original_range = format_range(change.original_start, change.original_end);
        let modified_range = format_range(change.modified_start, change.modified_end);

        output.push_str(&format!("\n@@ -{} +{} @@\n", original_range, modified_range));

        match change.change_type {
            ChangeType::Added => {
                output.push_str(&format!("+{} line(s) added\n", 
                    change.modified_end - change.modified_start));
            }
            ChangeType::Deleted => {
                output.push_str(&format!("-{} line(s) deleted\n", 
                    change.original_end - change.original_start));
            }
            ChangeType::Modified => {
                output.push_str(&format!("~{} line(s) modified\n", 
                    change.original_end - change.original_start));

                if let Some(ref char_changes) = change.char_changes {
                    output.push_str(&format!("  ({} character-level changes)\n", 
                        char_changes.len()));
                }
            }
        }
    }

    output
}

fn format_range(start: usize, end: usize) -> String {
    let count = end - start;
    if count == 0 {
        format!("{},0", start)
    } else if count == 1 {
        format!("{}", start + 1)
    } else {
        format!("{},{}", start + 1, count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_range() {
        assert_eq!(format_range(0, 0), "0,0");
        assert_eq!(format_range(0, 1), "1");
        assert_eq!(format_range(5, 10), "6,5");
    }
}
