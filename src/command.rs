use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output, Stdio, ExitStatus};

use crate::error::{GitError, GitCommandError, Result};

#[derive(Debug, Clone)]
pub struct GitCommand {
    git_executable: String,
    working_dir: Option<String>,
    env_vars: Vec<(String, String)>,
}

impl Default for GitCommand {
    fn default() -> Self {
        Self {
            git_executable: "git".to_string(),
            working_dir: None,
            env_vars: Vec::new(),
        }
    }
}

impl GitCommand {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_executable<S: AsRef<OsStr>>(executable: S) -> Self {
        Self {
            git_executable: executable.as_ref().to_string_lossy().to_string(),
            working_dir: None,
            env_vars: Vec::new(),
        }
    }
    
    pub fn with_working_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.working_dir = Some(path.as_ref().to_string_lossy().to_string());
        self
    }
    
    pub fn with_env<S: AsRef<OsStr>>(mut self, key: S, value: S) -> Self {
        self.env_vars.push((
            key.as_ref().to_string_lossy().to_string(),
            value.as_ref().to_string_lossy().to_string(),
        ));
        self
    }
    
    pub fn exec<I, S>(&self, args: I) -> Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args_vec: Vec<String> = args
            .into_iter()
            .map(|s| s.as_ref().to_string_lossy().to_string())
            .collect();
        
        let mut cmd = Command::new(&self.git_executable);
        
        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }
        
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }
        
        cmd.args(&args_vec);
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            
            return Err(GitError::from_exit_status(
                &self.git_executable,
                &args_vec,
                output.status,
                stdout,
                stderr,
            ));
        }
        
        Ok(output)
    }
    
    pub fn exec_text<I, S>(&self, args: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = self.exec(args)?;
        let text = String::from_utf8(output.stdout)?;
        Ok(text.trim().to_string())
    }
    
    pub fn exec_lines<I, S>(&self, args: I) -> Result<Vec<String>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let text = self.exec_text(args)?;
        if text.is_empty() {
            return Ok(Vec::new());
        }
        Ok(text.lines().map(|l| l.to_string()).collect())
    }
    
    pub fn exec_null_terminated<I, S>(&self, args: I) -> Result<Vec<String>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut final_args: Vec<String> = args
            .into_iter()
            .map(|s| s.as_ref().to_string_lossy().to_string())
            .collect();
        final_args.push("-z".to_string());
        
        let output = self.exec(final_args)?;
        let text = String::from_utf8(output.stdout)?;
        
        Ok(text.split('\0').map(|s| s.to_string()).filter(|s| !s.is_empty()).collect())
    }
    
    pub fn exec_streaming<I, S>(
        &self,
        args: I,
        stdout_callback: Option<Box<dyn Fn(&[u8]) + Send>>,
        stderr_callback: Option<Box<dyn Fn(&[u8]) + Send>>,
    ) -> Result<ExitStatus>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args_vec: Vec<String> = args
            .into_iter()
            .map(|s| s.as_ref().to_string_lossy().to_string())
            .collect();
        
        let mut cmd = Command::new(&self.git_executable);
        
        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }
        
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }
        
        cmd.args(&args_vec);
        
        if stdout_callback.is_some() {
            cmd.stdout(Stdio::piped());
        }
        if stderr_callback.is_some() {
            cmd.stderr(Stdio::piped());
        }
        
        let mut child = cmd.spawn()?;
        
        // TODO: Implement actual streaming with threads for stdout/stderr
        
        let status = child.wait()?;
        
        if !status.success() {
            let empty = String::new();
            return Err(GitError::Command(GitCommandError {
                command: self.git_executable.clone(),
                args: args_vec,
                stdout: empty.clone(),
                stderr: empty,
                exit_code: status.code(),
            }));
        }
        
        Ok(status)
    }
}

pub fn git<I, S>(args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    GitCommand::new().exec_text(args)
}

pub fn git_lines<I, S>(args: I) -> Result<Vec<String>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    GitCommand::new().exec_lines(args)
}
