use zed_extension_api as zed;

mod diff_core;
mod file_handler;
mod ui;

use diff_core::{DiffOptions, LineChange};
use file_handler::compare_files;
use ui::format_unified_diff;

struct DiffExtension {
    comparison_state: Option<ComparisonState>,
}

struct ComparisonState {
    file1_path: String,
    file2_path: String,
    diff_result: Vec<LineChange>,
}

impl zed::Extension for DiffExtension {
    fn new() -> Self {
        Self {
            comparison_state: None,
        }
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        Ok(zed::Command {
            command: "diff".into(),
            args: vec![],
            env: Default::default(),
        })
    }
}

impl DiffExtension {
    pub fn compare_two_files(&mut self, file1: String, file2: String) -> Result<String, String> {
        let options = DiffOptions {
            ignore_whitespace: false,
            ignore_case: false,
            max_computation_time_ms: 5000,
            compute_char_changes: true,
        };

        match compare_files(&file1, &file2, options) {
            Ok(changes) => {
                self.comparison_state = Some(ComparisonState {
                    file1_path: file1.clone(),
                    file2_path: file2.clone(),
                    diff_result: changes.clone(),
                });

                Ok(format_unified_diff(&file1, &file2, &changes))
            }
            Err(e) => Err(format!("Failed to compare files: {}", e)),
        }
    }
}

zed::register_extension!(DiffExtension);
