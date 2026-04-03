use crate::repository::Repository;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct StashEntry {
    pub index: usize,
    pub name: String,
    pub hash: String,
    pub subject: String,
    pub branch: Option<String>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_date: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct StashOptions {
    pub message: Option<String>,
    pub include_untracked: bool,
    pub keep_index: bool,
    pub patch: bool,
    pub all: bool,
}

impl StashOptions {
    pub fn message<S: AsRef<str>>(mut self, msg: S) -> Self {
        self.message = Some(msg.as_ref().to_string());
        self
    }
    
    pub fn include_untracked(mut self) -> Self {
        self.include_untracked = true;
        self
    }
    
    pub fn keep_index(mut self) -> Self {
        self.keep_index = true;
        self
    }
    
    pub fn patch(mut self) -> Self {
        self.patch = true;
        self
    }
    
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }
}

#[derive(Debug, Clone)]
pub enum StashCommand {
    Pop { stash_ref: Option<String> },
    Apply { stash_ref: Option<String> },
    Drop { stash_ref: String },
    Branch { branch_name: String, stash_ref: Option<String> },
    Clear,
}

impl Repository {
    pub fn stashes(&self) -> Result<Vec<StashEntry>> {
        let format = "%gd%x00%H%x00%s%x00%gD%x00%an%x00%ae%x00%ai";
        let output = self.cmd.exec_text(["stash", "list", &format!("--format={}", format)])?;
        
        let mut entries = Vec::new();
        
        for line in output.lines() {
            let parts: Vec<&str> = line.split('\0').collect();
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                let index = name
                    .trim_start_matches("stash@{")
                    .trim_end_matches('}')
                    .parse()
                    .unwrap_or(0);
                
                entries.push(StashEntry {
                    index,
                    name: name.clone(),
                    hash: parts[1].to_string(),
                    subject: parts[2].to_string(),
                    branch: None, // Would need to parse from subject
                    author_name: parts.get(4).map(|s| s.to_string()),
                    author_email: parts.get(5).map(|s| s.to_string()),
                    author_date: parts.get(6).map(|s| s.to_string()),
                });
            }
        }
        
        Ok(entries)
    }
    
    pub fn stash(&self, options: Option<StashOptions>) -> Result<String> {
        let mut args = vec!["stash", "push"];
        let opts = options.unwrap_or_default();
        
        if let Some(ref msg) = opts.message {
            args.push("-m");
            args.push(msg);
        }
        
        if opts.include_untracked {
            args.push("-u");
        }
        
        if opts.keep_index {
            args.push("--keep-index");
        }
        
        if opts.patch {
            args.push("-p");
        }
        
        if opts.all {
            args.push("-a");
        }
        
        let output = self.cmd.exec_text(args)?;
        
        // Parse stash reference from output
        for line in output.lines() {
            if line.contains("Saved ") {
                // Try to find stash ref
                if let Some(start) = line.find("stash@{") {
                    if let Some(end) = line[start..].find('}') {
                        return Ok(line[start..start + end + 1].to_string());
                    }
                }
            }
        }
        
        Ok(String::new())
    }
    
    pub fn stash_pop(&self, stash_ref: Option<&str>) -> Result<String> {
        let mut args = vec!["stash", "pop"];
        
        if let Some(ref_name) = stash_ref {
            args.push(ref_name);
        }
        
        self.cmd.exec_text(args)
    }
    
    pub fn stash_apply(&self, stash_ref: Option<&str>, index: bool) -> Result<String> {
        let mut args = vec!["stash", "apply"];
        
        if index {
            args.push("--index");
        }
        
        if let Some(ref_name) = stash_ref {
            args.push(ref_name);
        }
        
        self.cmd.exec_text(args)
    }
    
    pub fn stash_drop(&self, stash_ref: &str) -> Result<()> {
        self.cmd.exec(["stash", "drop", stash_ref])?;
        Ok(())
    }
    
    pub fn stash_clear(&self) -> Result<()> {
        self.cmd.exec(["stash", "clear"])?;
        Ok(())
    }
    
    pub fn stash_branch<S: AsRef<str>>(&self, branch_name: S, stash_ref: Option<&str>) -> Result<String> {
        let mut args = vec!["stash", "branch", branch_name.as_ref()];
        
        if let Some(ref_name) = stash_ref {
            args.push(ref_name);
        }
        
        self.cmd.exec_text(args)
    }
    
    pub fn stash_show(&self, stash_ref: &str, patch: bool, stat: bool) -> Result<String> {
        let mut args = vec!["stash", "show"];
        
        if patch {
            args.push("-p");
        }
        
        if stat {
            args.push("--stat");
        }
        
        args.push(stash_ref);
        
        self.cmd.exec_text(args)
    }
    
    pub fn stash_store<S: AsRef<str>, M: AsRef<str>>(&self, commit: S, message: Option<M>) -> Result<()> {
        let mut args: Vec<String> = vec!["stash".to_string(), "store".to_string(), "-m".to_string()];
        
        if let Some(msg) = message {
            args.push(msg.as_ref().to_string());
        } else {
            args.push("WIP".to_string());
        }
        
        args.push(commit.as_ref().to_string());
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn is_stash_dirty(&self) -> Result<bool> {
        match self.stashes() {
            Ok(stashes) => Ok(!stashes.is_empty()),
            Err(_) => Ok(false),
        }
    }
}
