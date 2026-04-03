#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FileStatus {
    Unmodified,
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    UpdatedButUnmerged,
    Untracked,
    Ignored,
}

impl From<char> for FileStatus {
    fn from(c: char) -> Self {
        match c {
            '.' | ' ' => FileStatus::Unmodified,
            'M' => FileStatus::Modified,
            'A' => FileStatus::Added,
            'D' => FileStatus::Deleted,
            'R' => FileStatus::Renamed,
            'C' => FileStatus::Copied,
            'U' => FileStatus::UpdatedButUnmerged,
            '?' => FileStatus::Untracked,
            '!' => FileStatus::Ignored,
            _ => FileStatus::Unmodified,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StatusEntry {
    pub index_status: FileStatus,
    pub worktree_status: FileStatus,
    pub path: String,
    pub original_path: Option<String>,
    pub submodule_status: Option<String>,
    pub file_mode_head: Option<String>,
    pub file_mode_index: Option<String>,
    pub file_mode_worktree: Option<String>,
    pub object_name_head: Option<String>,
    pub object_name_index: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BranchInfo {
    pub local_branch: Option<String>,
    pub upstream_branch: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RepositoryStatus {
    pub entries: Vec<StatusEntry>,
    pub branch_info: Option<BranchInfo>,
    pub is_clean: bool,
}

#[derive(Debug, Clone)]
pub struct StatusOptions {
    pub porcelain: bool,
    pub short: bool,
    pub branch: bool,
    pub show_submodules: bool,
    pub ignored: bool,
    pub untracked: UntrackedMode,
}

impl Default for StatusOptions {
    fn default() -> Self {
        Self {
            porcelain: true,
            short: false,
            branch: true,
            show_submodules: false,
            ignored: false,
            untracked: UntrackedMode::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UntrackedMode {
    No,
    Normal,
    All,
}

impl StatusOptions {
    pub fn porcelain(mut self) -> Self {
        self.porcelain = true;
        self
    }
    
    pub fn short(mut self) -> Self {
        self.short = true;
        self
    }
    
    pub fn with_branch(mut self) -> Self {
        self.branch = true;
        self
    }
    
    pub fn with_submodules(mut self) -> Self {
        self.show_submodules = true;
        self
    }
    
    pub fn with_ignored(mut self) -> Self {
        self.ignored = true;
        self
    }
    
    pub fn untracked(mut self, mode: UntrackedMode) -> Self {
        self.untracked = mode;
        self
    }
}

use crate::repository::Repository;
use crate::error::Result;

impl Repository {
    pub fn status(&self, options: Option<StatusOptions>) -> Result<RepositoryStatus> {
        let opts = options.unwrap_or_default();
        
        let mut args = vec!["status"];
        
        if opts.porcelain {
            args.push("--porcelain=v2");
        } else if opts.short {
            args.push("-s");
        }
        
        if opts.branch {
            args.push("-b");
        }
        
        if opts.show_submodules {
            args.push("--show-stash");
        }
        
        if opts.ignored {
            args.push("--ignored");
        }
        
        match opts.untracked {
            UntrackedMode::No => args.push("-uno"),
            UntrackedMode::Normal => {},
            UntrackedMode::All => args.push("-uall"),
        }
        
        let output = self.cmd.exec_text(args)?;
        self.parse_status_output(&output)
    }
    
    pub fn status_simple(&self) -> Result<Vec<StatusEntry>> {
        let output = self.cmd.exec_text(["status", "-s"])?;
        
        let mut entries = Vec::new();
        for line in output.lines() {
            if line.len() >= 3 {
                let index_status = FileStatus::from(line.chars().next().unwrap_or(' '));
                let worktree_status = FileStatus::from(line.chars().nth(1).unwrap_or(' '));
                let path = line[3..].to_string();
                
                entries.push(StatusEntry {
                    index_status,
                    worktree_status,
                    path,
                    original_path: None,
                    submodule_status: None,
                    file_mode_head: None,
                    file_mode_index: None,
                    file_mode_worktree: None,
                    object_name_head: None,
                    object_name_index: None,
                });
            }
        }
        
        Ok(entries)
    }
    
    pub fn is_clean(&self) -> Result<bool> {
        match self.status_simple() {
            Ok(entries) => Ok(entries.is_empty()),
            Err(_) => Ok(false),
        }
    }
    
    fn parse_status_output(&self, output: &str) -> Result<RepositoryStatus> {
        let mut entries = Vec::new();
        let mut branch_info: Option<BranchInfo> = None;
        
        for line in output.lines() {
            if line.starts_with("#") || line.starts_with("?") {
                // Porcelain v2 format
                if line.starts_with("# branch.head") {
                    let branch = line.split_whitespace().nth(2).map(|s| s.to_string());
                    branch_info = Some(BranchInfo {
                        local_branch: branch,
                        upstream_branch: None,
                        ahead: 0,
                        behind: 0,
                    });
                } else if line.starts_with("# branch.upstream") {
                    let upstream = line.split_whitespace().nth(2).map(|s| s.to_string());
                    if let Some(ref mut bi) = branch_info {
                        bi.upstream_branch = upstream;
                    }
                } else if line.starts_with("# branch.ab") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let ahead: usize = parts[2].parse().unwrap_or(0);
                        let behind: usize = parts[3].parse().unwrap_or(0);
                        if let Some(ref mut bi) = branch_info {
                            bi.ahead = ahead.abs_diff(0);
                            bi.behind = behind.abs_diff(0);
                        }
                    }
                }
            } else if line.starts_with("1 ") || line.starts_with("2 ") || line.starts_with("u ") {
                // Ordinary change, rename/copy, or unmerged entry
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 9 {
                    let status_codes = parts[1];
                    let index_status = FileStatus::from(status_codes.chars().next().unwrap_or('.'));
                    let worktree_status = FileStatus::from(status_codes.chars().nth(1).unwrap_or('.'));
                    
                    entries.push(StatusEntry {
                        index_status,
                        worktree_status,
                        path: parts[8].to_string(),
                        original_path: None,
                        submodule_status: Some(parts[2].to_string()),
                        file_mode_head: Some(parts[3].to_string()),
                        file_mode_index: Some(parts[4].to_string()),
                        file_mode_worktree: Some(parts[5].to_string()),
                        object_name_head: Some(parts[6].to_string()),
                        object_name_index: Some(parts[7].to_string()),
                    });
                }
            } else if line.starts_with("? ") {
                // Untracked file
                let path = line[2..].to_string();
                entries.push(StatusEntry {
                    index_status: FileStatus::Untracked,
                    worktree_status: FileStatus::Untracked,
                    path,
                    original_path: None,
                    submodule_status: None,
                    file_mode_head: None,
                    file_mode_index: None,
                    file_mode_worktree: None,
                    object_name_head: None,
                    object_name_index: None,
                });
            }
        }
        
        let is_clean = entries.is_empty();
        
        Ok(RepositoryStatus {
            entries,
            branch_info,
            is_clean,
        })
    }
}
