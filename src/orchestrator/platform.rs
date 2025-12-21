use sysinfo::System;
use std::time::Duration;
use std::process::Command;

/// Attempt to stop the named processes. Best-effort and tolerant of partial
/// failures. This mirrors the old shell logic but runs in-process so callers
/// can rely on a single API.
pub fn stop_all(targets: &[&str]) -> anyhow::Result<()> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    let mut matches: Vec<u32> = Vec::new();

    for (pid, proc_) in sys.processes() {
        let name = proc_.name().to_lowercase();
        let cmd_line: String = proc_.cmd().join(" ").to_lowercase();
        
        for &t in targets {
            let t_lc = t.to_lowercase();
            
            // Direct name match for shimmy processes
            if name == t_lc || name.ends_with(&format!("/{}", t_lc)) {
                matches.push(pid.as_u32());
                break;
            }
            
            // For node/npm processes, only kill if started from a shimmy-related directory
            // (theme-generator/themes, themes/, or has shimmy in the path)
            if (name == "node" || name == "node.exe" || name == "npm" || name == "npm.exe") 
                && (cmd_line.contains("theme-generator/themes") 
                    || cmd_line.contains("theme-generator\\themes")
                    || cmd_line.contains("/themes/")
                    || cmd_line.contains("\\themes\\")
                    || cmd_line.contains("shimmy")) {
                eprintln!("found shimmy-related node process (pid={}) cmd: {}", pid, &cmd_line[..cmd_line.len().min(100)]);
                matches.push(pid.as_u32());
                break;
            }
        }
    }

    if matches.is_empty() {
        // nothing to stop — return Ok as this is best-effort
        return Ok(());
    }

    // Force kill processes immediately - these are dev processes, no need for graceful shutdown
    for pid in &matches {
        force_kill_process(*pid);
        eprintln!("killed pid={}", pid);
    }

    // Brief wait to let OS clean up
    std::thread::sleep(Duration::from_millis(500));

    Ok(())
}

/// Force kill a process by PID - platform specific  
fn force_kill_process(pid: u32) {
    #[cfg(target_os = "windows")]
    {
        // Use cmd /C to run taskkill reliably on Windows
        let result = Command::new("cmd")
            .args(["/C", &format!("taskkill /F /PID {}", pid)])
            .output();
        if let Err(e) = result {
            eprintln!("taskkill failed for pid={}: {}", pid, e);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_all_noop() {
        // Non-existent pattern should be a no-op and return Ok
        let res = stop_all(&["not-a-real-process-pleasetest"]);
        assert!(res.is_ok());
    }
}
