use crate::constants::{AUTH_FAILED, AUTH_SUCCESS};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub struct AuthResult {
    pub success: bool,
    pub message: String,
}

pub struct AuthCache {
    authenticated: bool,
    auth_time: Option<Instant>,
    duration: Duration,
}

impl AuthCache {
    pub fn new(duration_minutes: u64) -> Self {
        Self {
            authenticated: false,
            auth_time: None,
            duration: Duration::from_secs(duration_minutes * 60),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        if let Some(auth_time) = self.auth_time {
            auth_time.elapsed() < self.duration
        } else {
            false
        }
    }

    pub fn set_authenticated(&mut self, authenticated: bool) {
        self.authenticated = authenticated;
        self.auth_time = if authenticated {
            Some(Instant::now())
        } else {
            None
        };
    }
}

pub fn authenticate_admin_action() -> AuthResult {
    #[cfg(target_os = "linux")]
    {
        authenticate_linux()
    }

    #[cfg(target_os = "macos")]
    {
        authenticate_macos()
    }

    #[cfg(target_os = "windows")]
    {
        authenticate_windows()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        AuthResult {
            success: false,
            message: "Unsupported operating system".to_string(),
        }
    }
}

#[cfg(target_os = "linux")]
fn authenticate_linux() -> AuthResult {
    let pkexec_result = Command::new("pkexec")
        .arg("/bin/true")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match pkexec_result {
        Ok(status) if status.success() => AuthResult {
            success: true,
            message: AUTH_SUCCESS.to_string(),
        },
        _ => {
            let uid_result = Command::new("id").arg("-u").output();

            if let Ok(output) = uid_result {
                let uid = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if uid == "0" {
                    AuthResult {
                        success: true,
                        message: "Already running with elevated privileges".to_string(),
                    }
                } else {
                    AuthResult {
                        success: false,
                        message: AUTH_FAILED.to_string(),
                    }
                }
            } else {
                AuthResult {
                    success: false,
                    message: "Unable to check authentication status".to_string(),
                }
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn authenticate_macos() -> AuthResult {
    let script = r#"do shell script "/bin/true" with administrator privileges"#;

    let status = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(exit_status) if exit_status.success() => AuthResult {
            success: true,
            message: AUTH_SUCCESS.to_string(),
        },
        Ok(_) => AuthResult {
            success: false,
            message: AUTH_FAILED.to_string(),
        },
        Err(e) => AuthResult {
            success: false,
            message: format!("Authentication error: {}", e),
        },
    }
}

#[cfg(target_os = "windows")]
fn authenticate_windows() -> AuthResult {
    let script = r#"
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = "cmd.exe"
        $psi.Arguments = "/c exit"
        $psi.Verb = "runas"
        $psi.UseShellExecute = $true
        try {
            $process = [System.Diagnostics.Process]::Start($psi)
            $process.WaitForExit()
            exit 0
        } catch {
            exit 1
        }
    "#;

    let status = Command::new("powershell")
        .arg("-Command")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(exit_status) if exit_status.success() => AuthResult {
            success: true,
            message: AUTH_SUCCESS.to_string(),
        },
        Ok(_) => AuthResult {
            success: false,
            message: AUTH_FAILED.to_string(),
        },
        Err(e) => AuthResult {
            success: false,
            message: format!("Authentication error: {}", e),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_cache_initial_state() {
        let cache = AuthCache::new(5);
        assert!(!cache.is_authenticated());
    }

    #[test]
    fn test_auth_cache_set_and_check() {
        let mut cache = AuthCache::new(5);
        cache.set_authenticated(true);
        assert!(cache.is_authenticated());
    }

    #[test]
    fn test_auth_cache_reset() {
        let mut cache = AuthCache::new(5);
        cache.set_authenticated(true);
        cache.set_authenticated(false);
        assert!(!cache.is_authenticated());
    }

    #[test]
    fn test_auth_result_success() {
        let result = AuthResult {
            success: true,
            message: "Success".to_string(),
        };
        assert!(result.success);
    }

    #[test]
    fn test_auth_result_failure() {
        let result = AuthResult {
            success: false,
            message: "Failed".to_string(),
        };
        assert!(!result.success);
    }
}
