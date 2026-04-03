use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::env;
use std::path::PathBuf;

use git_wrapper::{
    BranchOptions, CloneOptions, CommitOptions, FetchOptions, LogOptions, PushOptions,
    Repository, StashOptions, StatusOptions, TagOptions,
};

// cargo install --path . --bin gitw

#[derive(Parser)]
#[command(name = "gitw")]
#[command(about = "A Git CLI wrapper written in Rust")]
#[command(version = "0.1.0")]
struct Cli {
    /// Path to the repository (defaults to current directory)
    #[arg(short, long, global = true)]
    path: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Git repository
    Init {
        /// Path to create the repository
        path: Option<PathBuf>,
        /// Create a bare repository
        #[arg(long)]
        bare: bool,
    },

    /// Clone a repository
    Clone {
        /// Repository URL to clone
        url: String,
        /// Path to clone into
        path: Option<PathBuf>,
        /// Clone as bare repository
        #[arg(long)]
        bare: bool,
        /// Clone only the specified branch
        #[arg(long)]
        branch: Option<String>,
        /// Create a shallow clone with specified depth
        #[arg(long)]
        depth: Option<usize>,
    },

    /// Show working tree status
    Status {
        /// Show short format
        #[arg(short)]                           // show short-format
        short: bool,                            // do we use the short format?
        /// Show branch information
        #[arg(short = 'b', long)]               // use -b for short to avoid conflict with -s. (-s is for set-upstream.)
        show_branch: bool,                      // show branch boolean. Do we show it or not? 
        /// Show ignored files
        #[arg(long)]                            // CLI argument
        ignored: bool,                          // do we show ignored files?
    },

    /// Add files to the staging area
    Add {
        /// Files to add (use . for all)
        paths: Vec<PathBuf>,
        /// Stage all modified and deleted files
        #[arg(short, long)]
        all: bool,
        /// Interactively choose hunks
        #[arg(short, long)]
        patch: bool,
    },

    /// Remove files from working tree and index
    Rm {
        /// Files to remove
        paths: Vec<PathBuf>,
        /// Only remove from index
        #[arg(long)]
        cached: bool,
        #[arg(short, long)]
        recursive: bool,
    },

    /// Restore working tree files
    Restore {
        /// Files to restore
        paths: Vec<PathBuf>,
        /// Restore to index (unstage)
        #[arg(short, long)]
        staged: bool,
        /// Restore from specific commit/branch
        #[arg(short, long)]
        source: Option<String>,
    },

