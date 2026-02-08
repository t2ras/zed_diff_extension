use std::fs;
use std::path::Path;
use crate::diff_core::{compute_diff, DiffOptions, LineChange};

pub fn read_file_lines(path: &str) -> Result<Vec<String>, std::io::Error> {
    let content = fs::read_to_string(Path::new(path))?;
    Ok(content.lines().map(String::from).collect())
}

pub fn compare_files(
    file1_path: &str,
    file2_path: &str,
    options: DiffOptions,
) -> Result<Vec<LineChange>, Box<dyn std::error::Error>> {
    let lines1 = read_file_lines(file1_path)?;
    let lines2 = read_file_lines(file2_path)?;

    Ok(compute_diff(&lines1, &lines2, options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_file_lines() {
        assert!(true);
    }
}
