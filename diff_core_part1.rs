use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct DiffOptions {
    pub ignore_whitespace: bool,
    pub ignore_case: bool,
    pub max_computation_time_ms: u64,
    pub compute_char_changes: bool,
}

#[derive(Clone, Debug)]
pub struct LineChange {
    pub original_start: usize,
    pub original_end: usize,
    pub modified_start: usize,
    pub modified_end: usize,
    pub change_type: ChangeType,
    pub char_changes: Option<Vec<CharChange>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChangeType {
    Added,
    Deleted,
    Modified,
}

#[derive(Clone, Debug)]
pub struct CharChange {
    pub original_start: usize,
    pub original_length: usize,
    pub modified_start: usize,
    pub modified_length: usize,
}

/// Compute diff between two sets of lines using Myers algorithm
pub fn compute_diff(
    original_lines: &[String],
    modified_lines: &[String],
    options: DiffOptions,
) -> Vec<LineChange> {
    let start_time = Instant::now();
    let timeout = Duration::from_millis(options.max_computation_time_ms);

    // Preprocess lines based on options
    let processed_original = preprocess_lines(original_lines, &options);
    let processed_modified = preprocess_lines(modified_lines, &options);

    // Build line hash map for faster comparison
    let original_hashes = hash_lines(&processed_original);
    let modified_hashes = hash_lines(&processed_modified);

    // Compute LCS using Myers algorithm with DP
    let lcs_matrix = compute_lcs_matrix(
        &original_hashes,
        &modified_hashes,
        start_time,
        timeout,
    );

    // Backtrack to find changes
    let changes = backtrack_changes(
        &lcs_matrix,
        original_lines,
        modified_lines,
        &original_hashes,
        &modified_hashes,
    );

    // Compute character-level changes if requested
    if options.compute_char_changes {
        compute_character_changes(changes, original_lines, modified_lines)
    } else {
        changes
    }
}

fn preprocess_lines(lines: &[String], options: &DiffOptions) -> Vec<String> {
    lines
        .iter()
        .map(|line| {
            let mut processed = line.clone();
            if options.ignore_whitespace {
                processed = processed.trim().to_string();
            }
            if options.ignore_case {
                processed = processed.to_lowercase();
            }
            processed
        })
        .collect()
}

fn hash_lines(lines: &[String]) -> Vec<u64> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    lines
        .iter()
        .map(|line| {
            let mut hasher = DefaultHasher::new();
            line.hash(&mut hasher);
            hasher.finish()
        })
        .collect()
}

fn compute_lcs_matrix(
    original_hashes: &[u64],
    modified_hashes: &[u64],
    start_time: Instant,
    timeout: Duration,
) -> Vec<Vec<usize>> {
    let m = original_hashes.len();
    let n = modified_hashes.len();

    // Create DP matrix (m+1) x (n+1)
    let mut dp = vec![vec![0; n + 1]; m + 1];

    // Fill DP matrix
    for i in 1..=m {
        // Check timeout
        if start_time.elapsed() > timeout {
            break;
        }

        for j in 1..=n {
            if original_hashes[i - 1] == modified_hashes[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    dp
}