    /// Record changes to the repository
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: Option<String>,
        /// Amend previous commit
        #[arg(long)]
        amend: bool,
        /// Sign off the commit
        #[arg(long)]
        signoff: bool,
        /// Allow empty commit
        #[arg(long)]
        allow_empty: bool,
        /// Open editor to write message
        #[arg(short, long)]
        edit: bool,
    },

    /// Show commit history
    Log {
        /// Maximum number of commits to show
        #[arg(short, long)]
        max_count: Option<usize>,
        /// Show one line per commit
        #[arg(short, long)]
        oneline: bool,
        /// Show graph
        #[arg(long)]
        graph: bool,
        /// Show all branches
        #[arg(long)]
        all: bool,
        /// Filter by author
        #[arg(long)]
        author: Option<String>,
        /// Filter by date (since)
        #[arg(long)]
        since: Option<String>,
    },

    /// List, create, or delete branches
    Branch {
        /// Branch name to create or delete
        name: Option<String>,
        /// Delete the branch
        #[arg(short, long)]
        delete: bool,
        /// Force delete
        #[arg(short, long)]
        force: bool,
        /// List all branches (including remotes)
        #[arg(short, long)]
        all: bool,
        /// Create new branch
        #[arg(short, long)]
        create: bool,
        /// Switch to this branch after creation
        #[arg(short)]
        switch: bool,
    },

    /// Switch to a branch
    Switch {
        /// Branch to switch to
        branch: String,
        /// Create the branch if it doesn't exist
        #[arg(short, long)]
        create: bool,
        /// Force switch
        #[arg(short, long)]
        force: bool,
    },

    /// Checkout a branch or files
    Checkout {
        /// Branch or commit to checkout
        target: String,
        /// Create and checkout new branch
        #[arg(short, long)]
        new_branch: bool,
    },

    /// Show changes between commits, commit and working tree, etc.
    Diff {
        /// First commit
        old_commit: Option<String>,
        /// Second commit
        new_commit: Option<String>,
        /// Show staged changes
        #[arg(long)]
        staged: bool,
        /// Show statistics only
        #[arg(long)]
        stat: bool,
    },

    /// Manage set of tracked repositories
    Remote {
        #[command(subcommand)]
        command: RemoteCommands,
    },

    /// Download objects and refs from another repository
    Fetch {
        /// Remote to fetch from
        remote: Option<String>,
        /// Prune deleted branches
        #[arg(long)]
        prune: bool,
        /// Fetch all remotes
        #[arg(long)]
        all: bool,
    },

    /// Fetch from and integrate with another repository
    Pull {
        /// Remote to pull from
        remote: Option<String>,
        /// Branch to pull
        branch: Option<String>,
        /// Rebase instead of merge
        #[arg(long)]
        rebase: bool,
        /// Fast-forward only
        #[arg(long)]
        ff_only: bool,
    },

    /// Update remote refs along with associated objects
    Push {
        /// Remote to push to
        remote: Option<String>,
        /// Branch to push
        branch: Option<String>,
        /// Force push
        #[arg(short, long)]
        force: bool,
        /// Set upstream for the branch
        #[arg(short, long)]
        set_upstream: bool,
        /// Push all branches
        #[arg(long)]
        all: bool,
        /// Push tags
        #[arg(long)]
        tags: bool,
    },

    /// Create, list, delete or verify tags
    Tag {
        /// Tag name
        name: Option<String>,
        /// Object to tag (defaults to HEAD)
        commit: Option<String>,
        /// Delete tag
        #[arg(short, long)]
        delete: bool,
        /// List tags with pattern
        #[arg(short, long)]
        list: bool,
        /// Create annotated tag with message
        #[arg(short, long)]
        message: Option<String>,
        /// Push tag to remote after creation
        #[arg(long)]
        push: Option<String>,
    },

    /// Stash changes
    Stash {
        #[command(subcommand)]
        command: Option<StashCommands>,
        /// Include untracked files
        #[arg(short, long)]
        include_untracked: bool,
        /// Message for the stash
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Reset current HEAD to the specified state
    Reset {
        /// Commit to reset to (defaults to HEAD)
        commit: Option<String>,
        /// Reset mode (soft, mixed, hard)
        #[arg(short, long, default_value = "mixed")]
        mode: String,
    },

    /// Remove untracked files
    Clean {
        /// Actually remove files (dry-run by default)
        #[arg(short, long)]
        force: bool,
        /// Remove untracked directories
        #[arg(short, long)]
        directories: bool,
        /// Remove ignored files too
        #[arg(long)]
        ignored: bool,
    },

    /// Join two or more development histories
    Merge {
        /// Branch to merge
        branch: String,
        /// Create a merge commit even when fast-forward is possible
        #[arg(long)]
        no_ff: bool,
        /// Refuse to merge and exit with non-zero status unless fast-forward is possible
        #[arg(long)]
        ff_only: bool,
        /// Squash commits into a single commit
        #[arg(long)]
        squash: bool,
        /// Don't commit the merge
        #[arg(long)]
        no_commit: bool,
    },

    /// Reapply commits on top of another base tip
    Rebase {
        /// Branch to rebase onto
        upstream: String,
        /// Branch to rebase (defaults to current)
        branch: Option<String>,
        /// Interactive rebase
        #[arg(short, long)]
        interactive: bool,
        /// Continue rebase after resolving conflicts
        #[arg(long)]
        continue_rebase: bool,
        /// Abort rebase
        #[arg(long)]
        abort: bool,
    },

    /// Show what revision and author last modified each line
    Blame {
        /// File to blame
        file: PathBuf,
    },

    /// Get repository information
    Info {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
}

#[derive(Subcommand)]
enum RemoteCommands {
    /// List remotes
    List,
    /// Add a remote
    Add {
        /// Name of the remote
        name: String,
        /// URL of the remote
        url: String,
    },
    /// Remove a remote
    Remove {
        /// Name of the remote to remove
        name: String,
    },
    /// Rename a remote
    Rename {
        /// Old name
        old: String,
        /// New name
        new: String,
    },
    /// Set URL for a remote
    SetUrl {
        /// Name of the remote
        name: String,
        /// New URL
        url: String,
        /// Set push URL instead of fetch URL
        #[arg(long)]
        push: bool,
    },
    /// Show information about a remote
    Show {
        /// Name of the remote
        name: String,
    },
    /// Prune stale remote-tracking branches
    Prune {
        /// Name of the remote to prune
        name: String,
        /// Dry run
        #[arg(short, long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum StashCommands {
    /// List stashes
    List,
    /// Show stash changes
    Show {
        /// Stash reference (e.g., stash@{0})
        stash_ref: Option<String>,
    },
    /// Pop a stash
    Pop {
        /// Stash to pop (defaults to latest)
        stash_ref: Option<String>,
    },
    /// Apply a stash without removing it
    Apply {
        /// Stash to apply (defaults to latest)
        stash_ref: Option<String>,
    },
    /// Drop a stash
    Drop {
        /// Stash to drop (e.g., stash@{0})
        stash_ref: String,
    },
    /// Clear all stashes
    Clear,
    /// Create a branch from stash
    Branch {
        /// Branch name
        name: String,
        /// Stash reference (defaults to latest)
        stash_ref: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let repo_path = cli.path.unwrap_or_else(|| PathBuf::from("."));

    match cli.command {
        Commands::Init { path, bare } => cmd_init(path, bare),
        Commands::Clone {
            url,
            path,
            bare,
            branch,
            depth,
        } => cmd_clone(url, path, bare, branch, depth),
        Commands::Status {
            short,
            show_branch,
            ignored,
        } => cmd_status(&repo_path, short, show_branch, ignored),
        Commands::Add {
            paths,
            all,
            patch,
        } => cmd_add(&repo_path, paths, all, patch),
        Commands::Rm {
            paths,
            cached,
            recursive,
        } => cmd_rm(&repo_path, paths, cached, recursive),
        Commands::Restore {
            paths,
            staged,
            source,
        } => cmd_restore(&repo_path, paths, staged, source),
        Commands::Commit {
            message,
            amend,
            signoff,
            allow_empty,
            edit,
        } => cmd_commit(&repo_path, message, amend, signoff, allow_empty, edit),
        Commands::Log {
            max_count,
            oneline,
            graph,
            all,
            author,
            since,
        } => cmd_log(&repo_path, max_count, oneline, graph, all, author, since),
        Commands::Branch {
            name,
            delete,
            force,
            all,
            create,
            switch,
        } => cmd_branch(&repo_path, name, delete, force, all, create, switch),
        Commands::Switch {
            branch,
            create,
            force,
        } => cmd_switch(&repo_path, &branch, create, force),
        Commands::Checkout {
            target,
            new_branch,
        } => cmd_checkout(&repo_path, &target, new_branch),
        Commands::Diff {
            old_commit,
            new_commit,
            staged,
            stat,
        } => cmd_diff(&repo_path, old_commit, new_commit, staged, stat),
        Commands::Remote { command } => cmd_remote(&repo_path, command),
        Commands::Fetch { remote, prune, all } => cmd_fetch(&repo_path, remote, prune, all),
        Commands::Pull {
            remote,
            branch,
            rebase,
            ff_only,
        } => cmd_pull(&repo_path, remote, branch, rebase, ff_only),
        Commands::Push {
            remote,
            branch,
            force,
            set_upstream,
            all,
            tags,
        } => cmd_push(&repo_path, remote, branch, force, set_upstream, all, tags),
        Commands::Tag {
            name,
            commit,
            delete,
            list,
            message,
            push,
        } => cmd_tag(&repo_path, name, commit, delete, list, message, push),
        Commands::Stash {
            command,
            include_untracked,
            message,
        } => cmd_stash(&repo_path, command, include_untracked, message),
        Commands::Reset { commit, mode } => cmd_reset(&repo_path, commit, &mode),
        Commands::Clean {
            force,
            directories,
            ignored,
        } => cmd_clean(&repo_path, force, directories, ignored),
        Commands::Merge {
            branch,
            no_ff,
            ff_only,
            squash,
            no_commit,
        } => cmd_merge(&repo_path, &branch, no_ff, ff_only, squash, no_commit),
        Commands::Rebase {
            upstream,
            branch,
            interactive,
            continue_rebase,
            abort,
        } => cmd_rebase(
            &repo_path,
            &upstream,
            branch,
            interactive,
            continue_rebase,
            abort,
        ),
        Commands::Blame { file } => cmd_blame(&repo_path, &file),
        Commands::Info { detailed } => cmd_info(&repo_path, detailed),
    }
}

fn cmd_init(path: Option<PathBuf>, bare: bool) -> Result<()> {
    let path = path.unwrap_or_else(|| PathBuf::from("."));
    let repo = Repository::init(&path, bare)?;
    if bare {
        println!("{} Initialized bare repository at {}", "✓".green(), repo.path().cyan());
    } else {
        println!("{} Initialized repository at {}", "✓".green(), repo.path().cyan());
    }
    Ok(())
}

fn cmd_clone(
    url: String,
    path: Option<PathBuf>,
    bare: bool,
    branch: Option<String>,
    depth: Option<usize>,
) -> Result<()> {
    let clone_path = path.unwrap_or_else(|| {
        let default_name = url
            .split('/')
            .last()
            .and_then(|s| s.strip_suffix(".git"))
            .unwrap_or("cloned-repo");
        PathBuf::from(default_name)
    });

    let options = if bare {
        CloneOptions::default().bare()
    } else {
        CloneOptions::default()
    };
    
    let options = if let Some(b) = branch {
        options.branch(&b)
    } else {
        options
    };
    
    let options = if let Some(d) = depth {
        options.depth(d)
    } else {
        options
    };

    let repo = Repository::clone(&url, &clone_path, Some(options))?;
    println!("{} Cloned into {}", "✓".green(), repo.path().cyan());
    Ok(())
}

fn cmd_status(
    repo_path: &PathBuf,
    short: bool,
    show_branch: bool,
    ignored: bool,
) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    if short {
        let entries = repo.status_simple()?;
        for entry in entries {
            let index = format_status_char(&entry.index_status);
            let worktree = format_status_char(&entry.worktree_status);
            println!("{}{} {}", index, worktree, entry.path);
        }
        return Ok(());
    }

    let options = StatusOptions::default()
        .with_branch();
    
    let options = if ignored {
        options.with_ignored()
    } else {
        options
    };

    let status = repo.status(Some(options))?;

    if let Some(ref branch_info) = status.branch_info {
        if let Some(ref local) = branch_info.local_branch {
            print!("On branch {}", local.cyan());
            if let Some(ref upstream) = branch_info.upstream_branch {
                print!(" tracking {}", upstream.yellow());
                if branch_info.ahead > 0 || branch_info.behind > 0 {
                    print!(
                        " [ahead {}, behind {}]",
                        branch_info.ahead.to_string().green(),
                        branch_info.behind.to_string().red()
                    );
                }
            }
            println!();
        }
    }

    if status.is_clean {
        println!("{}", "nothing to commit, working tree clean".green());
        return Ok(());
    }

    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();

    for entry in &status.entries {
        match (&entry.index_status, &entry.worktree_status) {
            (FileStatus::Untracked, _) => untracked.push(entry),
            (FileStatus::Unmodified, _) => unstaged.push(entry),
            _ => {
                if entry.index_status != FileStatus::Unmodified {
                    staged.push(entry);
                }
                if entry.worktree_status != FileStatus::Unmodified 
                    && entry.worktree_status != FileStatus::Untracked {
                    unstaged.push(entry);
                }
            }
        }
    }

    if !staged.is_empty() {
        println!("\nChanges to be committed:");
        println!("  (use \"gitw restore --staged <file>...\" to unstage)");
        for entry in staged {
            println!("\t{} {}: {}", 
                format_status_char(&entry.index_status).green(),
                format_status_char(&entry.worktree_status),
                entry.path.green()
            );
        }
    }

    if !unstaged.is_empty() {
        println!("\nChanges not staged for commit:");
        println!("  (use \"gitw add <file>...\" to update)");
        println!("  (use \"gitw restore <file>...\" to discard changes)");
        for entry in unstaged {
            println!("\t{} {}: {}",
                format_status_char(&entry.index_status),
                format_status_char(&entry.worktree_status).red(),
                entry.path.red()
            );
        }
    }

    if !untracked.is_empty() {
        println!("\nUntracked files:");
        println!("  (use \"gitw add <file>...\" to include in commit)");
        for entry in untracked {
            println!("\t{}", entry.path.red());
        }
    }

    Ok(())
}

fn cmd_add(repo_path: &PathBuf, paths: Vec<PathBuf>, all: bool, _patch: bool) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    if all {
        repo.add_all()?;
        println!("{} Added all changes", "✓".green());
    } else if paths.is_empty() {
        return Err(anyhow!("No paths specified. Use -a to add all changes."));
    } else {
        for path in &paths {
            repo.add(path)?;
        }
        println!("{} Added {} file(s)", "✓".green(), paths.len());
    }
    
    Ok(())
}

fn cmd_rm(repo_path: &PathBuf, paths: Vec<PathBuf>, cached: bool, _recursive: bool) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    if paths.is_empty() {
        return Err(anyhow!("No paths specified"));
    }
    
    for path in &paths {
        repo.remove(path, cached)?;
    }
    
    if cached {
        println!("{} Removed {} file(s) from index", "✓".green(), paths.len());
    } else {
        println!("{} Deleted {} file(s)", "✓".green(), paths.len());
    }
    
    Ok(())
}

