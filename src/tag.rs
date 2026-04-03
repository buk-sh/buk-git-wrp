use crate::repository::Repository;
use crate::error::{GitError, Result};

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub object: String,
    pub object_type: String,
    pub tagger_name: Option<String>,
    pub tagger_email: Option<String>,
    pub tagger_date: Option<String>,
    pub message: Option<String>,
    pub is_annotated: bool,
}

#[derive(Debug, Clone)]
pub struct TagInfo {
    pub name: String,
    pub ref_name: String,
}

#[derive(Debug, Default, Clone)]
pub struct TagOptions {
    pub annotate: bool,
    pub message: Option<String>,
    pub sign: bool,
    pub force: bool,
    pub delete: bool,
}

impl TagOptions {
    pub fn annotate(mut self) -> Self {
        self.annotate = true;
        self
    }
    
    pub fn message<S: AsRef<str>>(mut self, msg: S) -> Self {
        self.message = Some(msg.as_ref().to_string());
        self
    }
    
    pub fn sign(mut self) -> Self {
        self.sign = true;
        self
    }
    
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }
    
    pub fn delete(mut self) -> Self {
        self.delete = true;
        self
    }
}

impl Repository {
    pub fn tags(&self, pattern: Option<&str>) -> Result<Vec<TagInfo>> {
        let mut args = vec!["tag", "-l"];
        
        if let Some(pat) = pattern {
            args.push(pat);
        }
        
        let output = self.cmd.exec_text(args)?;
        let mut tags = Vec::new();
        
        for line in output.lines() {
            if !line.is_empty() {
                tags.push(TagInfo {
                    name: line.to_string(),
                    ref_name: format!("refs/tags/{}", line),
                });
            }
        }
        
        Ok(tags)
    }
    
    pub fn tags_with_info(&self, pattern: Option<&str>) -> Result<Vec<Tag>> {
        let infos = self.tags(pattern)?;
        let mut tags = Vec::new();
        
        for info in infos {
            match self.tag_info(&info.name) {
                Ok(tag) => tags.push(tag),
                Err(_) => {
                    // If we can't get full info, create a basic tag
                    tags.push(Tag {
                        name: info.name,
                        object: String::new(),
                        object_type: "commit".to_string(),
                        tagger_name: None,
                        tagger_email: None,
                        tagger_date: None,
                        message: None,
                        is_annotated: false,
                    });
                }
            }
        }
        
        Ok(tags)
    }
    
    pub fn tag_info<S: AsRef<str>>(&self, name: S) -> Result<Tag> {
        let name_str = name.as_ref();
        
        // Check if it's an annotated tag
        let format_output = self.cmd.exec_text([
            "for-each-ref",
            "--format=%(objecttype)%(objectname) %(taggername) %(taggeremail) %(taggerdate) %(subject)",
            &format!("refs/tags/{}", name_str),
        ]);
        
        let is_annotated = format_output.as_ref().map(|o| o.contains("tag")).unwrap_or(false);
        
        let object = if is_annotated {
            self.cmd.exec_text(["rev-parse", &format!("refs/tags/{name_str}^{{}}")])?
        } else {
            self.cmd.exec_text(["rev-parse", &format!("refs/tags/{}", name_str)])?
        };
        
        let mut tagger_name = None;
        let mut tagger_email = None;
        let mut tagger_date = None;
        let mut message = None;
        
        if is_annotated {
            if let Ok(output) = format_output {
                let parts: Vec<&str> = output.splitn(5, ' ').collect();
                if parts.len() >= 5 {
                    // parts[0] contains objecttype + objectname
                    tagger_name = Some(parts[1].to_string());
                    tagger_email = Some(parts[2].to_string());
                    tagger_date = Some(parts[3].to_string());
                    message = Some(parts[4].to_string());
                }
            }
        }
        
        Ok(Tag {
            name: name_str.to_string(),
            object,
            object_type: if is_annotated { "tag".to_string() } else { "commit".to_string() },
            tagger_name,
            tagger_email,
            tagger_date,
            message,
            is_annotated,
        })
    }
    
    pub fn tag_create<S: AsRef<str>, T: AsRef<str>>(&self, name: S, target: T, options: Option<TagOptions>) -> Result<()> {
        let mut args = vec!["tag"];
        let opts = options.unwrap_or_default();
        
        if opts.delete {
            args.push("-d");
            args.push(name.as_ref());
            self.cmd.exec(args)?;
            return Ok(());
        }
        
        if opts.annotate || opts.message.is_some() || opts.sign {
            args.push("-a");
        }
        
        if let Some(ref msg) = opts.message {
            args.push("-m");
            args.push(msg);
        }
        
        if opts.sign {
            args.push("-s");
        }
        
        if opts.force {
            args.push("-f");
        }
        
        args.push(name.as_ref());
        args.push(target.as_ref());
        
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn tag_delete<S: AsRef<str>>(&self, name: S) -> Result<()> {
        self.cmd.exec(["tag", "-d", name.as_ref()])?;
        Ok(())
    }
    
    pub fn tag_delete_multiple<S: AsRef<str>>(&self, names: &[S]) -> Result<()> {
        let mut args = vec!["tag", "-d"];
        for name in names {
            args.push(name.as_ref());
        }
        self.cmd.exec(args)?;
        Ok(())
    }
    
    pub fn tag_verify<S: AsRef<str>>(&self, name: S) -> Result<String> {
        self.cmd.exec_text(["tag", "-v", name.as_ref()])
    }
    
    pub fn push_tag<S: AsRef<str>, R: AsRef<str>>(&self, remote: R, tag: S, force: bool, all: bool) -> Result<String> {
        let mut args = vec!["push"];
        
        if force {
            args.push("--force");
        }
        
        args.push(remote.as_ref());
        
        if all {
            args.push("--tags");
        } else {
            args.push(tag.as_ref());
        }
        
        self.cmd.exec_text(args)
    }
    
    pub fn describe<S: AsRef<str>>(&self, commit: S, all_tags: bool, exact_match: bool, long: bool) -> Result<String> {
        let mut args = vec!["describe"];
        
        if all_tags {
            args.push("--all");
        }
        
        if exact_match {
            args.push("--exact-match");
        }
        
        if long {
            args.push("--long");
        }
        
        args.push(commit.as_ref());
        
        self.cmd.exec_text(args)
    }
    
    pub fn get_tags_containing<S: AsRef<str>>(&self, commit: S) -> Result<Vec<String>> {
        self.cmd.exec_lines(["tag", "--contains", commit.as_ref()])
    }
    
    pub fn get_tags_pointing_at<S: AsRef<str>>(&self, commit: S) -> Result<Vec<String>> {
        self.cmd.exec_lines(["tag", "--points-at", commit.as_ref()])
    }
}
