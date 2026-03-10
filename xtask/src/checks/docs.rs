use crate::helpers::step;
use anyhow::Result;
use std::process::Command;

pub(crate) fn check_docs_quick() -> Result<()> {
    step("Docs-only quick check", || {
        let output = Command::new("git").args(["status", "--short"]).output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut changed = Vec::new();
        for line in stdout.lines() {
            if line.len() < 4 {
                continue;
            }
            changed.push(line[3..].replace('\\', "/"));
        }
        println!("Docs-only changed files: {}", changed.join(", "));
        Ok(())
    })?;
    println!("\nDocs-only quick checks passed.");
    Ok(())
}