fn cmd_restore(repo_path: &PathBuf, paths: Vec<PathBuf>, staged: bool, source: Option<String>) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    if paths.is_empty() {
        return Err(anyhow!("No paths specified"));
    }
    
    for path in &paths {
        repo.restore(path, staged, source.as_deref())?;
    }
    
    if staged {
        println!("{} Unstaged {} file(s)", "✓".green(), paths.len());
    } else {
        println!("{} Restored {} file(s)", "✓".green(), paths.len());
    }
    
    Ok(())
}

fn cmd_commit(
    repo_path: &PathBuf,
    message: Option<String>,
    amend: bool,
    signoff: bool,
    allow_empty: bool,
    edit: bool,
) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    let msg = if edit || message.is_none() {
        // Open editor
        std::env::var("EDITOR")
            .ok()
            .or_else(|| Some("nano".to_string()));
        message.ok_or_else(|| anyhow!("Editor mode not yet implemented, use -m flag"))?
    } else {
        message.unwrap()
    };

    let options = CommitOptions::default()
        .message(&msg);
    
    let options = if signoff {
        options.signoff()
    } else {
        options
    };
    
    let options = if allow_empty {
        options.allow_empty()
    } else {
        options
    };
    
    let options = if amend {
        options.amend()
    } else {
        options
    };

    let hash = repo.commit(options)?;
    let short_hash = &hash[..8.min(hash.len())];
    
    if amend {
        println!("{} Amended commit {}", "✓".green(), short_hash.cyan());
    } else {
        println!("{} Created commit {}", "✓".green(), short_hash.cyan());
    }
    
    Ok(())
}

