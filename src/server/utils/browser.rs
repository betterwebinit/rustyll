use log::error;
use std::process::Command;

/// Open a URL in the user's default browser
pub fn open_browser(url: &str) -> bool {
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()
            .is_ok()
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(url)
            .spawn()
            .is_ok()
    } else if cfg!(target_os = "linux") {
        // Check if we're running in WSL by checking for WSL in uname
        let is_wsl = Command::new("uname")
            .arg("-r")
            .output()
            .map(|output| {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.contains("microsoft") || output_str.contains("WSL")
            })
            .unwrap_or(false);

        if is_wsl {
            // In WSL, try powershell.exe first
            Command::new("powershell.exe")
                .args(["-Command", &format!("Start-Process '{}'", url)])
                .spawn()
                .is_ok() ||
            // Fallback to xdg-open if powershell isn't available
            Command::new("xdg-open")
                .arg(url)
                .spawn()
                .is_ok()
        } else {
            Command::new("xdg-open")
                .arg(url)
                .spawn()
                .is_ok()
        }
    } else {
        error!("Unsupported platform for opening browser");
        false
    };
    
    if !result {
        error!("Failed to open browser at {}", url);
    }
    
    result
} 