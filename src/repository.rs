use std::path::Path;

use crate::command::GitCommand;
use crate::error::{GitError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryState {
    Clean,
    Merge,
    Rebase,
    RebaseInteractive,
    CherryPick,
    CherryPickSequence,
    Revert,
    RevertSequence,
    Bisect,
    ApplyMailbox,
    ApplyMailboxOrRebase,
    Other(String),
}

impl From<&str> for RepositoryState {
    fn from(s: &str) -> Self {
        match s {
            "clean" => RepositoryState::Clean,
            "merge" => RepositoryState::Merge,
            "rebase" => RepositoryState::Rebase,
            "rebase-i" => RepositoryState::RebaseInteractive,
            "rebase-m" => RepositoryState::RebaseInteractive,
            "cherry-pick" => RepositoryState::CherryPick,
            "cherry-pick-sequence" => RepositoryState::CherryPickSequence,
            "revert" => RepositoryState::Revert,
            "revert-sequence" => RepositoryState::RevertSequence,
            "bisect" => RepositoryState::Bisect,
            "am" => RepositoryState::ApplyMailbox,
            "am/rebase" => RepositoryState::ApplyMailboxOrRebase,
            _ => RepositoryState::Other(s.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Repository {
    pub(crate) cmd: GitCommand,
    path: String,
}

impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        // Verify it's a git repository
        let cmd = GitCommand::new().with_working_dir(&path_str);
        cmd.exec(["rev-parse", "--git-dir"])?;
        
        Ok(Self {
            cmd,
            path: path_str,
        })
    }
    
    pub fn init<P: AsRef<Path>>(path: P, bare: bool) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        let mut args = vec!["init"];
        if bare {
            args.push("--bare");
        }
        args.push(&path_str);
        
        GitCommand::new().exec(args)?;
        
        Self::open(&path_str)
    }
    
    pub fn clone<S: AsRef<str>, P: AsRef<Path>>(
        url: S,
        path: P,
        options: Option<CloneOptions>,
    ) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        let mut args = vec!["clone".to_string()];
        
        if let Some(opts) = options {
            if opts.bare {
                args.push("--bare".to_string());
            }
            if opts.mirror {
                args.push("--mirror".to_string());
            }
            if opts.single_branch {
                args.push("--single-branch".to_string());
            }
            if let Some(ref branch) = opts.branch {
                args.push("--branch".to_string());
                args.push(branch.clone());
            }
            if let Some(ref depth) = opts.depth {
                args.push("--depth".to_string());
                args.push(depth.to_string());
            }
            if opts.recursive {
                args.push("--recursive".to_string());
            }
        }
        
        args.push(url.as_ref().to_string());
        args.push(path_str.clone());
        
        GitCommand::new().exec(args)?;
        
        Self::open(&path_str)
    }
    
    pub fn path(&self) -> &str {
        &self.path
    }
    
    pub fn git_command(&self) -> &GitCommand {
        &self.cmd
    }
    
    pub fn git_dir(&self) -> Result<String> {
        self.cmd.exec_text(["rev-parse", "--git-dir"])
    }
    
    pub fn work_dir(&self) -> Result<Option<String>> {
        let result = self.cmd.exec_text(["rev-parse", "--show-toplevel"]);
        match result {
            Ok(path) => Ok(Some(path)),
            Err(GitError::NotARepository) => Ok(None),
            Err(e) => Err(e),
        }
    }
    
    pub fn is_bare(&self) -> Result<bool> {
        let output = self.cmd.exec_text(["rev-parse", "--is-bare-repository"])?;
        Ok(output == "true")
    }
    
    pub fn is_shallow(&self) -> Result<bool> {
        let output = self.cmd.exec_text(["rev-parse", "--is-shallow-repository"])?;
        Ok(output == "true")
    }
    
    pub fn state(&self) -> Result<RepositoryState> {
        let output = self.cmd.exec_text(["status", "--porcelain=v2", "--branch"])
            .or_else(|_| self.cmd.exec_text(["status", "--porcelain"]));
        
        // Check for operation in progress by looking at .git directory
        let git_dir = self.git_dir()?;
        
        let rebase_dir = format!("{}/rebase-apply", git_dir);
        let rebase_merge_dir = format!("{}/rebase-merge", git_dir);
        let merge_dir = format!("{}/MERGE_HEAD", git_dir);
        let cherry_pick_dir = format!("{}/CHERRY_PICK_HEAD", git_dir);
        let revert_dir = format!("{}/REVERT_HEAD", git_dir);
        let bisect_log = format!("{}/BISECT_LOG", git_dir);
        let am_dir = format!("{}/rebase-apply/applying", git_dir);
        
        if std::path::Path::new(&rebase_merge_dir).exists() {
            if std::path::Path::new(&format!("{}/interactive", rebase_merge_dir)).exists() {
                return Ok(RepositoryState::RebaseInteractive);
            }
            return Ok(RepositoryState::Rebase);
        }
        
        if std::path::Path::new(&rebase_dir).exists() {
            return Ok(RepositoryState::Rebase);
        }
        
        if std::path::Path::new(&merge_dir).exists() {
            return Ok(RepositoryState::Merge);
        }
        
        if std::path::Path::new(&cherry_pick_dir).exists() {
            return Ok(RepositoryState::CherryPick);
        }
        
        if std::path::Path::new(&revert_dir).exists() {
            return Ok(RepositoryState::Revert);
        }
        
        if std::path::Path::new(&bisect_log).exists() {
            return Ok(RepositoryState::Bisect);
        }
        
        if std::path::Path::new(&am_dir).exists() {
            return Ok(RepositoryState::ApplyMailbox);
        }
        
        Ok(RepositoryState::Clean)
    }
    
    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        match self.cmd.exec_text(["config", key]) {
            Ok(value) => Ok(Some(value)),
            Err(_) => Ok(None),
        }
    }
    
    pub fn set_config(&self, key: &str, value: &str, global: bool) -> Result<()> {
        let mut args = vec!["config"];
        if global {
            args.push("--global");
        }
        args.push(key);
        args.push(value);
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn unset_config(&self, key: &str, global: bool) -> Result<()> {
        let mut args = vec!["config", "--unset"];
        if global {
            args.push("--global");
        }
        args.push(key);
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn list_config(&self) -> Result<Vec<(String, String)>> {
        let lines = self.cmd.exec_lines(["config", "--list"])?;
        let mut result = Vec::new();
        for line in lines {
            if let Some(pos) = line.find('=') {
                let key = line[..pos].to_string();
                let value = line[pos + 1..].to_string();
                result.push((key, value));
            }
        }
        Ok(result)
    }
}

#[derive(Debug, Default, Clone)]
pub struct CloneOptions {
    pub bare: bool,
    pub mirror: bool,
    pub single_branch: bool,
    pub branch: Option<String>,
    pub depth: Option<usize>,
    pub recursive: bool,
}

impl CloneOptions {
    pub fn bare(mut self) -> Self {
        self.bare = true;
        self
    }
    
    pub fn mirror(mut self) -> Self {
        self.mirror = true;
        self
    }
    
    pub fn single_branch(mut self) -> Self {
        self.single_branch = true;
        self
    }
    
    pub fn branch<S: AsRef<str>>(mut self, branch: S) -> Self {
        self.branch = Some(branch.as_ref().to_string());
        self
    }
    
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }
    
    pub fn recursive(mut self) -> Self {
        self.recursive = true;
        self
    }
}
