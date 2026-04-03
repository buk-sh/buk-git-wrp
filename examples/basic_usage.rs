use git_wrapper::{Repository, CloneOptions, CommitOptions, PushOptions, StatusOptions};
use std::env;

fn main() -> git_wrapper::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <command> [args...]", args[0]);
        println!("\nCommands:");
        println!("  clone <url> <path>     Clone a repository");
        println!("  status <path>          Show repository status");
        println!("  commit <path> <msg>    Stage all and commit");
        println!("  branches <path>        List branches");
        println!("  log <path> [n]         Show log (n commits)");
        println!("  push <path>            Push to origin");
        return Ok(());
    }
    
    match args[1].as_str() {
        "clone" => {
            if args.len() < 4 {
                eprintln!("Usage: {} clone <url> <path>", args[0]);
                return Ok(());
            }
            let url = &args[2];
            let path = &args[3];
            println!("Cloning {} into {}...", url, path);
            let repo = Repository::clone(url, path, Some(CloneOptions::default()))?;
            println!("Successfully cloned to {}", repo.path());
        }
        
        "status" => {
            if args.len() < 3 {
                eprintln!("Usage: {} status <path>", args[0]);
                return Ok(());
            }
            let path = &args[2];
            let repo = Repository::open(path)?;
            let status = repo.status(Some(StatusOptions::default()))?;
            
            println!("Repository status:");
            if status.is_clean {
                println!("  Working tree clean");
            } else {
                for entry in &status.entries {
                    println!("  {} {} - {}", 
                        format_status(&entry.index_status),
                        format_status(&entry.worktree_status),
                        entry.path
                    );
                }
            }
            
            if let Some(ref branch) = status.branch_info {
                if let Some(ref local) = branch.local_branch {
                    println!("\nOn branch: {}", local);
                }
                if let Some(ref upstream) = branch.upstream_branch {
                    println!("Tracking: {}", upstream);
                }
                if branch.ahead > 0 {
                    println!("Ahead by {} commit(s)", branch.ahead);
                }
                if branch.behind > 0 {
                    println!("Behind by {} commit(s)", branch.behind);
                }
            }
        }
        
        "commit" => {
            if args.len() < 4 {
                eprintln!("Usage: {} commit <path> <message>", args[0]);
                return Ok(());
            }
            let path = &args[2];
            let message = &args[3];
            let repo = Repository::open(path)?;
            
            // Stage all changes
            repo.add_all()?;
            println!("Staged all changes");
            
            // Commit
            let hash = repo.commit(CommitOptions::default().message(message))?;
            println!("Created commit: {}", hash);
        }
        
        "branches" => {
            if args.len() < 3 {
                eprintln!("Usage: {} branches <path>", args[0]);
                return Ok(());
            }
            let path = &args[2];
            let repo = Repository::open(path)?;
            let branches = repo.branches(false, false)?;
            
            println!("Branches:");
            for branch in &branches {
                let marker = if branch.is_current { "*" } else { " " };
                println!("{} {}", marker, branch.name);
                if let Some(ref upstream) = branch.upstream {
                    println!("    -> {} (ahead: {}, behind: {})", 
                        upstream, branch.ahead, branch.behind);
                }
            }
        }
        
        "log" => {
            if args.len() < 3 {
                eprintln!("Usage: {} log <path> [count]", args[0]);
                return Ok(());
            }
            let path = &args[2];
            let count = args.get(3).and_then(|s| s.parse().ok());
            
            let repo = Repository::open(path)?;
            let log = repo.log_simple(count)?;
            
            println!("Commit log:");
            for entry in log {
                println!("  {}", entry);
            }
        }
        
        "push" => {
            if args.len() < 3 {
                eprintln!("Usage: {} push <path>", args[0]);
                return Ok(());
            }
            let path = &args[2];
            let repo = Repository::open(path)?;
            
            match repo.current_branch()? {
                Some(branch) => {
                    println!("Pushing {} to origin...", branch);
                    let result = repo.push(Some("origin"), Some(&branch), Some(PushOptions::default()))?;
                    println!("{}", result);
                }
                None => {
                    println!("Not on any branch");
                }
            }
        }
        
        _ => {
            eprintln!("Unknown command: {}", args[1]);
        }
    }
    
    Ok(())
}

fn format_status(status: &git_wrapper::FileStatus) -> &'static str {
    use git_wrapper::FileStatus::*;
    match status {
        Unmodified => ".",
        Modified => "M",
        Added => "A",
        Deleted => "D",
        Renamed => "R",
        Copied => "C",
        UpdatedButUnmerged => "U",
        Untracked => "?",
        Ignored => "!",
    }
}