fn cmd_log(
    repo_path: &PathBuf,
    max_count: Option<usize>,
    oneline: bool,
    graph: bool,
    all: bool,
    author: Option<String>,
    since: Option<String>,
) -> Result<()> {
    let repo = Repository::open(repo_path)?;

    if oneline || graph {
        let output = if graph {
            repo.log_graph(max_count)?
        } else {
            repo.log_simple(max_count)?.join("\n")
        };
        println!("{}", output);
        return Ok(());
    }

    let options = LogOptions::default()
        .max_count(max_count.unwrap_or(10))
        .decorate();
    
    let options = if all {
        options.all()
    } else {
        options
    };
    
    let options = if let Some(a) = author {
        options.author(a)
    } else {
        options
    };
    
    let options = if let Some(s) = since {
        options.since(s)
    } else {
        options
    };

    let log = repo.log(Some(options))?;
    
    for entry in log {
        let short_hash = &entry.hash[..8.min(entry.hash.len())];
        println!("{} {} - {}",
            short_hash.cyan(),
            entry.author_name.yellow(),
            entry.subject
        );
        let date = entry.author_date.split_whitespace().next().unwrap_or("");
        println!("  {} {}", "Date:".dimmed(), date.dimmed());
        if !entry.body.trim().is_empty() {
            for line in entry.body.lines() {
                println!("  {}", line);
            }
        }
        println!();
    }
    
    Ok(())
}

