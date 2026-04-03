use crate::repository::Repository;
use crate::error::{GitError, Result};

#[derive(Debug, Clone)]
pub struct Remote {
    pub name: String,
    pub url: Option<String>,
    pub push_url: Option<String>,
    pub fetch_refspecs: Vec<String>,
    pub push_refspecs: Vec<String>,
    pub branches: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Default, Clone)]
pub struct PushOptions {
    pub force: bool,
    pub force_with_lease: bool,
    pub all: bool,
    pub tags: bool,
    pub delete: bool,
    pub set_upstream: bool,
    pub dry_run: bool,
}

impl PushOptions {
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }
    
    pub fn force_with_lease(mut self) -> Self {
        self.force_with_lease = true;
        self
    }
    
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }
    
    pub fn tags(mut self) -> Self {
        self.tags = true;
        self
    }
    
    pub fn delete(mut self) -> Self {
        self.delete = true;
        self
    }
    
    pub fn set_upstream(mut self) -> Self {
        self.set_upstream = true;
        self
    }
    
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct FetchOptions {
    pub all: bool,
    pub prune: bool,
    pub tags: bool,
    pub no_tags: bool,
    pub depth: Option<usize>,
    pub force: bool,
}

impl FetchOptions {
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }
    
    pub fn prune(mut self) -> Self {
        self.prune = true;
        self
    }
    
    pub fn tags(mut self) -> Self {
        self.tags = true;
        self
    }
    
    pub fn no_tags(mut self) -> Self {
        self.no_tags = true;
        self
    }
    
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }
    
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct PullOptions {
    pub rebase: bool,
    pub no_rebase: bool,
    pub ff_only: bool,
    pub no_ff: bool,
    pub squash: bool,
    pub tags: bool,
    pub prune: bool,
    pub depth: Option<usize>,
}

impl PullOptions {
    pub fn rebase(mut self) -> Self {
        self.rebase = true;
        self
    }
    
    pub fn no_rebase(mut self) -> Self {
        self.no_rebase = true;
        self
    }
    
    pub fn ff_only(mut self) -> Self {
        self.ff_only = true;
        self
    }
    
    pub fn no_ff(mut self) -> Self {
        self.no_ff = true;
        self
    }
    
    pub fn squash(mut self) -> Self {
        self.squash = true;
        self
    }
    
    pub fn tags(mut self) -> Self {
        self.tags = true;
        self
    }
    
    pub fn prune(mut self) -> Self {
        self.prune = true;
        self
    }
    
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }
}

