use crate::repository::Repository;
use crate::error::{GitError, Result};

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub is_current: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub commit_hash: String,
    pub commit_message: String,
}

#[derive(Debug, Clone)]
pub struct BranchOptions {
    pub force: bool,
    pub track: bool,
    pub no_track: bool,
    pub set_upstream: bool,
    pub upstream: Option<String>,
}

impl Default for BranchOptions {
    fn default() -> Self {
        Self {
            force: false,
            track: false,
            no_track: false,
            set_upstream: false,
            upstream: None,
        }
    }
}

impl BranchOptions {
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }
    
    pub fn track(mut self) -> Self {
        self.track = true;
        self
    }
    
    pub fn no_track(mut self) -> Self {
        self.no_track = true;
        self
    }
    
    pub fn set_upstream<S: AsRef<str>>(mut self, upstream: S) -> Self {
        self.set_upstream = true;
        self.upstream = Some(upstream.as_ref().to_string());
        self
    }
}

impl Repository {
    pub fn branches(&self, all: bool, remote: bool) -> Result<Vec<Branch>> {
        let mut args = vec!["branch", "-vv"];
        
        if all {
            args.push("-a");
        }
        
        if remote {
            args.push("-r");
        }
        
        let output = self.cmd.exec_text(args)?;
        let mut branches = Vec::new();
        
        for line in output.lines() {
            if line.len() < 2 {
                continue;
            }
            
            let is_current = line.starts_with('*');
            let line = if is_current { &line[2..] } else { &line[2..] };
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            
            let name = parts[0].to_string();
            let commit_hash = parts.get(1).unwrap_or(&"").to_string();
            
            // Parse commit message (everything after hash)
            let commit_message = if parts.len() > 2 {
                parts[2..].join(" ")
            } else {
                String::new()
            };
            
            // Parse upstream info if present [upstream: ahead X, behind Y]
            let mut upstream = None;
            let mut ahead = 0;
            let mut behind = 0;
            
            if let Some(start) = line.find('[') {
                if let Some(end) = line.find(']') {
                    let info = &line[start + 1..end];
                    upstream = info.split(':').next().map(|s| s.to_string());
                    
                    // Parse ahead/behind
                    for part in info.split(',') {
                        if part.contains("ahead") {
                            ahead = part.split_whitespace()
                                .last()
                                .and_then(|n| n.parse().ok())
                                .unwrap_or(0);
                        }
                        if part.contains("behind") {
                            behind = part.split_whitespace()
                                .last()
                                .and_then(|n| n.parse().ok())
                                .unwrap_or(0);
                        }
                    }
                }
            }
            
            branches.push(Branch {
                name,
                is_current,
                upstream,
                ahead,
                behind,
                commit_hash,
                commit_message,
            });
        }
        
        Ok(branches)
    }
    
    pub fn current_branch(&self) -> Result<Option<String>> {
        match self.cmd.exec_text(["branch", "--show-current"]) {
            Ok(name) if !name.is_empty() => Ok(Some(name)),
            _ => Ok(None),
        }
    }
    
