use crate::repository::Repository;
use crate::error::{GitError, Result};

#[derive(Debug, Clone)]
pub struct RebaseState {
    pub onto: String,
    pub head_name: String,
    pub orig_head: String,
}

#[derive(Debug, Default, Clone)]
pub struct RebaseOptions {
    pub interactive: bool,
    pub rebase_merges: bool,
    pub onto: Option<String>,
    pub strategy: Option<String>,
    pub autosquash: bool,
    pub autostash: bool,
    pub committer_date_is_author_date: bool,
    pub ignore_date: bool,
    pub signoff: bool,
    pub exec: Vec<String>,
}

impl RebaseOptions {
    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }
    
    pub fn rebase_merges(mut self) -> Self {
        self.rebase_merges = true;
        self
    }
    
    pub fn onto<S: AsRef<str>>(mut self, onto: S) -> Self {
        self.onto = Some(onto.as_ref().to_string());
        self
    }
    
    pub fn strategy<S: AsRef<str>>(mut self, strategy: S) -> Self {
        self.strategy = Some(strategy.as_ref().to_string());
        self
    }
    
    pub fn autosquash(mut self) -> Self {
        self.autosquash = true;
        self
    }
    
    pub fn autostash(mut self) -> Self {
        self.autostash = true;
        self
    }
    
    pub fn signoff(mut self) -> Self {
        self.signoff = true;
        self
    }
    
    pub fn exec<S: AsRef<str>>(mut self, command: S) -> Self {
        self.exec.push(command.as_ref().to_string());
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct CherryPickOptions {
    pub edit: bool,
    pub no_commit: bool,
    pub signoff: bool,
    pub mainline: Option<usize>,
    pub strategy: Option<String>,
    pub strategy_option: Vec<String>,
}

impl CherryPickOptions {
    pub fn edit(mut self) -> Self {
        self.edit = true;
        self
    }
    
    pub fn no_commit(mut self) -> Self {
        self.no_commit = true;
        self
    }
    
    pub fn signoff(mut self) -> Self {
        self.signoff = true;
        self
    }
    
    pub fn mainline(mut self, parent: usize) -> Self {
        self.mainline = Some(parent);
        self
    }
    
    pub fn strategy<S: AsRef<str>>(mut self, strategy: S) -> Self {
        self.strategy = Some(strategy.as_ref().to_string());
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct RevertOptions {
    pub edit: bool,
    pub no_commit: bool,
    pub signoff: bool,
    pub mainline: Option<usize>,
}

impl RevertOptions {
    pub fn edit(mut self) -> Self {
        self.edit = true;
        self
    }
    
    pub fn no_commit(mut self) -> Self {
        self.no_commit = true;
        self
    }
    
    pub fn signoff(mut self) -> Self {
        self.signoff = true;
        self
    }
    
    pub fn mainline(mut self, parent: usize) -> Self {
        self.mainline = Some(parent);
        self
    }
}

impl Repository {
    // Rebase operations
    pub fn rebase<U: AsRef<str>, B: AsRef<str>>(&self, upstream: U, branch: Option<B>, options: Option<RebaseOptions>) -> Result<()> {
        let mut args: Vec<String> = vec!["rebase".to_string()];
        let opts = options.unwrap_or_default();
        
        if opts.interactive {
            args.push("-i".to_string());
        }
        
        if opts.rebase_merges {
            args.push("--rebase-merges".to_string());
        }
        
        if let Some(ref onto) = opts.onto {
            args.push("--onto".to_string());
            args.push(onto.clone());
        }
        
        if let Some(ref strategy) = opts.strategy {
            args.push("--strategy".to_string());
            args.push(strategy.clone());
        }
        
        if opts.autosquash {
            args.push("--autosquash".to_string());
        }
        
        if opts.autostash {
            args.push("--autostash".to_string());
        }
        
        if opts.signoff {
            args.push("--signoff".to_string());
        }
        
        for cmd in &opts.exec {
            args.push("--exec".to_string());
            args.push(cmd.clone());
        }
        
        args.push(upstream.as_ref().to_string());
        
        if let Some(b) = branch {
            args.push(b.as_ref().to_string());
        }
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn rebase_continue(&self) -> Result<()> {
        self.cmd.exec(["rebase", "--continue"])?;
        Ok(())
    }
    
    pub fn rebase_abort(&self) -> Result<()> {
        self.cmd.exec(["rebase", "--abort"])?;
        Ok(())
    }
    
    pub fn rebase_skip(&self) -> Result<()> {
        self.cmd.exec(["rebase", "--skip"])?;
        Ok(())
    }
    
    pub fn rebase_edit_todo(&self) -> Result<()> {
        self.cmd.exec(["rebase", "--edit-todo"])?;
        Ok(())
    }
    
    pub fn rebase_state(&self) -> Result<Option<RebaseState>> {
        let git_dir = self.git_dir()?;
        let rebase_merge_dir = format!("{}/rebase-merge", git_dir);
        let rebase_apply_dir = format!("{}/rebase-apply", git_dir);
        
        let active_dir = if std::path::Path::new(&rebase_merge_dir).exists() {
            rebase_merge_dir
        } else if std::path::Path::new(&rebase_apply_dir).exists() {
            rebase_apply_dir
        } else {
            return Ok(None);
        };
        
        let onto = std::fs::read_to_string(format!("{}/onto", active_dir))
            .unwrap_or_default()
            .trim()
            .to_string();
        
        let head_name = std::fs::read_to_string(format!("{}/head-name", active_dir))
            .unwrap_or_default()
            .trim()
            .to_string();
        
        let orig_head = std::fs::read_to_string(format!("{}/orig-head", active_dir))
            .unwrap_or_default()
            .trim()
            .to_string();
        
        Ok(Some(RebaseState {
            onto,
            head_name,
            orig_head,
        }))
    }
    
    // Cherry-pick operations
    pub fn cherry_pick<S: AsRef<str>>(&self, commits: &[S], options: Option<CherryPickOptions>) -> Result<()> {
        let mut args: Vec<String> = vec!["cherry-pick".to_string()];
        let opts = options.unwrap_or_default();
        
        if opts.edit {
            args.push("-e".to_string());
        }
        
        if opts.no_commit {
            args.push("-n".to_string());
        }
        
        if opts.signoff {
            args.push("-s".to_string());
        }
        
        if let Some(mainline) = opts.mainline {
            args.push("-m".to_string());
            args.push(mainline.to_string());
        }
        
        if let Some(ref strategy) = opts.strategy {
            args.push("--strategy".to_string());
            args.push(strategy.clone());
        }
        
        for commit in commits {
            args.push(commit.as_ref().to_string());
        }
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn cherry_pick_continue(&self) -> Result<()> {
        self.cmd.exec(["cherry-pick", "--continue"])?;
        Ok(())
    }
    
    pub fn cherry_pick_abort(&self) -> Result<()> {
        self.cmd.exec(["cherry-pick", "--abort"])?;
        Ok(())
    }
    
    pub fn cherry_pick_skip(&self) -> Result<()> {
        self.cmd.exec(["cherry-pick", "--skip"])?;
        Ok(())
    }
    
    // Revert operations
    pub fn revert<S: AsRef<str>>(&self, commits: &[S], options: Option<RevertOptions>) -> Result<()> {
        let mut args: Vec<String> = vec!["revert".to_string()];
        let opts = options.unwrap_or_default();
        
        if !opts.edit {
            args.push("--no-edit".to_string());
        }
        
        if opts.no_commit {
            args.push("-n".to_string());
        }
        
        if opts.signoff {
            args.push("-s".to_string());
        }
        
        if let Some(mainline) = opts.mainline {
            args.push("-m".to_string());
            args.push(mainline.to_string());
        }
        
        for commit in commits {
            args.push(commit.as_ref().to_string());
        }
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn revert_continue(&self) -> Result<()> {
        self.cmd.exec(["revert", "--continue"])?;
        Ok(())
    }
    
    pub fn revert_abort(&self) -> Result<()> {
        self.cmd.exec(["revert", "--abort"])?;
        Ok(())
    }
    
    pub fn revert_skip(&self) -> Result<()> {
        self.cmd.exec(["revert", "--skip"])?;
        Ok(())
    }
    
    // Bisect operations
    pub fn bisect_start(&self) -> Result<()> {
        self.cmd.exec(["bisect", "start"])?;
        Ok(())
    }
    
    pub fn bisect_bad(&self, commit: Option<&str>) -> Result<()> {
        let mut args: Vec<String> = vec!["bisect".to_string(), "bad".to_string()];
        if let Some(c) = commit {
            args.push(c.to_string());
        }
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn bisect_good<S: AsRef<str>>(&self, commits: &[S]) -> Result<()> {
        let mut args: Vec<String> = vec!["bisect".to_string(), "good".to_string()];
        for commit in commits {
            args.push(commit.as_ref().to_string());
        }
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn bisect_skip<S: AsRef<str>>(&self, commits: &[S]) -> Result<()> {
        let mut args: Vec<String> = vec!["bisect".to_string(), "skip".to_string()];
        for commit in commits {
            args.push(commit.as_ref().to_string());
        }
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn bisect_reset(&self) -> Result<()> {
        self.cmd.exec(["bisect", "reset"])?;
        Ok(())
    }
    
    pub fn bisect_log(&self) -> Result<String> {
        self.cmd.exec_text(["bisect", "log"])
    }
    
    pub fn bisect_visualize(&self) -> Result<String> {
        self.cmd.exec_text(["bisect", "visualize"])
    }
    
    pub fn bisect_replay<S: AsRef<str>>(&self, logfile: S) -> Result<()> {
        self.cmd.exec(["bisect", "replay", logfile.as_ref()])?;
        Ok(())
    }
    
    pub fn bisect_run<S: AsRef<str>>(&self, command: S, args: &[S]) -> Result<String> {
        let mut cmd_args: Vec<&str> = vec!["bisect", "run", command.as_ref()];
        for arg in args {
            cmd_args.push(arg.as_ref());
        }
        self.cmd.exec_text(cmd_args)
    }
    
    // Worktree operations
    pub fn worktree_list(&self) -> Result<Vec<WorktreeEntry>> {
        let output = self.cmd.exec_text(["worktree", "list", "--porcelain"])?;
        let mut entries = Vec::new();
        let mut current = None;
        
        for line in output.lines() {
            if line.starts_with("worktree ") {
                if let Some(entry) = current.take() {
                    entries.push(entry);
                }
                let path = line[9..].to_string();
                current = Some(WorktreeEntry {
                    path,
                    head: String::new(),
                    branch: None,
                    detached: false,
                    bare: false,
                });
            } else if line.starts_with("HEAD ") {
                if let Some(ref mut e) = current {
                    e.head = line[5..].to_string();
                }
            } else if line.starts_with("branch ") {
                if let Some(ref mut e) = current {
                    e.branch = Some(line[7..].to_string());
                }
            } else if line == "detached" {
                if let Some(ref mut e) = current {
                    e.detached = true;
                }
            } else if line == "bare" {
                if let Some(ref mut e) = current {
                    e.bare = true;
                }
            }
        }
        
        if let Some(entry) = current {
            entries.push(entry);
        }
        
        Ok(entries)
    }
    
    pub fn worktree_add<P: AsRef<std::path::Path>, S: AsRef<str>>(&self, path: P, commit_ish: S, force: bool) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        let mut args: Vec<String> = vec!["worktree".to_string(), "add".to_string()];
        
        if force {
            args.push("-f".to_string());
        }
        
        args.push(path_str);
        args.push(commit_ish.as_ref().to_string());
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn worktree_remove<P: AsRef<std::path::Path>>(&self, path: P, force: bool) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        let mut args: Vec<String> = vec!["worktree".to_string(), "remove".to_string()];
        
        if force {
            args.push("-f".to_string());
        }
        
        args.push(path_str);
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn worktree_prune(&self, dry_run: bool, expire: Option<&str>) -> Result<()> {
        let mut args: Vec<String> = vec!["worktree".to_string(), "prune".to_string()];
        
        if dry_run {
            args.push("-n".to_string());
        }
        
        if let Some(exp) = expire {
            args.push("--expire".to_string());
            args.push(exp.to_string());
        }
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn worktree_lock<P: AsRef<std::path::Path>, S: AsRef<str>>(&self, path: P, reason: Option<S>) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        let mut args: Vec<String> = vec!["worktree".to_string(), "lock".to_string()];
        
        if let Some(r) = reason {
            args.push("--reason".to_string());
            args.push(r.as_ref().to_string());
        }
        
        args.push(path_str);
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn worktree_unlock<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.cmd.exec(["worktree", "unlock", &path_str])?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WorktreeEntry {
    pub path: String,
    pub head: String,
    pub branch: Option<String>,
    pub detached: bool,
    pub bare: bool,
}
