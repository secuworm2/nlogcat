pub struct ItunesChecker;

impl ItunesChecker {
    /// iTunes(Apple Mobile Device Support) 설치 여부를 Windows 레지스트리로 확인한다.
    #[must_use]
    pub fn is_installed() -> bool {
        #[cfg(target_os = "windows")]
        {
            let result = std::process::Command::new("reg")
                .args(["query", r"HKLM\SOFTWARE\Apple Inc.\Apple Mobile Device Support"])
                .output();
            matches!(result, Ok(o) if o.status.success())
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
}