impl Repository {
    pub fn remotes(&self) -> Result<Vec<RemoteInfo>> {
        let output = self.cmd.exec_text(["remote", "-v"])?;
        let mut remotes = Vec::new();
        let mut seen = std::collections::HashSet::new();
        
        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let url = parts[1].to_string();
                
                if seen.insert(name.clone()) {
                    remotes.push(RemoteInfo { name, url });
                }
            }
        }
        
        Ok(remotes)
    }
    
    pub fn remote_show<S: AsRef<str>>(&self, name: S) -> Result<Remote> {
        let name_str = name.as_ref();
        
        // Get URL
        let url = self.cmd.exec_text(["remote", "get-url", name_str]).ok();
        let push_url = self.cmd.exec_text(["remote", "get-url", "--push", name_str]).ok();
        
        // Get refs
        let fetch_refspecs = self.cmd
            .exec_lines(["remote", "get-url", name_str])
            .unwrap_or_default();
        
        let mut remote = Remote {
            name: name_str.to_string(),
            url,
            push_url,
            fetch_refspecs: Vec::new(),
            push_refspecs: Vec::new(),
            branches: Vec::new(),
        };
        
        // Get more detailed info
        let show_output = self.cmd.exec_text(["remote", "show", "-n", name_str]);
        if let Ok(output) = show_output {
            for line in output.lines() {
                if line.contains("Remote branches") {
                    // Parse branches...
                }
            }
        }
        
        Ok(remote)
    }
    
    pub fn remote_add<S: AsRef<str>, U: AsRef<str>>(&self, name: S, url: U, fetch: bool) -> Result<()> {
        let mut args = vec!["remote", "add"];
        
        if !fetch {
            args.push("--no-tags");
        }
        
        args.push(name.as_ref());
        args.push(url.as_ref());
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn remote_remove<S: AsRef<str>>(&self, name: S) -> Result<()> {
        self.cmd.exec(["remote", "remove", name.as_ref()])?;
        Ok(())
    }
    
    pub fn remote_rename<S: AsRef<str>, N: AsRef<str>>(&self, old_name: S, new_name: N) -> Result<()> {
        self.cmd.exec(["remote", "rename", old_name.as_ref(), new_name.as_ref()])?;
        Ok(())
    }
    
    pub fn remote_set_url<S: AsRef<str>, U: AsRef<str>>(&self, name: S, url: U, push: bool) -> Result<()> {
        let mut args = vec!["remote", "set-url"];
        
        if push {
            args.push("--push");
        }
        
        args.push(name.as_ref());
        args.push(url.as_ref());
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn remote_prune<S: AsRef<str>>(&self, name: S, dry_run: bool) -> Result<Vec<String>> {
        let mut args = vec!["remote", "prune"];
        
        if dry_run {
            args.push("--dry-run");
        }
        
        args.push(name.as_ref());
        
        let output = self.cmd.exec_text(args)?;
        
        // Parse pruned branches from output
        let mut pruned = Vec::new();
        for line in output.lines() {
            if line.contains("* [would prune]") || line.contains("* [pruned]") {
                if let Some(remote_ref) = line.split_whitespace().last() {
                    pruned.push(remote_ref.to_string());
                }
            }
        }
        
        Ok(pruned)
    }
    
    pub fn fetch<S: AsRef<str>>(&self, remote: Option<S>, refspecs: Option<&[S]>, options: Option<FetchOptions>) -> Result<String> {
        let mut args: Vec<String> = vec!["fetch".to_string()];
        let opts = options.unwrap_or_default();
        
        if opts.all {
            args.push("--all".to_string());
        }
        
        if opts.prune {
            args.push("--prune".to_string());
        }
        
        if opts.tags {
            args.push("--tags".to_string());
        }
        
        if opts.no_tags {
            args.push("--no-tags".to_string());
        }
        
        if let Some(depth) = opts.depth {
            args.push("--depth".to_string());
            args.push(depth.to_string());
        }
        
        if opts.force {
            args.push("--force".to_string());
        }
        
        if let Some(remote) = remote {
            args.push(remote.as_ref().to_string());
        }
        
        if let Some(specs) = refspecs {
            for spec in specs {
                args.push(spec.as_ref().to_string());
            }
        }
        
        self.cmd.exec_text(args)
    }
    
    pub fn pull<S: AsRef<str>, B: AsRef<str>>(&self, remote: Option<S>, branch: Option<B>, options: Option<PullOptions>) -> Result<String> {
        let mut args: Vec<String> = vec!["pull".to_string()];
        let opts = options.unwrap_or_default();
        
        if opts.rebase {
            args.push("--rebase".to_string());
        }
        
        if opts.no_rebase {
            args.push("--no-rebase".to_string());
        }
        
        if opts.ff_only {
            args.push("--ff-only".to_string());
        }
        
        if opts.tags {
            args.push("--tags".to_string());
        }
        
        if opts.prune {
            args.push("--prune".to_string());
        }
        
        if let Some(depth) = opts.depth {
            args.push("--depth".to_string());
            args.push(depth.to_string());
        }
        
        if let Some(remote) = remote {
            args.push(remote.as_ref().to_string());
        }
        
        if let Some(branch) = branch {
            args.push(branch.as_ref().to_string());
        }
        
        self.cmd.exec_text(args)
    }
    
    pub fn push<S: AsRef<str>, R: AsRef<str>>(
        &self,
        remote: Option<S>,
        refspec: Option<R>,
        options: Option<PushOptions>,
    ) -> Result<String> {
        let mut args: Vec<String> = vec!["push".to_string()];
        let opts = options.unwrap_or_default();
        
        if opts.force {
            args.push("--force".to_string());
        }
        
        if opts.force_with_lease {
            args.push("--force-with-lease".to_string());
        }
        
        if opts.all {
            args.push("--all".to_string());
        }
        
        if opts.tags {
            args.push("--tags".to_string());
        }
        
        if opts.delete {
            args.push("--delete".to_string());
        }
        
        if opts.set_upstream {
            args.push("--set-upstream".to_string());
        }
        
        if opts.dry_run {
            args.push("--dry-run".to_string());
        }
        
        if let Some(remote) = remote {
            args.push(remote.as_ref().to_string());
        }
        
        if let Some(refspec) = refspec {
            args.push(refspec.as_ref().to_string());
        }
        
        self.cmd.exec_text(args)
    }
    
    pub fn push_simple<S: AsRef<str>>(&self, remote: S, branch: S) -> Result<()> {
        self.cmd.exec(["push", remote.as_ref(), branch.as_ref()])?;
        Ok(())
    }
}
