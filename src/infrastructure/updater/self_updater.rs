use std::path::{Path, PathBuf};

pub struct SelfUpdateService;

impl SelfUpdateService {
    pub async fn download_update(url: &str) -> Result<PathBuf, String> {
        let client = reqwest::Client::builder()
            .user_agent("benchmark-llm-gui-updater/1.0")
            .timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_default();

        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Network error during download: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Download failed (HTTP {})", resp.status()));
        }

        let bytes = resp.bytes().await.map_err(|e| format!("Failed to read release asset: {e}"))?;
        if bytes.is_empty() {
            return Err("Downloaded release asset is empty.".to_string());
        }

        let pid = std::process::id();
        let temp_dir = std::env::temp_dir();
        let lower_url = url.to_lowercase();
        let is_zip = lower_url.ends_with(".zip") || bytes.starts_with(b"PK\x03\x04");
        let is_targz = lower_url.ends_with(".tar.gz") || lower_url.ends_with(".tgz") || bytes.starts_with(&[0x1f, 0x8b]);

        if is_zip {
            Self::extract_zip_binary(&temp_dir, pid, &bytes)
        } else if is_targz {
            Self::extract_targz_binary(&temp_dir, pid, &bytes)
        } else {
            let dest = temp_dir.join(format!("benchmark_update_{pid}.exe"));
            std::fs::write(&dest, &bytes).map_err(|e| format!("Failed to write binary: {e}"))?;
            Self::verify_executable_magic(&dest)?;
            Ok(dest)
        }
    }

    fn extract_zip_binary(temp_dir: &Path, pid: u32, bytes: &[u8]) -> Result<PathBuf, String> {
        let zip_path = temp_dir.join(format!("benchmark_update_{pid}.zip"));
        let unpack_dir = temp_dir.join(format!("benchmark_unpack_{pid}"));
        let _ = std::fs::remove_dir_all(&unpack_dir);
        std::fs::create_dir_all(&unpack_dir).map_err(|e| format!("Failed to create temp unpack dir: {e}"))?;
        std::fs::write(&zip_path, bytes).map_err(|e| format!("Failed to write temporary archive: {e}"))?;

        #[cfg(target_os = "windows")]
        let extract_ok = {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let script = format!(
                "Add-Type -AssemblyName System.IO.Compression.FileSystem; [System.IO.Compression.ZipFile]::ExtractToDirectory('{}', '{}')",
                zip_path.display().to_string().replace('\'', "''"),
                unpack_dir.display().to_string().replace('\'', "''")
            );
            std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", &script])
                .creation_flags(CREATE_NO_WINDOW)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        };
        #[cfg(not(target_os = "windows"))]
        let extract_ok = std::process::Command::new("unzip")
            .args(["-o", &zip_path.to_string_lossy(), "-d", &unpack_dir.to_string_lossy()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        let _ = std::fs::remove_file(&zip_path);
        if !extract_ok {
            let _ = std::fs::remove_dir_all(&unpack_dir);
            return Err("Failed to extract update ZIP archive.".to_string());
        }

        let extracted = Self::find_executable_in_dir(&unpack_dir)
            .ok_or_else(|| "No executable binary found inside release archive.".to_string())?;
        let final_bin = temp_dir.join(format!("benchmark_update_{pid}.exe"));
        std::fs::copy(&extracted, &final_bin).map_err(|e| format!("Failed to stage extracted binary: {e}"))?;
        let _ = std::fs::remove_dir_all(&unpack_dir);
        Self::verify_executable_magic(&final_bin)?;
        Ok(final_bin)
    }

    fn extract_targz_binary(temp_dir: &Path, pid: u32, bytes: &[u8]) -> Result<PathBuf, String> {
        let tgz_path = temp_dir.join(format!("benchmark_update_{pid}.tar.gz"));
        let unpack_dir = temp_dir.join(format!("benchmark_unpack_{pid}"));
        let _ = std::fs::remove_dir_all(&unpack_dir);
        std::fs::create_dir_all(&unpack_dir).map_err(|e| format!("Failed to create temp unpack dir: {e}"))?;
        std::fs::write(&tgz_path, bytes).map_err(|e| format!("Failed to write temporary archive: {e}"))?;

        let ok = std::process::Command::new("tar")
            .args(["-xzf", &tgz_path.to_string_lossy(), "-C", &unpack_dir.to_string_lossy()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        let _ = std::fs::remove_file(&tgz_path);
        if !ok {
            let _ = std::fs::remove_dir_all(&unpack_dir);
            return Err("Failed to extract release tar.gz archive.".to_string());
        }

        let extracted = Self::find_executable_in_dir(&unpack_dir)
            .ok_or_else(|| "No executable binary found inside tar.gz archive.".to_string())?;
        let final_bin = temp_dir.join(format!("benchmark_update_{pid}.bin"));
        std::fs::copy(&extracted, &final_bin).map_err(|e| format!("Failed to stage extracted binary: {e}"))?;
        let _ = std::fs::remove_dir_all(&unpack_dir);
        Self::verify_executable_magic(&final_bin)?;
        Ok(final_bin)
    }

    fn find_executable_in_dir(dir: &Path) -> Option<PathBuf> {
        let entries = std::fs::read_dir(dir).ok()?;
        let mut fallback = None;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = Self::find_executable_in_dir(&path) {
                    return Some(found);
                }
            } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let low = name.to_lowercase();
                if low.ends_with(".exe") || low == "benchmark-llm-gui" {
                    return Some(path);
                }
                fallback = Some(path);
            }
        }
        fallback
    }

    fn verify_executable_magic(path: &Path) -> Result<(), String> {
        let head = std::fs::read(path).map_err(|e| format!("Failed to verify downloaded binary: {e}"))?;
        if head.len() < 2 {
            return Err("Downloaded binary is corrupt or incomplete.".to_string());
        }
        if head.starts_with(b"<html") || head.starts_with(b"<!DOCTYPE") || head.starts_with(b"<!doctype") {
            return Err("Downloaded asset returned HTML instead of a binary executable.".to_string());
        }
        #[cfg(target_os = "windows")]
        if !head.starts_with(b"MZ") {
            return Err("Downloaded Windows asset is not a valid PE executable (missing MZ header).".to_string());
        }
        Ok(())
    }

    pub fn apply_and_restart(temp_new_binary: &Path) -> Result<(), String> {
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Unable to resolve current executable path: {e}"))?;
        let old_exe = current_exe.with_extension("exe.old");
        let _ = std::fs::remove_file(&old_exe);

        std::fs::rename(&current_exe, &old_exe)
            .map_err(|e| format!("Unable to rename active executable: {e}"))?;

        if let Err(e) = std::fs::copy(temp_new_binary, &current_exe) {
            let _ = std::fs::rename(&old_exe, &current_exe);
            return Err(format!("Failed to install updated binary: {e}"));
        }
        let _ = std::fs::remove_file(temp_new_binary);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&current_exe, std::fs::Permissions::from_mode(0o755));
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            if let Some(old_str) = old_exe.to_str() {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "ping", "127.0.0.1", "-n", "3", ">", "nul", "&", "del", "/F", "/Q", old_str])
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn();
            }
        }

        std::process::Command::new(&current_exe)
            .spawn()
            .map_err(|e| format!("Failed to restart updated application: {e}"))?;

        std::process::exit(0);
    }
}
