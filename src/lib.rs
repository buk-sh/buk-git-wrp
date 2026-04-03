//! # Git Wrapper
//! 
//! A comprehensive Rust wrapper for Git CLI operations.
//! 
//! This crate provides a high-level, type-safe interface to Git operations
//! by wrapping the git command-line tool. It supports all common git workflows
//! including repository management, staging, committing, branching, remotes,
//! and more.
//! 
//! ## Features
//! 
//! - Repository initialization, cloning, and inspection
//! - Staging and committing with flexible options
//! - Branch creation, switching, and merging
//! - Remote management and network operations (fetch, pull, push)
//! - Tag and stash operations
//! - Rebase, cherry-pick, and revert workflows
//! - Comprehensive error handling with detailed error types
//! - Support for worktrees and bisecting
//! 
//! ## Example
//! 
//! ```rust
//! use git_wrapper::{Repository, CloneOptions};
//!
//! // Clone a repository
//! let repo = Repository::clone(
//!     "https://github.com/example/repo.git",
//!     "/path/to/clone",
//!     None,
//! )?;
//!
//! // Check status
//! let status = repo.status(None)?;
//! println!("Repository is clean: {}", status.is_clean);
//!
//! // Stage files and commit
//! repo.add_all()?;
//! let commit_hash = repo.commit_simple("Initial commit")?;
//! ```

pub mod error;
pub mod command;
pub mod repository;
pub mod status;
pub mod commit;
pub mod branch;
pub mod remote;
pub mod log;
pub mod tag;
pub mod stash;
pub mod merge_ops;

pub use error::{GitError, GitCommandError, Result};
pub use command::{GitCommand, git, git_lines};
pub use repository::{Repository, RepositoryState, CloneOptions};
pub use status::{RepositoryStatus, StatusEntry, StatusOptions, FileStatus, BranchInfo, UntrackedMode};
pub use commit::{CommitOptions, CommitInfo, DiffEntry};
pub use branch::{Branch, BranchOptions};
pub use remote::{Remote, RemoteInfo, RemoteInfo as RemoteRef, PushOptions, FetchOptions, PullOptions};
pub use log::{LogEntry, LogOptions, FileLogEntry, BlameLine};
pub use tag::{Tag, TagInfo, TagOptions};
pub use stash::{StashEntry, StashOptions, StashCommand};
pub use merge_ops::{RebaseState, RebaseOptions, CherryPickOptions, RevertOptions, WorktreeEntry};

#[cfg(feature = "serde")]
pub use status::{RepositoryStatus as SerializableStatus};