fn cmd_branch(
    repo_path: &PathBuf,
    name: Option<String>,
    delete: bool,
    force: bool,
    all: bool,
    create: bool,
    switch_after: bool,
) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    if delete {
        let branch_name = name.ok_or_else(|| anyhow!("Branch name required for delete"))?;
        repo.branch_delete(&branch_name, force)?;
        println!("{} Deleted branch {}", "✓".green(), branch_name.red());
    } else if create || (name.is_some() && switch_after) {
        let branch_name = name.ok_or_else(|| anyhow!("Branch name required"))?;
        let opts = BranchOptions::default();
        repo.branch_create(&branch_name, None::<String>, Some(opts))?;
        
        if switch_after {
            repo.switch(&branch_name, false, false, false)?;
            println!("{} Created and switched to branch {}", "✓".green(), branch_name.cyan());
        } else {
            println!("{} Created branch {}", "✓".green(), branch_name.cyan());
        }
    } else if let Some(branch_name) = name {
        // Just show info about a specific branch
        let branches = repo.branches(all, false)?;
        if let Some(branch) = branches.iter().find(|b| b.name == branch_name) {
            println!("Branch: {}", branch.name.cyan());
            println!("  Commit: {}", branch.commit_hash);
            println!("  Message: {}", branch.commit_message);
            if let Some(ref upstream) = branch.upstream {
                println!("  Upstream: {} (ahead {}, behind {})", 
                    upstream, branch.ahead, branch.behind);
            }
        } else {
            return Err(anyhow!("Branch '{}' not found", branch_name));
        }
    } else {
        // List branches
        let branches = repo.branches(all, false)?;
        let current = repo.current_branch()?;
        
        for branch in branches {
            let marker = if Some(branch.name.clone()) == current {
                "*".green()
            } else {
                " ".normal()
            };
            
            let name: String = if Some(branch.name.clone()) == current {
                branch.name.cyan().to_string()
            } else {
                branch.name.clone()
            };
            
            let short_hash = &branch.commit_hash[..8.min(branch.commit_hash.len())];
            print!("{} {} {} {}", 
                marker,
                name,
                short_hash.cyan(),
                branch.commit_message
            );
            
            if let Some(ref upstream) = branch.upstream {
                print!(" [{}]", upstream.yellow());
            }
            println!();
        }
    }
    
    Ok(())
}

