use crate::error::AppError;
use crate::install;
use crate::process;
use regex::Regex;
use std::sync::LazyLock;
use tauri::AppHandle;

/// An audio device as reported by PortAudioDevices.exe for one host API/backend.
#[derive(serde::Serialize, Debug, PartialEq)]
pub struct AudioDevice {
    pub name: String,
    pub host_api: String,
    pub max_input_channels: u32,
    pub max_output_channels: u32,
}

pub fn list(app: &AppHandle) -> Result<Vec<AudioDevice>, AppError> {
    let exe = install::find_tool(app, "PortAudioDevices.exe").ok_or(AppError::FlexAsioNotFound)?;

    let output = process::hidden_command(exe)
        .output()
        .map_err(|e| AppError::ProcessFailed(e.to_string()))?;
    if !output.status.success() {
        return Err(AppError::ProcessFailed(format!(
            "PortAudioDevices.exe exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_all(&stdout))
}

fn parse_all(output: &str) -> Vec<AudioDevice> {
    let output = output.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    output.split("\n\n").filter_map(parse_one).collect()
}

fn parse_one(block: &str) -> Option<AudioDevice> {
    static NAME_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"Device name: "([^"]+)""#).unwrap());
    static HOST_API_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Host API name: (.+)").unwrap());
    static INPUT_CHANNELS_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"Input: max channel count (\d+)").unwrap());
    static OUTPUT_CHANNELS_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"Output: max channel count (\d+)").unwrap());

    Some(AudioDevice {
        name: NAME_RE.captures(block)?[1].to_string(),
        host_api: HOST_API_RE.captures(block)?[1].to_string(),
        max_input_channels: INPUT_CHANNELS_RE
            .captures(block)
            .and_then(|c| c[1].parse().ok())
            .unwrap_or(0),
        max_output_channels: OUTPUT_CHANNELS_RE
            .captures(block)
            .and_then(|c| c[1].parse().ok())
            .unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_portaudio_devices_output() {
        let fixture = include_str!("../tests/fixtures/portaudio_devices_output.txt");

        let devices = parse_all(fixture);

        assert_eq!(devices.len(), 38);
        let first = &devices[0];
        assert_eq!(first.name, "Microsoft Soundmapper - Input");
        assert_eq!(first.host_api, "MME");
        assert_eq!(first.max_input_channels, 2);
        assert_eq!(first.max_output_channels, 0);
        let wasapi_device = devices
            .iter()
            .find(|d| d.host_api == "Windows WASAPI" && d.name.contains("T24i-30"))
            .expect("WASAPI device should be found");
        assert_eq!(
            wasapi_device.name,
            "1 - T24i-30 (AMD High Definition Audio Device)"
        );
    }

    #[test]
    fn parses_single_minimal_block() {
        let input = "Device index: 0\nDevice name: \"Test\"\nInput: max channel count 2, default latency 0s (low) 0s (high)\nOutput: max channel count 0, default latency 0s (low) 0s (high)\nHost API name: MME\n";

        let devices = parse_all(input);

        assert_eq!(
            devices,
            vec![AudioDevice {
                name: "Test".to_string(),
                host_api: "MME".to_string(),
                max_input_channels: 2,
                max_output_channels: 0,
            }]
        );
    }

    #[test]
    fn ignores_incomplete_block_without_name() {
        let input = "Device index: 0\nHost API name: MME\n";

        let devices = parse_all(input);

        assert_eq!(devices, vec![]);
    }

    #[test]
    fn handles_crlf_line_endings() {
        let input = "Device index: 0\r\nDevice name: \"Test\"\r\nInput: max channel count 1, default latency 0s (low) 0s (high)\r\nOutput: max channel count 0, default latency 0s (low) 0s (high)\r\nHost API name: MME\r\n";

        let devices = parse_all(input);

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Test");
    }
}
