use crate::error::AppError;
use crate::install;
use crate::logging;
use crate::process;
use std::path::Path;
use tauri::AppHandle;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SystemReport {
    pub flexasio_version: Option<String>,
    pub flexasio_install_dir: Option<String>,
    pub config_path: String,
    pub config_exists: bool,
    pub log_path: String,
    pub log_exists: bool,
    pub os_version: String,
}

pub fn gather(app: &AppHandle) -> Result<SystemReport, AppError> {
    let flexasio_tool = install::find_tool(app, "PortAudioDevices.exe");
    let flexasio_install_dir = flexasio_tool
        .as_deref()
        .and_then(Path::parent)
        .map(|dir| dir.display().to_string());

    let config_path = crate::config::path()?;
    let config_exists = config_path.exists();

    let log_path = logging::path()?;
    let log_exists = log_path.exists();

    Ok(SystemReport {
        flexasio_version: read_flexasio_version(),
        flexasio_install_dir,
        config_path: config_path.display().to_string(),
        config_exists,
        log_path: log_path.display().to_string(),
        log_exists,
        os_version: read_os_version(),
    })
}

const TARGET_PUBLISHER: &str = "Etienne Dechamps";
const UNINSTALL_KEYS: [&str; 2] = [
    r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
];

/// FlexASIO's binaries carry no VERSIONINFO in practice, so read the
/// installer's registry entry instead - filtered by publisher, not name,
/// since an older third-party FlexASIO GUI is also commonly installed.
/// Uses `reg query` rather than PowerShell: much faster, and unaffected by
/// execution-policy/AppLocker restrictions that can block PowerShell.
fn read_flexasio_version() -> Option<String> {
    UNINSTALL_KEYS.iter().find_map(|key| {
        let output = process::hidden_command("reg")
            .args(["query", key, "/s"])
            .output()
            .ok()?;
        parse_reg_query_output(&String::from_utf8_lossy(&output.stdout))
    })
}

fn parse_reg_query_output(output: &str) -> Option<String> {
    split_into_blocks(output)
        .into_iter()
        .find(|block| find_field(block, "Publisher").as_deref() == Some(TARGET_PUBLISHER))
        .and_then(|block| find_field(&block, "DisplayVersion"))
}

/// `reg query ... /s` prints one un-indented key header per entry, followed
/// by its indented `Name    REG_SZ    Value` lines; group those together.
fn split_into_blocks(output: &str) -> Vec<String> {
    let mut blocks: Vec<String> = Vec::new();
    for line in output.lines() {
        let is_header = line.starts_with(|c: char| !c.is_whitespace()) && !line.trim().is_empty();
        if is_header || blocks.is_empty() {
            blocks.push(String::new());
        }
        let block = blocks.last_mut().unwrap();
        block.push_str(line);
        block.push('\n');
    }
    blocks
}

fn find_field(block: &str, name: &str) -> Option<String> {
    block.lines().find_map(|line| {
        let rest = line.trim_start().strip_prefix(name)?;
        let rest = rest.trim_start().strip_prefix("REG_SZ")?;
        Some(rest.trim().to_string())
    })
}

fn read_os_version() -> String {
    process::hidden_command("cmd")
        .args(["/c", "ver"])
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_matching_publisher_from_reg_query_output() {
        let output = "HKEY_LOCAL_MACHINE\\SOFTWARE\\...\\Uninstall\\SomeOtherApp\n    DisplayName    REG_SZ    Some Other App\n    DisplayVersion    REG_SZ    9.9\n    Publisher    REG_SZ    Someone Else\n\nHKEY_LOCAL_MACHINE\\SOFTWARE\\...\\Uninstall\\FlexASIO\n    DisplayName    REG_SZ    FlexASIO\n    DisplayVersion    REG_SZ    1.10b\n    Publisher    REG_SZ    Etienne Dechamps\n";

        assert_eq!(parse_reg_query_output(output).as_deref(), Some("1.10b"));
    }

    #[test]
    fn ignores_entries_from_other_publishers() {
        let output = "HKEY_LOCAL_MACHINE\\SOFTWARE\\...\\Uninstall\\SomeOtherApp\n    DisplayVersion    REG_SZ    9.9\n    Publisher    REG_SZ    Someone Else\n";

        assert_eq!(parse_reg_query_output(output), None);
    }

    #[test]
    fn field_order_within_a_block_does_not_matter() {
        let output = "HKEY_LOCAL_MACHINE\\SOFTWARE\\...\\Uninstall\\FlexASIO\n    Publisher    REG_SZ    Etienne Dechamps\n    DisplayVersion    REG_SZ    1.10b\n";

        assert_eq!(parse_reg_query_output(output).as_deref(), Some("1.10b"));
    }
}