fn cmd_switch(repo_path: &PathBuf, branch: &str, create: bool, force: bool) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    repo.switch(branch, create, force, false)?;
    
    if create {
        println!("{} Created and switched to branch {}", "✓".green(), branch.cyan());
    } else {
        println!("{} Switched to branch {}", "✓".green(), branch.cyan());
    }
    
    Ok(())
}

fn cmd_checkout(repo_path: &PathBuf, target: &str, new_branch: bool) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    repo.checkout(target, new_branch, false)?;
    
    if new_branch {
        println!("{} Created and checked out branch {}", "✓".green(), target.cyan());
    } else {
        println!("{} Checked out {}", "✓".green(), target.cyan());
    }
    
    Ok(())
}

fn cmd_diff(
    repo_path: &PathBuf,
    old_commit: Option<String>,
    new_commit: Option<String>,
    staged: bool,
    stat: bool,
) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    let output = if stat {
        if staged {
            repo.diff_stats(None)?;
            "".to_string()
        } else {
            repo.diff_stats(old_commit.as_deref())?;
            "".to_string()
        }
    } else if staged {
        repo.diff_staged()?
    } else {
        repo.diff(old_commit.as_deref(), new_commit.as_deref())?
    };
    
    if !output.is_empty() {
        print!("{}", output);
    }
    
    Ok(())
}

fn cmd_remote(repo_path: &PathBuf, command: RemoteCommands) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    match command {
        RemoteCommands::List => {
            let remotes = repo.remotes()?;
            if remotes.is_empty() {
                println!("No remotes configured");
            } else {
                for remote in remotes {
                    println!("{}\t{}", remote.name.green(), remote.url);
                }
            }
        }
        RemoteCommands::Add { name, url } => {
            repo.remote_add(&name, &url, true)?;
            println!("{} Added remote {} -> {}", "✓".green(), name.cyan(), url);
        }
        RemoteCommands::Remove { name } => {
            repo.remote_remove(&name)?;
            println!("{} Removed remote {}", "✓".green(), name.red());
        }
        RemoteCommands::Rename { old, new } => {
            repo.remote_rename(&old, &new)?;
            println!("{} Renamed remote {} -> {}", "✓".green(), old.red(), new.cyan());
        }
        RemoteCommands::SetUrl { name, url, push } => {
            repo.remote_set_url(&name, &url, push)?;
            if push {
                println!("{} Set push URL for {} -> {}", "✓".green(), name.cyan(), url);
            } else {
                println!("{} Set URL for {} -> {}", "✓".green(), name.cyan(), url);
            }
        }
        RemoteCommands::Show { name } => {
            let remote = repo.remote_show(&name)?;
            println!("Remote: {}", remote.name.cyan());
            if let Some(url) = remote.url {
                println!("  Fetch URL: {}", url);
            }
            if let Some(push_url) = remote.push_url {
                println!("  Push URL: {}", push_url);
            }
        }
        RemoteCommands::Prune { name, dry_run } => {
            let pruned = repo.remote_prune(&name, dry_run)?;
            if pruned.is_empty() {
                println!("No stale branches to prune");
            } else {
                for branch in pruned {
                    println!("{} {}", "Pruned:".green(), branch);
                }
            }
        }
    }
    
    Ok(())
}

fn cmd_fetch(repo_path: &PathBuf, remote: Option<String>, prune: bool, all: bool) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    let options = FetchOptions::default();
    
    let options = if prune {
        options.prune()
    } else {
        options
    };
    
    let options = if all {
        options.all()
    } else {
        options
    };
    
    let output = repo.fetch(remote.as_deref(), None::<&[&str]>, Some(options))?;
    print!("{}", output);
    
    Ok(())
}

