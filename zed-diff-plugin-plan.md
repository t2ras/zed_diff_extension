# Plan for Building a File Comparison Plugin for Zed Editor

## Executive Summary

This document outlines a comprehensive plan to develop a file comparison (diff) plugin for the Zed editor, based on analysis of VSCode's diff implementation and Zed's extension architecture.

---

## 1. Understanding VSCode's File Comparison Tool

### 1.1 Core Architecture

VSCode's file comparison system consists of several key components:

1. **Diff Algorithm Implementation**
   - Based on **Myers diff algorithm** (Eugene W. Myers' "An O(ND) Difference Algorithm and its Variations")
   - Solves the "longest common subsequence" (LCS) problem
   - Implements dynamic programming approach for optimal performance
   - Finds minimal set of changes (insertions, deletions) to transform one file into another

2. **Two Algorithm Versions in VSCode**
   - **Legacy Algorithm** (`DiffComputer`): Traditional line-by-line comparison
   - **Advanced Algorithm** (`AdvancedLinesDiffComputer`): Introduced in VSCode 1.71.0, includes move detection

3. **Key Features**
   - Line-by-line comparison with character-level diff highlighting
   - Ignore whitespace options (leading/trailing)
   - Timeout limits for large files (5-second hard limit)
   - Web Worker execution to prevent UI blocking
   - Side-by-side and inline diff views

### 1.2 Algorithm Details

**Input:**
- Original lines: Array of strings (each line of original file)
- Modified lines: Array of strings (each line of modified file)
- Options:
  - `shouldIgnoreTrimWhitespace`: Ignore leading/trailing spaces
  - `shouldComputeCharChanges`: Compute character-level changes
  - `shouldPostProcessCharChanges`: Refine character changes
  - `shouldMakePrettyDiff`: Enhance readability
  - `maxComputationTime`: Timeout in milliseconds

**Output:**
- `ILineChange[]` array containing:
  - `originalStartLineNumber`
  - `originalEndLineNumber`
  - `modifiedStartLineNumber`
  - `modifiedEndLineNumber`
  - Character-level changes (optional)

**Algorithm Steps:**
1. Create (m+1) × (n+1) matrix for dynamic programming
2. Fill matrix using LCS principles
3. Backtrack to identify differences
4. Post-process for character-level changes
5. Apply prettification heuristics

### 1.3 VSCode Implementation Stack

- **Language**: TypeScript (95.8% of codebase)
- **Execution**: Web Workers for async computation
- **UI Components**: 
  - Diff Editor (side-by-side view)
  - Inline decorations
  - SCM (Source Control Management) integration
  - Merge Editor (3-way diff)
- **API**: Command `vscode.diff` to open comparison

---

## 2. Understanding Zed Editor Extension System

### 2.1 Extension Architecture

Zed uses a **WebAssembly (Wasm)** based extension system:

1. **Language**: Extensions written in **Rust**
2. **API**: `zed_extension_api` crate from crates.io
3. **Compilation**: Extensions compiled to Wasm modules
4. **Distribution**: Wasm binaries hosted on zed.dev
5. **Isolation**: Sandboxed execution environment

### 2.2 Extension Structure

```rust
use zed_extension_api as zed;

struct MyExtension {
    // Extension state
}

impl zed::Extension for MyExtension {
    // Extension methods
}

zed::register_extension!(MyExtension);
```

### 2.3 Key Extension Capabilities

Based on `zed_extension_api` documentation:

- **HTTP Client**: Make network requests
- **LSP Integration**: Language Server Protocol support
- **Process Management**: Execute external processes
- **Settings Access**: Read/write Zed configuration
- **File System**: Limited file operations (security sandboxed)

### 2.4 Development Workflow

1. Create extension directory with `extension.toml` manifest
2. Create `src/lib.rs` with extension implementation
3. Use `zed: install dev extension` command for local testing
4. View debug output with `--foreground` flag
5. Publish to Zed extension registry

### 2.5 Current Limitation

**Critical Finding**: As of February 2026, Zed does **not** have native file comparison functionality (GitHub Issue #17100 from August 2024 still open).

---

## 3. Technical Challenges & Considerations

### 3.1 Zed API Limitations

**Challenge**: Zed's extension API is more limited than VSCode's:

- No direct access to editor buffer manipulation APIs (yet)
- No built-in UI components for side-by-side views
- Limited text decoration capabilities compared to VSCode
- No web worker equivalent (Wasm runs in main thread with time limits)

**Implications**: 
- May need to work with Zed core team on API extensions
- Initial version might have limited UI capabilities
- Performance optimization critical due to main thread execution

### 3.2 Architecture Decisions

**Option A: Pure Extension Approach**
- Implement diff algorithm entirely in extension
- Use available Zed APIs for display
- **Pros**: No Zed core changes needed, faster iteration
- **Cons**: Limited UI capabilities, performance concerns

**Option B: Hybrid Approach (Recommended)**
- Contribute core diff functionality to Zed itself
- Build extension as UI/UX layer on top
- **Pros**: Better performance, richer UI, benefits all users
- **Cons**: Requires collaboration with Zed team, longer timeline

**Option C: External Tool Integration**
- Shell out to external diff tools (diff, git diff)
- Parse results and display in Zed
- **Pros**: Leverage existing tools, simpler implementation
- **Cons**: Platform dependencies, slower, limited customization

### 3.3 Performance Considerations

- Large files (>3500 lines) require optimization
- Hashing techniques for line comparison
- Timeout mechanisms to prevent freezing
- Incremental diff for real-time updates (future enhancement)

---

## 4. Detailed Implementation Plan

### Phase 1: Research & Setup (Week 1-2)

#### 4.1 Environment Setup
- [ ] Install Rust and cargo
- [ ] Install Zed editor
- [ ] Set up Zed extension development environment
- [ ] Clone and study VSCode diff implementation
  - Repository: `https://github.com/microsoft/vscode`
  - Key files to review:
    - `/src/vs/editor/common/diff/`
    - `/src/vs/editor/browser/widget/diffEditor/`

#### 4.2 Deep Dive Research
- [ ] Study `vscode-diff` npm package (extracted algorithm)
  - Repository: `https://github.com/micnil/vscode-diff`
- [ ] Review Myers diff algorithm paper
- [ ] Analyze Zed's extension API capabilities
  - Documentation: `https://docs.rs/zed_extension_api`
- [ ] Join Zed community (Discord/GitHub) to discuss approach

#### 4.3 Prototype Planning
- [ ] Create proof-of-concept decision matrix
- [ ] Identify API gaps and propose enhancements to Zed team
- [ ] Document technical requirements

---

### Phase 2: Core Algorithm Implementation (Week 3-5)

#### 4.4 Diff Algorithm Module

Create a Rust crate for the diff algorithm:

```rust
// diff_algorithm/src/lib.rs

pub struct DiffOptions {
    pub ignore_whitespace: bool,
    pub ignore_case: bool,
    pub max_computation_time_ms: u64,
}

pub struct LineChange {
    pub original_start: usize,
    pub original_end: usize,
    pub modified_start: usize,
    pub modified_end: usize,
    pub char_changes: Option<Vec<CharChange>>,
}

pub struct CharChange {
    pub original_start: usize,
    pub original_length: usize,
    pub modified_start: usize,
    pub modified_length: usize,
}

pub fn compute_diff(
    original_lines: &[String],
    modified_lines: &[String],
    options: DiffOptions,
) -> Vec<LineChange> {
    // Myers diff algorithm implementation
    // 1. Build DP matrix
    // 2. Compute LCS
    // 3. Backtrack to find changes
    // 4. Post-process for character changes
}
```

**Implementation Steps:**

1. **Basic Myers Algorithm**
   - Implement core DP matrix computation
   - Line-by-line comparison
   - Basic change detection

2. **Optimization Layer**
   - Line hashing for faster comparison
   - Timeout handling
   - Memory-efficient matrix storage (only keep necessary rows)

3. **Character-Level Diff**
   - For changed lines, compute character differences
   - Use same algorithm recursively on character arrays
   - Apply prettification heuristics

4. **Unit Tests**
   - Test cases from VSCode test suite
   - Edge cases: empty files, identical files, completely different files
   - Performance tests with large files

---

### Phase 3: Zed Extension Development (Week 6-8)

#### 4.5 Extension Structure

```
zed-diff-plugin/
├── Cargo.toml
├── extension.toml
└── src/
    ├── lib.rs          # Extension entry point
    ├── diff_core.rs    # Diff algorithm
    ├── file_handler.rs # File reading/comparison
    └── ui.rs           # Display logic
```

#### 4.6 Extension Manifest (`extension.toml`)

```toml
id = "file-diff-comparison"
name = "File Diff Comparison"
description = "Compare two files side-by-side with diff highlighting"
version = "0.1.0"
schema_version = 1
authors = ["Your Name <email@example.com>"]
repository = "https://github.com/yourusername/zed-diff-plugin"
```

#### 4.7 Core Extension Implementation

```rust
// src/lib.rs
use zed_extension_api as zed;

struct DiffExtension {
    // Store comparison state
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

    // Implement extension lifecycle methods
}

zed::register_extension!(DiffExtension);
```

#### 4.8 Command Registration

Implement commands:
- `diff: Select First File` - Mark first file for comparison
- `diff: Compare With Selected` - Compare current file with marked file
- `diff: Compare Two Files` - Open file picker for both files
- `diff: Show Diff Panel` - Toggle diff view

#### 4.9 File Reading & Processing

```rust
// src/file_handler.rs

use std::fs;
use std::path::Path;

pub fn read_file_lines(path: &Path) -> Result<Vec<String>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    Ok(content.lines().map(String::from).collect())
}

pub async fn compare_files(
    file1: &Path,
    file2: &Path,
    options: DiffOptions,
) -> Result<Vec<LineChange>, Box<dyn std::error::Error>> {
    let lines1 = read_file_lines(file1)?;
    let lines2 = read_file_lines(file2)?;
    
    Ok(compute_diff(&lines1, &lines2, options))
}
```

---

### Phase 4: User Interface (Week 9-11)

#### 4.10 Display Strategy

Since Zed's API may be limited, consider multiple approaches:

**Approach 1: Text Buffer Output**
- Generate unified diff format
- Display in new Zed buffer
- Use syntax highlighting for diff markers

```rust
pub fn format_unified_diff(changes: &[LineChange]) -> String {
    // Generate Git-style unified diff
    // --- file1.txt
    // +++ file2.txt
    // @@ -1,3 +1,4 @@
    //  unchanged line
    // -deleted line
    // +added line
}
```

**Approach 2: Split View (if API allows)**
- Create two side-by-side editor panes
- Synchronize scrolling
- Highlight changed regions

**Approach 3: Inline Annotations**
- Show diff in original file
- Use text decorations/markers for changes
- Clickable navigation between changes

#### 4.11 Color Coding

Define clear visual markers:
- **Green background**: Added lines
- **Red background**: Deleted lines
- **Yellow background**: Modified lines
- **Character-level**: Darker shades for specific changes

#### 4.12 Navigation Features

- Jump to next/previous difference
- Collapse unchanged sections
- Quick accept/reject changes (for merge scenarios)

---

### Phase 5: Testing & Refinement (Week 12-13)

#### 4.13 Test Suite

1. **Unit Tests**
   - Algorithm correctness
   - Edge cases
   - Performance benchmarks

2. **Integration Tests**
   - File reading
   - Command execution
   - UI rendering

3. **User Acceptance Testing**
   - Real-world file comparisons
   - Large file handling
   - Cross-platform compatibility (macOS, Linux, Windows)

#### 4.14 Performance Optimization

- Profile with large files (10k+ lines)
- Optimize hot paths
- Implement lazy loading for very large diffs
- Add progress indicators

#### 4.15 Documentation

- User guide with screenshots
- API documentation
- Contributing guidelines
- Known limitations

---

### Phase 6: Release & Iteration (Week 14+)

#### 4.16 Initial Release (v0.1.0)

**Minimum Viable Product Features:**
- ✅ Compare two files via command
- ✅ Display unified diff output
- ✅ Basic syntax highlighting
- ✅ Ignore whitespace option
- ✅ Works with text files up to 5000 lines

#### 4.17 Future Enhancements (v0.2.0+)

1. **Enhanced UI**
   - Side-by-side view (pending Zed API)
   - Inline diff annotations
   - Minimap overview

2. **Advanced Features**
   - Directory comparison
   - 3-way merge support
   - Conflict resolution tools
   - Integration with Git

3. **Performance**
   - Streaming diff for huge files
   - Incremental updates
   - Background processing

4. **Customization**
   - Configurable color schemes
   - Custom diff algorithms (word-level, semantic)
   - Filter rules (ignore patterns)

---

## 5. Technical Reference Materials

### 5.1 Key Repositories

1. **VSCode**: `https://github.com/microsoft/vscode`
   - Original implementation reference
   
2. **vscode-diff**: `https://github.com/micnil/vscode-diff`
   - Extracted, standalone diff algorithm
   - Zero dependencies
   - Good starting point for porting

3. **vs-diff**: `https://github.com/vscode-utility/vs-diff`
   - Alternative extraction with latest updates

4. **Zed Editor**: `https://github.com/zed-industries/zed`
   - Core editor codebase
   - Extension system implementation

### 5.2 Essential Documentation

- Zed Extension API: `https://docs.rs/zed_extension_api`
- Zed Extension Guide: `https://zed.dev/docs/extensions/developing-extensions`
- Myers Diff Paper: "An O(ND) Difference Algorithm and Its Variations" (1986)
- Git Diff Documentation: Understanding practical diff implementations

### 5.3 Community Resources

- Zed Issue #17100: Feature request for file comparison
- Zed Issue #4845: VSCode extension compatibility discussion
- VSCode Diff Algorithm Video: YouTube explanation of new algorithm

---

## 6. Risk Mitigation

### 6.1 Identified Risks

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Zed API insufficient for UI | High | Collaborate with Zed team for API extensions |
| Performance issues with large files | Medium | Implement timeout, optimization, streaming |
| Different platforms behave differently | Low | Thorough cross-platform testing |
| Algorithm complexity too high | Medium | Start with simple implementation, iterate |
| Extension adoption low | Low | Good documentation, demo videos |

### 6.2 Contingency Plans

- **Plan B for UI**: If Zed API too limited, contribute core diff to Zed instead of pure extension
- **Plan B for Algorithm**: Use external diff tool initially, replace with native later
- **Plan B for Performance**: Limit file size, add warnings, implement sampling

---

## 7. Success Metrics

### 7.1 Technical Metrics

- ✅ Successfully compare files up to 5000 lines in <1 second
- ✅ Correctly identify all differences (validated against Git diff)
- ✅ Zero crashes with malformed input
- ✅ Cross-platform compatibility (macOS, Linux, Windows)

### 7.2 User Metrics

- ✅ 100+ installations in first month
- ✅ <5% bug report rate
- ✅ Positive community feedback
- ✅ Active usage in real development workflows

---

## 8. Implementation Timeline Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| 1. Research & Setup | 2 weeks | Development environment, technical design |
| 2. Core Algorithm | 3 weeks | Working diff algorithm with tests |
| 3. Extension Development | 3 weeks | Zed extension with file handling |
| 4. User Interface | 3 weeks | Usable diff display |
| 5. Testing & Refinement | 2 weeks | Bug fixes, optimization, docs |
| 6. Release | 1 week | Published extension, user guide |
| **Total** | **14 weeks** | **Production-ready v0.1.0** |

---

## 9. Next Steps

### Immediate Actions (This Week)

1. **Set up development environment**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install Zed
   # Download from https://zed.dev
   
   # Create extension project
   cargo new --lib zed-diff-plugin
   cd zed-diff-plugin
   cargo add zed_extension_api
   ```

2. **Study reference implementations**
   - Clone `vscode-diff` repository
   - Run and analyze examples
   - Understand data structures

3. **Connect with Zed community**
   - Join Zed Discord/forum
   - Comment on GitHub Issue #17100
   - Propose collaboration approach

4. **Create proof-of-concept**
   - Implement basic Myers diff in Rust
   - Test with simple string arrays
   - Measure performance

---

## 10. Conclusion

Building a file comparison plugin for Zed is an ambitious but achievable project. The key challenges are:

1. **Algorithm Complexity**: Myers diff algorithm is well-documented and proven
2. **API Limitations**: May require collaboration with Zed team
3. **Performance**: Critical for large files, needs careful optimization

**Recommended Approach**: Start with **Option B (Hybrid)** - engage Zed team early, contribute core functionality to Zed itself, build rich extension on top. This maximizes long-term value and overcomes API limitations.

With the 14-week plan outlined above, you can deliver a production-ready diff plugin that brings VSCode-quality file comparison to Zed users.

---

## Appendix: Code Examples

### A.1 Myers Diff Algorithm (Simplified Rust)

```rust
pub fn myers_diff(original: &[String], modified: &[String]) -> Vec<Change> {
    let m = original.len();
    let n = modified.len();
    
    // Create DP table
    let mut dp = vec![vec![0; n + 1]; m + 1];
    
    // Initialize first row and column
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }
    
    // Fill DP table
    for i in 1..=m {
        for j in 1..=n {
            if original[i-1] == modified[j-1] {
                dp[i][j] = dp[i-1][j-1];
            } else {
                dp[i][j] = 1 + dp[i-1][j].min(dp[i][j-1]).min(dp[i-1][j-1]);
            }
        }
    }
    
    // Backtrack to find changes
    let mut changes = Vec::new();
    let mut i = m;
    let mut j = n;
    
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && original[i-1] == modified[j-1] {
            // No change
            i -= 1;
            j -= 1;
        } else if i > 0 && (j == 0 || dp[i][j] == dp[i-1][j] + 1) {
            // Deletion
            changes.push(Change::Delete { line: i-1 });
            i -= 1;
        } else {
            // Insertion
            changes.push(Change::Insert { line: j-1 });
            j -= 1;
        }
    }
    
    changes.reverse();
    changes
}

pub enum Change {
    Delete { line: usize },
    Insert { line: usize },
}
```

### A.2 Zed Extension Skeleton

```rust
use zed_extension_api as zed;
use std::fs;

struct DiffExtension;

impl zed::Extension for DiffExtension {
    fn new() -> Self {
        Self
    }
    
    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // Extension initialization
        Ok(zed::Command {
            command: "diff".into(),
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(DiffExtension);
```

---

**Document Version**: 1.0  
**Last Updated**: February 8, 2026  
**Author**: Technical Planning Assistant  
**Status**: Draft for Review
