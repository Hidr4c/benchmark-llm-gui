use std::path::{Path, PathBuf};

pub struct SelfUpdateService;

impl SelfUpdateService {
    pub async fn download_update(url: &str) -> Result<PathBuf, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_default();

        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Erreur reseau lors du telechargement: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Echec du telechargement (HTTP {})", resp.status()));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("Erreur de lecture du binaire: {e}"))?;

        if bytes.is_empty() {
            return Err("Le fichier binaire telecharge est vide.".to_string());
        }

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("benchmark_update_{}.exe", std::process::id()));
        std::fs::write(&temp_file, &bytes)
            .map_err(|e| format!("Echec d'ecriture du fichier temporaire: {e}"))?;

        Ok(temp_file)
    }

    pub fn apply_and_restart(temp_new_binary: &Path) -> Result<(), String> {
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Impossible de resoudre le chemin de l'application: {e}"))?;

        let old_exe = current_exe.with_extension("exe.old");

        // 1. Remove previous .old if exists
        let _ = std::fs::remove_file(&old_exe);

        // 2. Rename running exe to .old (NTFS allows renaming open running binaries)
        std::fs::rename(&current_exe, &old_exe)
            .map_err(|e| format!("Impossible de renommer l'executable actif: {e}"))?;

        // 3. Move new downloaded binary to current_exe location
        if let Err(e) = std::fs::copy(temp_new_binary, &current_exe) {
            // Rollback on failure
            let _ = std::fs::rename(&old_exe, &current_exe);
            return Err(format!("Echec de copie du nouveau binaire: {e}"));
        }

        // Cleanup temporary download
        let _ = std::fs::remove_file(temp_new_binary);

        // 4. Executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&current_exe, std::fs::Permissions::from_mode(0o755));
        }

        // 5. On Windows, spawn detached silent cmd to delete .exe.old after process termination
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

        // 6. Spawn new updated process
        std::process::Command::new(&current_exe)
            .spawn()
            .map_err(|e| format!("Echec du redemarrage de la nouvelle version: {e}"))?;

        // 7. Clean exit
        std::process::exit(0);
    }
}