fn cmd_pull(
    repo_path: &PathBuf,
    remote: Option<String>,
    branch: Option<String>,
    rebase: bool,
    ff_only: bool,
) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    let options = git_wrapper::PullOptions::default();
    
    let options = if rebase {
        options.rebase()
    } else {
        options
    };
    
    let options = if ff_only {
        options.ff_only()
    } else {
        options
    };
    
    let output = repo.pull(remote.as_deref(), branch.as_deref(), Some(options))?;
    print!("{}", output);
    
    Ok(())
}

fn cmd_push(
    repo_path: &PathBuf,
    remote: Option<String>,
    branch: Option<String>,
    force: bool,
    set_upstream: bool,
    all: bool,
    tags: bool,
) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    let options = if force {
        PushOptions::default().force()
    } else {
        PushOptions::default()
    };
    
    let options = if set_upstream {
        options.set_upstream()
    } else {
        options
    };
    
    let options = if all {
        options.all()
    } else {
        options
    };
    
    let options = if tags {
        options.tags()
    } else {
        options
    };
    
    let output = repo.push(remote.as_deref(), branch.as_deref(), Some(options))?;
    print!("{}", output);
    
    Ok(())
}

fn cmd_tag(
    repo_path: &PathBuf,
    name: Option<String>,
    commit: Option<String>,
    delete: bool,
    list: bool,
    message: Option<String>,
    push_remote: Option<String>,
) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    if delete {
        let tag_name = name.ok_or_else(|| anyhow!("Tag name required for delete"))?;
        repo.tag_delete(&tag_name)?;
        println!("{} Deleted tag {}", "✓".green(), tag_name.red());
    } else if list || name.is_none() {
        let tags = repo.tags(None)?;
        for tag in tags {
            println!("{}", tag.name.cyan());
        }
    } else {
        let tag_name = name.unwrap();
        let target = commit.as_deref().unwrap_or("HEAD");
        
        let options = if let Some(ref msg) = message {
            TagOptions::default().annotate().message(msg)
        } else {
            TagOptions::default()
        };
        
        repo.tag_create(&tag_name, target, Some(options))?;
        println!("{} Created tag {} -> {}", "✓".green(), tag_name.cyan(), target);
        
        if let Some(remote) = push_remote {
            repo.push_tag(&remote, &tag_name, false, false)?;
            println!("{} Pushed tag {} to {}", "✓".green(), tag_name.cyan(), remote);
        }
    }
    
    Ok(())
}

fn cmd_stash(
    repo_path: &PathBuf,
    command: Option<StashCommands>,
    include_untracked: bool,
    message: Option<String>,
) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    match command {
        None => {
            // Push stash
    let options = if include_untracked {
        StashOptions::default().include_untracked()
    } else {
        StashOptions::default()
    };
    
    let options = options.message(message.unwrap_or_else(|| "WIP".to_string()));
            
            let stash_ref = repo.stash(Some(options))?;
            println!("{} Stashed changes {}", "✓".green(), stash_ref.cyan());
        }
        Some(StashCommands::List) => {
            let stashes = repo.stashes()?;
            for stash in stashes {
                println!("{}: {}", stash.name.cyan(), stash.subject);
            }
        }
        Some(StashCommands::Show { stash_ref }) => {
            let output = repo.stash_show(stash_ref.as_deref().unwrap_or("stash@{0}"), true, false)?;
            print!("{}", output);
        }
        Some(StashCommands::Pop { stash_ref }) => {
            let output = repo.stash_pop(stash_ref.as_deref())?;
            print!("{}", output);
            println!("{} Popped stash", "✓".green());
        }
        Some(StashCommands::Apply { stash_ref }) => {
            let output = repo.stash_apply(stash_ref.as_deref(), false)?;
            print!("{}", output);
            println!("{} Applied stash", "✓".green());
        }
        Some(StashCommands::Drop { stash_ref }) => {
            repo.stash_drop(&stash_ref)?;
            println!("{} Dropped {}", "✓".green(), stash_ref.red());
        }
        Some(StashCommands::Clear) => {
            repo.stash_clear()?;
            println!("{} Cleared all stashes", "✓".green());
        }
        Some(StashCommands::Branch { name, stash_ref }) => {
            repo.stash_branch(&name, stash_ref.as_deref())?;
            println!("{} Created branch {} from stash", "✓".green(), name.cyan());
        }
    }
    
    Ok(())
}

