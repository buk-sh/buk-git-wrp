use git_wrapper::{Repository, BranchOptions, LogOptions, TagOptions, RebaseOptions};

fn workflow_example() -> git_wrapper::Result<()> {
    // 1. Create or open a repository
    let repo = match Repository::open(".") {
        Ok(r) => r,
        Err(_) => Repository::init("./new-repo", false)?,
    };
    
    // 2. Check current status
    let status = repo.status(None)?;
    println!("Working tree clean: {}", status.is_clean);
    
    // 3. Stage specific files
    repo.add("./src/main.rs")?;
    repo.add("./Cargo.toml")?;
    
    // 4. Check diff of staged changes
    let diff = repo.diff_staged()?;
    if !diff.is_empty() {
        println!("Staged changes:\n{}", diff);
    }
    
    // 5. Commit with options
    let commit_hash = repo.commit(
        git_wrapper::CommitOptions::default()
            .message("feat: add main functionality")
            .signoff(true)
    )?;
    println!("Created commit: {}", commit_hash);
    
    // 6. Create and switch to a feature branch
    repo.branch_create("feature/new-feature", Some("main"), None)?;
    repo.checkout("feature/new-feature", false, false)?;
    
    // 7. Make more changes and commit
    repo.add_all()?;
    let feature_commit = repo.commit_simple("feat: implement new feature")?;
    
    // 8. View log
    let log = repo.log(Some(
        LogOptions::default()
            .max_count(5)
            .decorate(true)
    ))?;
    
    for entry in log {
        println!("{} - {} by {}", 
            entry.short_hash, 
            entry.subject,
            entry.author_name
        );
    }
    
    // 9. Tag a release
    repo.tag_create("v1.0.0", &commit_hash, Some(
        TagOptions::default()
            .annotate()
            .message("Release version 1.0.0")
    ))?;
    
    // 10. Switch back to main and merge
    repo.checkout("main", false, false)?;
    repo.merge("feature/new-feature", false, false, false, false)?;
    
    // 11. Push to remote
    repo.push(Some("origin"), Some("main"), None)?;
    repo.push_tag("origin", "v1.0.0", false, false)?;
    
    // 12. Clean up feature branch
    repo.branch_delete("feature/new-feature", false)?;
    
    Ok(())
}

fn stash_example(repo: &Repository) -> git_wrapper::Result<()> {
    // Stash current changes
    let stash_ref = repo.stash(Some(
        git_wrapper::StashOptions::default()
            .message("WIP: before switching")
            .include_untracked(true)
    ))?;
    println!("Created stash: {}", stash_ref);
    
    // List stashes
    let stashes = repo.stashes()?;
    for stash in &stashes {
        println!("{}: {}", stash.name, stash.subject);
    }
    
    // Apply and pop stash
    repo.stash_apply(Some(&stash_ref), false)?;
    repo.stash_pop(None)?;
    
    Ok(())
}

fn rebase_example(repo: &Repository) -> git_wrapper::Result<()> {
    // Start interactive rebase
    repo.rebase("main", None, Some(
        RebaseOptions::default()
            .interactive()
            .autosquash(true)
    ))?;
    
    // Check rebase state
    if let Some(state) = repo.rebase_state()? {
        println!("Rebasing onto: {}", state.onto);
    }
    
    // Continue, abort, or skip as needed
    // repo.rebase_continue()?;
    // repo.rebase_abort()?;
    
    Ok(())
}

fn main() {
    if let Err(e) = workflow_example() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
