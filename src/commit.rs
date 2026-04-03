use std::path::Path;
use std::collections::HashMap;

use crate::repository::Repository;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub author_name: String,
    pub author_email: String,
    pub author_date: String,
    pub committer_name: String,
    pub committer_email: String,
    pub committer_date: String,
    pub subject: String,
    pub body: String,
    pub parents: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub old_path: String,
    pub new_path: String,
    pub status: char,  // A, D, M, R, C, T, U
    pub similarity: Option<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct CommitOptions {
    pub message: Option<String>,
    pub message_file: Option<String>,
    pub amend: bool,
    pub no_edit: bool,
    pub author: Option<String>,
    pub date: Option<String>,
    pub signoff: bool,
    pub gpg_sign: Option<bool>,
    pub allow_empty: bool,
    pub allow_empty_message: bool,
}

impl CommitOptions {
    pub fn message<S: AsRef<str>>(mut self, msg: S) -> Self {
        self.message = Some(msg.as_ref().to_string());
        self
    }
    
    pub fn message_file<S: AsRef<str>>(mut self, path: S) -> Self {
        self.message_file = Some(path.as_ref().to_string());
        self
    }
    
    pub fn amend(mut self) -> Self {
        self.amend = true;
        self
    }
    
    pub fn no_edit(mut self) -> Self {
        self.no_edit = true;
        self
    }
    
    pub fn author<S: AsRef<str>>(mut self, author: S) -> Self {
        self.author = Some(author.as_ref().to_string());
        self
    }
    
    pub fn date<S: AsRef<str>>(mut self, date: S) -> Self {
        self.date = Some(date.as_ref().to_string());
        self
    }
    
    pub fn signoff(mut self) -> Self {
        self.signoff = true;
        self
    }
    
    pub fn gpg_sign(mut self, sign: bool) -> Self {
        self.gpg_sign = Some(sign);
        self
    }
    
    pub fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }
    
    pub fn allow_empty_message(mut self) -> Self {
        self.allow_empty_message = true;
        self
    }
}

