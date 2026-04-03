use std::fmt;
use std::io;
use std::process::ExitStatus;
use std::string::FromUtf8Error;

pub type Result<T> = std::result::Result<T, GitError>;

#[derive(Debug)]
pub struct GitCommandError {
    pub command: String,
    pub args: Vec<String>,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl fmt::Display for GitCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Git command failed: {} {}\nstdout: {}\nstderr: {}\nexit_code: {:?}",
            self.command,
            self.args.join(" "),
            self.stdout,
            self.stderr,
            self.exit_code
        )
    }
}

impl std::error::Error for GitCommandError {}

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Command execution failed: {0}")]
    Command(#[from] GitCommandError),
    
    #[error("Invalid UTF-8 output: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),
    
    #[error("Not a git repository")]
    NotARepository,
    
    #[error("Repository already exists at path: {0}")]
    RepositoryAlreadyExists(String),
    
    #[error("Branch not found: {0}")]
    BranchNotFound(String),
    
    #[error("Remote not found: {0}")]
    RemoteNotFound(String),
    
    #[error("Tag not found: {0}")]
    TagNotFound(String),
    
    #[error("Commit not found: {0}")]
    CommitNotFound(String),
    
    #[error("Merge conflict")]
    MergeConflict,
    
    #[error("Rebase in progress")]
    RebaseInProgress,
    
    #[error("Cherry-pick in progress")]
    CherryPickInProgress,
    
    #[error("Revert in progress")]
    RevertInProgress,
    
    #[error("Working tree is not clean")]
    DirtyWorkingTree,
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Invalid reference: {0}")]
    InvalidReference(String),
    
    #[error("Pathspec error: {0}")]
    PathspecError(String),
    
    #[error("Other error: {0}")]
    Other(String),
}

impl GitError {
    pub fn from_exit_status(command: &str, args: &[String], status: ExitStatus, stdout: String, stderr: String) -> Self {
        let exit_code = status.code();
        
        let cmd_error = GitCommandError {
            command: command.to_string(),
            args: args.to_vec(),
            stdout,
            stderr: stderr.clone(),
            exit_code,
        };
        
        // Try to parse common git error patterns from stderr
        if stderr.contains("not a git repository") {
            return GitError::NotARepository;
        }
        if stderr.contains("already exists") {
            return GitError::RepositoryAlreadyExists("".to_string());
        }
        if stderr.contains("could not resolve") || stderr.contains("does not exist") {
            if stderr.contains("remote") {
                return GitError::RemoteNotFound("unknown".to_string());
            }
            if stderr.contains("branch") {
                return GitError::BranchNotFound("unknown".to_string());
            }
            if stderr.contains("tag") {
                return GitError::TagNotFound("unknown".to_string());
            }
            if stderr.contains("commit") || stderr.contains("object") {
                return GitError::CommitNotFound("unknown".to_string());
            }
        }
        if stderr.contains("merge conflict") || stderr.contains("conflict") {
            return GitError::MergeConflict;
        }
        if stderr.contains("authentication") || stderr.contains("permission denied") {
            return GitError::AuthenticationFailed;
        }
        if stderr.contains("rebase") && stderr.contains("in progress") {
            return GitError::RebaseInProgress;
        }
        
        GitError::Command(cmd_error)
    }
}