fn cmd_reset(repo_path: &PathBuf, commit: Option<String>, mode: &str) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    let target = commit.as_deref();
    
    match mode {
        "soft" => repo.reset_soft(target)?,
        "hard" => repo.reset_hard(target)?,
        _ => repo.reset_mixed(target)?,
    }
    
    println!("{} Reset to {}", "✓".green(), target.unwrap_or("HEAD").cyan());
    Ok(())
}

fn cmd_clean(repo_path: &PathBuf, force: bool, directories: bool, ignored: bool) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    if !force {
        println!("Dry run (use -f to actually remove files):");
    }
    
    let files = repo.clean(force, directories, ignored)?;
    
    if files.is_empty() {
        println!("No files to remove");
    } else {
        for file in files {
            if force {
                println!("{} Removed {}", "✓".green(), file);
            } else {
                println!("Would remove {}", file);
            }
        }
    }
    
    Ok(())
}

fn cmd_merge(
    repo_path: &PathBuf,
    branch: &str,
    no_ff: bool,
    ff_only: bool,
    squash: bool,
    no_commit: bool,
) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    repo.merge(branch, no_ff, ff_only, squash, no_commit)?;
    
    if squash {
        println!("{} Squashed {} into working tree", "✓".green(), branch.cyan());
    } else if no_commit {
        println!("{} Merged {} (not committed)", "✓".green(), branch.cyan());
    } else {
        println!("{} Merged {}", "✓".green(), branch.cyan());
    }
    
    Ok(())
}

fn cmd_rebase(
    repo_path: &PathBuf,
    upstream: &str,
    branch: Option<String>,
    interactive: bool,
    continue_rebase: bool,
    abort: bool,
) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    if continue_rebase {
        repo.rebase_continue()?;
        println!("{} Continued rebase", "✓".green());
    } else if abort {
        repo.rebase_abort()?;
        println!("{} Aborted rebase", "✓".green());
    } else {
        let options = git_wrapper::RebaseOptions::default();
        
        let options = if interactive {
            options.interactive()
        } else {
            options
        };
        
        repo.rebase(upstream, branch, Some(options))?;
        println!("{} Rebased onto {}", "✓".green(), upstream.cyan());
    }
    
    Ok(())
}

fn cmd_blame(repo_path: &PathBuf, file: &PathBuf) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    let lines = repo.blame(file, None)?;
    
    for line in lines {
        let commit = line.commit[..8].to_string().cyan();
        let content = line.content.unwrap_or_default();
        println!("{} {:4} {}", commit, line.final_line, content);
    }
    
    Ok(())
}

fn cmd_info(repo_path: &PathBuf, detailed: bool) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    
    println!("Repository: {}", repo.path().cyan());
    
    if let Ok(git_dir) = repo.git_dir() {
        println!("Git directory: {}", git_dir.dimmed());
    }
    
    if let Ok(Some(work_dir)) = repo.work_dir() {
        println!("Working directory: {}", work_dir.dimmed());
    }
    
    if let Ok(bare) = repo.is_bare() {
        println!("Bare: {}", if bare { "yes".yellow() } else { "no".normal() });
    }
    
    if let Ok(shallow) = repo.is_shallow() {
        println!("Shallow: {}", if shallow { "yes".yellow() } else { "no".normal() });
    }
    
    if let Ok(state) = repo.state() {
        println!("State: {:?}", state);
    }
    
    if let Ok(Some(current)) = repo.current_branch() {
        println!("Current branch: {}", current.cyan());
    }
    
    if detailed {
        if let Ok(remotes) = repo.remotes() {
            println!("\nRemotes:");
            for remote in remotes {
                println!("  {} -> {}", remote.name.green(), remote.url);
            }
        }
        
        if let Ok(count) = repo.count_commits() {
            println!("\nTotal commits: {}", count);
        }
    }
    
    Ok(())
}

use git_wrapper::FileStatus;

fn format_status_char(status: &FileStatus) -> String {
    use git_wrapper::FileStatus::*;
    let c = match status {
        Unmodified => " ",
        Modified => "M",
        Added => "A",
        Deleted => "D",
        Renamed => "R",
        Copied => "C",
        UpdatedButUnmerged => "U",
        Untracked => "?",
        Ignored => "!",
    };
    c.to_string()
}

// Trait for normal styling that works with colored crate
trait Styled {
    fn normal(&self) -> String;
}

impl Styled for String {
    fn normal(&self) -> String {
        self.clone()
    }
}

impl Styled for &str {
    fn normal(&self) -> String {
        self.to_string()
    }
}
