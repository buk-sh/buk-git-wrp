use crate::repository::Repository;
use crate::error::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub hash: String,
    pub short_hash: String,
    pub author_name: String,
    pub author_email: String,
    pub author_date: String,
    pub author_timestamp: i64,
    pub committer_name: String,
    pub committer_email: String,
    pub committer_date: String,
    pub committer_timestamp: i64,
    pub subject: String,
    pub body: String,
    pub parents: Vec<String>,
    pub refs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileLogEntry {
    pub hash: String,
    pub author_name: String,
    pub author_date: String,
    pub subject: String,
    pub stat: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct LogOptions {
    pub max_count: Option<usize>,
    pub skip: Option<usize>,
    pub all: bool,
    pub decorate: bool,
    pub oneline: bool,
    pub graph: bool,
    pub reverse: bool,
    pub author: Option<String>,
    pub committer: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub grep: Option<String>,
    pub path: Option<String>,
    pub follow: bool,
}

impl LogOptions {
    pub fn max_count(mut self, count: usize) -> Self {
        self.max_count = Some(count);
        self
    }
    
    pub fn skip(mut self, count: usize) -> Self {
        self.skip = Some(count);
        self
    }
    
    pub fn all(mut self) -> Self {
        self.all = true;
        self
    }
    
    pub fn decorate(mut self) -> Self {
        self.decorate = true;
        self
    }
    
    pub fn oneline(mut self) -> Self {
        self.oneline = true;
        self
    }
    
    pub fn graph(mut self) -> Self {
        self.graph = true;
        self
    }
    
    pub fn reverse(mut self) -> Self {
        self.reverse = true;
        self
    }
    
    pub fn author<S: AsRef<str>>(mut self, author: S) -> Self {
        self.author = Some(author.as_ref().to_string());
        self
    }
    
    pub fn committer<S: AsRef<str>>(mut self, committer: S) -> Self {
        self.committer = Some(committer.as_ref().to_string());
        self
    }
    
    pub fn since<S: AsRef<str>>(mut self, since: S) -> Self {
        self.since = Some(since.as_ref().to_string());
        self
    }
    
    pub fn until<S: AsRef<str>>(mut self, until: S) -> Self {
        self.until = Some(until.as_ref().to_string());
        self
    }
    
    pub fn grep<S: AsRef<str>>(mut self, pattern: S) -> Self {
        self.grep = Some(pattern.as_ref().to_string());
        self
    }
    
    pub fn path<S: AsRef<str>>(mut self, path: S) -> Self {
        self.path = Some(path.as_ref().to_string());
        self
    }
    
    pub fn follow(mut self) -> Self {
        self.follow = true;
        self
    }
}

impl Repository {
    pub fn log(&self, options: Option<LogOptions>) -> Result<Vec<LogEntry>> {
        let opts = options.unwrap_or_default();
        
        // Use pretty format for structured output
        let format_str = "%H%x00%h%x00%an%x00%ae%x00%ai%x00%at%x00%cn%x00%ce%x00%ci%x00%ct%x00%P%x00%D%x00%s%x00%b%x00%x00";
        
        let mut args: Vec<String> = vec!["log".to_string(), format!("--format={}", format_str)];
        
        if let Some(max) = opts.max_count {
            args.push(format!("-n {}", max));
        }
        
        if let Some(skip) = opts.skip {
            args.push(format!("--skip={}", skip));
        }
        
        if opts.all {
            args.push("--all".to_string());
        }
        
        if opts.reverse {
            args.push("--reverse".to_string());
        }
        
        if let Some(ref author) = opts.author {
            args.push(format!("--author={}", author));
        }
        
        if let Some(ref committer) = opts.committer {
            args.push(format!("--committer={}", committer));
        }
        
        if let Some(ref since) = opts.since {
            args.push(format!("--since={}", since));
        }
        
        if let Some(ref until) = opts.until {
            args.push(format!("--until={}", until));
        }
        
        if let Some(ref grep) = opts.grep {
            args.push(format!("--grep={}", grep));
        }
        
        if opts.follow {
            args.push("--follow".to_string());
        }
        
        if let Some(ref path) = opts.path {
            args.push("--".to_string());
            args.push(path.clone());
        }
        
        let output = self.cmd.exec_text(args)?;
        self.parse_log_output(&output)
    }
    
    pub fn log_simple(&self, max_count: Option<usize>) -> Result<Vec<String>> {
        let mut args: Vec<String> = vec!["log".to_string(), "--oneline".to_string()];
        
        if let Some(max) = max_count {
            args.push(format!("-n {}", max));
        }
        
        let output = self.cmd.exec_text(args)?;
        Ok(output.lines().map(|l| l.to_string()).collect())
    }
    
    pub fn log_graph(&self, max_count: Option<usize>) -> Result<String> {
        let mut args: Vec<String> = vec!["log".to_string(), "--graph".to_string(), "--oneline".to_string(), "--decorate".to_string()];
        
        if let Some(max) = max_count {
            args.push(format!("-n {}", max));
        }
        
        self.cmd.exec_text(args)
    }
    
    pub fn log_file<P: AsRef<std::path::Path>>(&self, path: P, max_count: Option<usize>, follow: bool) -> Result<Vec<FileLogEntry>> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        let format_str = "%H%x00%an%x00%ai%x00%s%x00%x00";
        
        let mut args: Vec<String> = vec!["log".to_string(), format!("--format={}", format_str)];
        
        if let Some(max) = max_count {
            args.push(format!("-n {}", max));
        }
        
        if follow {
            args.push("--follow".to_string());
        }
        
        args.push("--".to_string());
        args.push(path_str);
        
        let output = self.cmd.exec_text(args)?;
        let mut entries = Vec::new();
        
        for record in output.split("\n\n") {
            let parts: Vec<&str> = record.split('\0').collect();
            if parts.len() >= 4 {
                entries.push(FileLogEntry {
                    hash: parts[0].to_string(),
                    author_name: parts[1].to_string(),
                    author_date: parts[2].to_string(),
                    subject: parts[3].to_string(),
                    stat: None,
                });
            }
        }
        
        Ok(entries)
    }
    
    pub fn rev_list<S: AsRef<str>>(&self, rev: S, max_count: Option<usize>) -> Result<Vec<String>> {
        let mut args: Vec<String> = vec!["rev-list".to_string(), rev.as_ref().to_string()];
        
        if let Some(max) = max_count {
            args.push(format!("--max-count={}", max));
        }
        
        self.cmd.exec_lines(args)
    }
    
    pub fn rev_parse<S: AsRef<str>>(&self, rev: S) -> Result<String> {
        self.cmd.exec_text(["rev-parse", rev.as_ref()])
    }
    
    pub fn get_commit_info<S: AsRef<str>>(&self, commit: S) -> Result<LogEntry> {
        let format = "%H%x00%h%x00%an%x00%ae%x00%ai%x00%at%x00%cn%x00%ce%x00%ci%x00%ct%x00%P%x00%D%x00%s%x00%b";
        
        let output = self.cmd.exec_text([
            "log",
            &format!("--format={}", format),
            "-n", "1",
            commit.as_ref(),
        ])?;
        
        let entries = self.parse_log_output(&output)?;
        entries.into_iter().next().ok_or_else(|| GitError::Other("No commit found".to_string()))
    }
    
    pub fn get_last_commit(&self) -> Result<Option<LogEntry>> {
        match self.log(Some(LogOptions::default().max_count(1))) {
            Ok(mut entries) => Ok(entries.pop()),
            Err(_) => Ok(None),
        }
    }
    
    pub fn count_commits(&self) -> Result<usize> {
        let output = self.cmd.exec_text(["rev-list", "--count", "HEAD"])?;
        output.parse().map_err(|_| GitError::Other("Failed to parse commit count".to_string()))
    }
    
    pub fn blame<P: AsRef<std::path::Path>>(&self, path: P, range: Option<(usize, usize)>) -> Result<Vec<BlameLine>> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        let mut args: Vec<String> = vec!["blame".to_string(), "--porcelain".to_string()];
        
        if let Some((start, end)) = range {
            args.push("-L".to_string());
            args.push(format!("{},{}:{}", start, end, path_str));
        } else {
            args.push("-L".to_string());
            args.push(format!("1,+1000000:{}", path_str));
        }
        
        let output = self.cmd.exec_text(args)?;
        self.parse_blame_output(&output)
    }
    
    fn parse_log_output(&self, output: &str) -> Result<Vec<LogEntry>> {
        let mut entries = Vec::new();
        
        for record in output.split("\n\n") {
            let parts: Vec<&str> = record.split('\0').collect();
            if parts.len() >= 12 {
                let parents = if parts[10].is_empty() {
                    Vec::new()
                } else {
                    parts[10].split_whitespace().map(|s| s.to_string()).collect()
                };
                
                let refs = if parts[11].is_empty() {
                    Vec::new()
                } else {
                    parts[11].split(", ").map(|s| s.to_string()).collect()
                };
                
                entries.push(LogEntry {
                    hash: parts[0].to_string(),
                    short_hash: parts[1].to_string(),
                    author_name: parts[2].to_string(),
                    author_email: parts[3].to_string(),
                    author_date: parts[4].to_string(),
                    author_timestamp: parts[5].parse().unwrap_or(0),
                    committer_name: parts[6].to_string(),
                    committer_email: parts[7].to_string(),
                    committer_date: parts[8].to_string(),
                    committer_timestamp: parts[9].parse().unwrap_or(0),
                    parents,
                    refs,
                    subject: parts[12].to_string(),
                    body: parts.get(13).unwrap_or(&"").to_string(),
                });
            }
        }
        
        Ok(entries)
    }
    
    fn parse_blame_output(&self, output: &str) -> Result<Vec<BlameLine>> {
        let mut lines = Vec::new();
        let mut current_commit = String::new();
        
        for line in output.lines() {
            if line.len() >= 40 && line.chars().next().map(|c| c.is_ascii_hexdigit()).unwrap_or(false) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    current_commit = parts[0].to_string();
                    let original_line: usize = parts[1].parse().unwrap_or(0);
                    let final_line: usize = parts[2].parse().unwrap_or(0);
                    let group_lines: usize = parts[3].parse().unwrap_or(1);
                    
                    lines.push(BlameLine {
                        commit: current_commit.clone(),
                        original_line,
                        final_line,
                        group_lines,
                        author: None,
                        author_mail: None,
                        author_time: None,
                        author_tz: None,
                        summary: None,
                        boundary: false,
                        content: None,
                    });
                }
            } else if line.starts_with('\t') {
                if let Some(last) = lines.last_mut() {
                    last.content = Some(line[1..].to_string());
                }
            } else if let Some(space_pos) = line.find(' ') {
                let key = &line[..space_pos];
                let value = &line[space_pos + 1..];
                
                if let Some(last) = lines.last_mut() {
                    match key {
                        "author" => last.author = Some(value.to_string()),
                        "author-mail" => last.author_mail = Some(value.to_string()),
                        "author-time" => last.author_time = Some(value.parse().unwrap_or(0)),
                        "author-tz" => last.author_tz = Some(value.to_string()),
                        "summary" => last.summary = Some(value.to_string()),
                        "boundary" => last.boundary = true,
                        _ => {}
                    }
                }
            }
        }
        
        Ok(lines)
    }
}

#[derive(Debug, Clone)]
pub struct BlameLine {
    pub commit: String,
    pub original_line: usize,
    pub final_line: usize,
    pub group_lines: usize,
    pub author: Option<String>,
    pub author_mail: Option<String>,
    pub author_time: Option<i64>,
    pub author_tz: Option<String>,
    pub summary: Option<String>,
    pub boundary: bool,
    pub content: Option<String>,
}

use crate::error::GitError;