    pub fn branch_create<N: AsRef<str>, S: AsRef<str>>(&self, name: N, start_point: Option<S>, options: Option<BranchOptions>) -> Result<()> {
        let mut args: Vec<String> = vec!["branch".to_string()];
        let opts = options.unwrap_or_default();
        
        if opts.force {
            args.push("-f".to_string());
        }
        
        if opts.track {
            args.push("--track".to_string());
        }
        
        if opts.no_track {
            args.push("--no-track".to_string());
        }
        
        if opts.set_upstream {
            if let Some(upstream) = opts.upstream {
                args.push("--set-upstream-to".to_string());
                args.push(upstream);
            }
        }
        
        args.push(name.as_ref().to_string());
        
        if let Some(start) = start_point {
            args.push(start.as_ref().to_string());
        }
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn branch_delete<S: AsRef<str>>(&self, name: S, force: bool) -> Result<()> {
        let mut args: Vec<String> = vec!["branch".to_string()];
        
        if force {
            args.push("-D".to_string());
        } else {
            args.push("-d".to_string());
        }
        
        args.push(name.as_ref().to_string());
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn branch_rename<O: AsRef<str>, N: AsRef<str>>(&self, old_name: O, new_name: N, force: bool) -> Result<()> {
        let mut args: Vec<String> = vec!["branch".to_string(), "-m".to_string()];
        
        if force {
            args.push("-f".to_string());
        }
        
        args.push(old_name.as_ref().to_string());
        args.push(new_name.as_ref().to_string());
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn branch_set_upstream<B: AsRef<str>, U: AsRef<str>>(&self, branch: B, upstream: U) -> Result<()> {
        self.cmd.exec([
            "branch",
            "--set-upstream-to",
            upstream.as_ref(),
            branch.as_ref(),
        ])?;
        Ok(())
    }
    
    pub fn branch_unset_upstream<S: AsRef<str>>(&self, branch: S) -> Result<()> {
        self.cmd.exec(["branch", "--unset-upstream", branch.as_ref()])?;
        Ok(())
    }
    
    pub fn checkout<S: AsRef<str>>(&self, branch: S, create: bool, force: bool) -> Result<()> {
        let mut args: Vec<String> = vec!["checkout".to_string()];
        
        if create {
            args.push("-b".to_string());
        }
        
        if force {
            args.push("-f".to_string());
        }
        
        args.push(branch.as_ref().to_string());
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn checkout_detach<S: AsRef<str>>(&self, commit: S) -> Result<()> {
        self.cmd.exec(["checkout", "--detach", commit.as_ref()])?;
        Ok(())
    }
    
    pub fn checkout_paths<P: AsRef<std::path::Path>, S: AsRef<str>>(&self, paths: &[P], source: Option<S>) -> Result<()> {
        let mut args: Vec<String> = vec!["checkout".to_string()];
        
        if let Some(src) = source {
            args.push(src.as_ref().to_string());
        }
        
        for path in paths {
            args.push(path.as_ref().to_string_lossy().to_string());
        }
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn switch<S: AsRef<str>>(&self, branch: S, create: bool, force: bool, detach: bool) -> Result<()> {
        let mut args: Vec<String> = vec!["switch".to_string()];
        
        if create {
            args.push("-c".to_string());
        } else if detach {
            args.push("--detach".to_string());
        }
        
        if force {
            args.push("-f".to_string());
        }
        
        args.push(branch.as_ref().to_string());
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn merge<B: AsRef<str>>(&self, branch: B, no_ff: bool, ff_only: bool, squash: bool, no_commit: bool) -> Result<()> {
        let mut args: Vec<String> = vec!["merge".to_string()];
        
        if no_ff {
            args.push("--no-ff".to_string());
        }
        
        if ff_only {
            args.push("--ff-only".to_string());
        }
        
        if squash {
            args.push("--squash".to_string());
        }
        
        if no_commit {
            args.push("--no-commit".to_string());
        }
        
        args.push(branch.as_ref().to_string());
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn merge_abort(&self) -> Result<()> {
        self.cmd.exec(["merge", "--abort"])?;
        Ok(())
    }
    
    pub fn merge_continue(&self) -> Result<()> {
        self.cmd.exec(["merge", "--continue"])?;
        Ok(())
    }
    
    pub fn merge_base<S: AsRef<str>>(&self, commits: &[S]) -> Result<String> {
        let mut args: Vec<String> = vec!["merge-base".to_string()];
        for commit in commits {
            args.push(commit.as_ref().to_string());
        }
        self.cmd.exec_text(args)
    }
    
    pub fn is_ancestor<A: AsRef<str>, D: AsRef<str>>(&self, ancestor: A, descendant: D) -> Result<bool> {
        let result = self.cmd.exec([
            "merge-base",
            "--is-ancestor",
            ancestor.as_ref(),
            descendant.as_ref(),
        ]);
        
        match result {
            Ok(_) => Ok(true),
            Err(GitError::Command(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}