impl Repository {
    // Staging operations
    pub fn add<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.cmd.exec(["add", &path_str])?;
        Ok(())
    }
    
    pub fn add_all(&self) -> Result<()> {
        self.cmd.exec(["add", "--all"])?;
        Ok(())
    }
    
    pub fn add_update(&self) -> Result<()> {
        self.cmd.exec(["add", "-u"])?;
        Ok(())
    }
    
    pub fn add_patch<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.cmd.exec(["add", "-p", &path_str])?;
        Ok(())
    }
    
    pub fn remove<P: AsRef<Path>>(&self, path: P, cached: bool) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let mut args = vec!["rm"];
        if cached {
            args.push("--cached");
        }
        args.push(&path_str);
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn remove_all(&self, cached: bool) -> Result<()> {
        let mut args = vec!["rm", "-r"];
        if cached {
            args.push("--cached");
        }
        args.push(".");
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn unstage<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.cmd.exec(["restore", "--staged", &path_str])?;
        Ok(())
    }
    
    pub fn restore<P: AsRef<Path>>(&self, path: P, staged: bool, source: Option<&str>) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let mut args = vec!["restore"];
        
        if staged {
            args.push("--staged");
        }
        
        if let Some(src) = source {
            args.push("--source");
            args.push(src);
        }
        
        args.push(&path_str);
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn reset_soft(&self, commit: Option<&str>) -> Result<()> {
        let mut args = vec!["reset", "--soft"];
        if let Some(c) = commit {
            args.push(c);
        }
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn reset_mixed(&self, commit: Option<&str>) -> Result<()> {
        let mut args = vec!["reset", "--mixed"];
        if let Some(c) = commit {
            args.push(c);
        }
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn reset_hard(&self, commit: Option<&str>) -> Result<()> {
        let mut args = vec!["reset", "--hard"];
        if let Some(c) = commit {
            args.push(c);
        }
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn clean(&self, force: bool, directories: bool, ignored: bool) -> Result<Vec<String>> {
        let mut args = vec!["clean"];
        
        if force {
            args.push("-f");
        }
        
        if directories {
            args.push("-d");
        }
        
        if ignored {
            args.push("-x");
        }
        
        args.push("-n");  // Dry run first to get list
        
        let dry_output = self.cmd.exec_text(&args)?;
        let files: Vec<String> = dry_output.lines()
            .filter(|l| l.starts_with("Would remove "))
            .map(|l| l[14..].to_string())
            .collect();
        
        // Now actually clean
        args.pop();  // Remove -n
        if !force {
            args.push("-f");
        }
        self.cmd.exec(args)?;
        
        Ok(files)
    }
    
    // Commit operations
    pub fn commit(&self, options: CommitOptions) -> Result<String> {
        let mut args = vec!["commit"];
        
        if let Some(ref msg) = options.message {
            args.push("-m");
            args.push(msg);
        }
        
        if let Some(ref file) = options.message_file {
            args.push("-F");
            args.push(file);
        }
        
        if options.amend {
            args.push("--amend");
        }
        
        if options.no_edit {
            args.push("--no-edit");
        }
        
        if let Some(ref author) = options.author {
            args.push("--author");
            args.push(author);
        }
        
        if let Some(ref date) = options.date {
            args.push("--date");
            args.push(date);
        }
        
        if options.signoff {
            args.push("--signoff");
        }
        
        if let Some(gpg) = options.gpg_sign {
            if gpg {
                args.push("-S");
            } else {
                args.push("--no-gpg-sign");
            }
        }
        
        if options.allow_empty {
            args.push("--allow-empty");
        }
        
        if options.allow_empty_message {
            args.push("--allow-empty-message");
        }
        
        let output = self.cmd.exec_text(args)?;
        
        // Parse commit hash from output
        for line in output.lines() {
            if line.starts_with('[') {
                // Format: [main abc1234] message
                if let Some(end) = line.find(']') {
                    let parts: Vec<&str> = line[1..end].split_whitespace().collect();
                    if parts.len() >= 2 {
                        return Ok(parts[1].to_string());
                    }
                }
            }
        }
        
        Ok(String::new())
    }
    
    pub fn commit_simple(&self, message: &str) -> Result<String> {
        self.commit(CommitOptions::default().message(message))
    }
    
    // Diff operations
    pub fn diff(&self, old_commit: Option<&str>, new_commit: Option<&str>) -> Result<String> {
        let mut args = vec!["diff"];
        
        if let Some(old) = old_commit {
            args.push(old);
            if let Some(new) = new_commit {
                args.push(new);
            }
        }
        
        self.cmd.exec_text(args)
    }
    
    pub fn diff_staged(&self) -> Result<String> {
        self.cmd.exec_text(["diff", "--staged"])
    }
    
    pub fn diff_unstaged(&self) -> Result<String> {
        self.cmd.exec_text(["diff"])
    }
    
    pub fn diff_stats(&self, commit: Option<&str>) -> Result<HashMap<String, (usize, usize, usize)>> {
        let mut args = vec!["diff", "--stat"];
        
        if let Some(c) = commit {
            args.push(c);
        } else {
            args.push("--cached");
        }
        
        let output = self.cmd.exec_text(args)?;
        let mut stats = HashMap::new();
        
        for line in output.lines() {
            // Parse "file.txt | 10 +++---" format
            if let Some(pipe_pos) = line.find('|') {
                let file = line[..pipe_pos].trim().to_string();
                let rest = &line[pipe_pos + 1..];
                
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if !parts.is_empty() {
                    let changes: usize = parts[0].parse().unwrap_or(0);
                    let plus_count = parts.get(1).map(|s| s.matches('+').count()).unwrap_or(0);
                    let minus_count = parts.get(1).map(|s| s.matches('-').count()).unwrap_or(0);
                    
                    stats.insert(file, (changes, plus_count, minus_count));
                }
            }
        }
        
        Ok(stats)
    }
    
    pub fn show(&self, commit: &str) -> Result<String> {
        self.cmd.exec_text(["show", commit])
    }
    
    pub fn show_stat(&self, commit: &str) -> Result<String> {
        self.cmd.exec_text(["show", "--stat", commit])
    }
    
    pub fn show_name_only(&self, commit: &str) -> Result<Vec<String>> {
        let output = self.cmd.exec_text(["show", "--name-only", "--format=", commit])?;
        Ok(output.lines().map(|l| l.to_string()).filter(|l| !l.is_empty()).collect())
    }
}
