use super::entries::DetectMethod;

pub(super) fn is_available(method: &DetectMethod) -> bool {
    match method {
        DetectMethod::Path(bin) => which::which(bin).is_ok(),
        #[cfg(target_os = "macos")]
        DetectMethod::MacosApp(app) => macos_app_exists(app),
        DetectMethod::Always => true,
    }
}

#[cfg(target_os = "macos")]
fn macos_app_exists(app_name: &str) -> bool {
    let system = std::path::Path::new("/Applications").join(format!("{app_name}.app"));
    let user = dirs::home_dir().map(|h| h.join("Applications").join(format!("{app_name}.app")));
    system.exists() || user.is_some_and(|p| p.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_which_sh_exists() {
        assert!(which::which("sh").is_ok());
    }
    #[test]
    fn test_which_nonexistent() {
        assert!(which::which("__no_such_binary_xyz__").is_err());
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_available_macos_app() {
        assert!(is_available(&DetectMethod::MacosApp("Safari")));
        assert!(!is_available(&DetectMethod::MacosApp(
            "__NonExistentApp__xyz123__"
        )));
    }
}
