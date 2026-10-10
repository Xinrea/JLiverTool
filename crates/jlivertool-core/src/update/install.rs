//! Download a release asset, then hand installation to a separate process.
//! The helper keeps the old application available for rollback and only starts
//! replacing files after the UI process has exited (its stdin pipe reaches EOF).

use super::{fetch_latest_release, ReleaseAsset};
use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const APPLY_UPDATE_FLAG: &str = "--apply-update";
pub const UPDATE_RESULT_FLAG: &str = "--update-result";
static HELPER: Mutex<Option<Child>> = Mutex::new(None);

#[derive(Clone, Debug)]
pub enum InstallProgress {
    Downloading { downloaded: u64, total: u64 },
    Preparing,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
enum InstallKind {
    MacBundle,
    WindowsInstaller,
    PortableBinary,
    DebianPackage,
}

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    kind: InstallKind,
    target: PathBuf,
    payload: PathBuf,
    version: String,
    release_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallOutcome {
    pub version: String,
    pub release_url: String,
    pub error: Option<String>,
}

pub struct PreparedUpdate {
    directory: tempfile::TempDir,
}

fn current_target() -> Result<(InstallKind, PathBuf)> {
    let exe = std::env::current_exe()?;
    // Never replace a developer's cargo build with an installed release.
    ensure!(
        !exe.ancestors()
            .any(|p| p.file_name().is_some_and(|n| n == "target")),
        "开发构建不支持自动安装，请使用正式发布的应用"
    );
    #[cfg(target_os = "macos")]
    {
        let app = exe
            .ancestors()
            .find(|p| p.extension().is_some_and(|e| e == "app"))
            .context("请将 JLiverTool.app 安装到应用程序目录后再更新")?;
        ensure!(
            !app.to_string_lossy().contains("/AppTranslocation/") && !app.starts_with("/Volumes"),
            "请先将 JLiverTool.app 从磁盘映像移到应用程序目录并重新打开"
        );
        return Ok((InstallKind::MacBundle, app.to_path_buf()));
    }
    #[cfg(target_os = "windows")]
    {
        let kind = if exe
            .parent()
            .context("应用路径无效")?
            .join("uninstall.exe")
            .is_file()
        {
            InstallKind::WindowsInstaller
        } else {
            InstallKind::PortableBinary
        };
        return Ok((kind, exe));
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(appimage) = std::env::var_os("APPIMAGE") {
            return Ok((
                InstallKind::PortableBinary,
                std::fs::canonicalize(appimage)?,
            ));
        }
        let owner = Command::new("dpkg-query").args(["-S"]).arg(&exe).output();
        if owner.is_ok_and(|o| {
            o.status.success() && String::from_utf8_lossy(&o.stdout).starts_with("jlivertool:")
        }) {
            return Ok((InstallKind::DebianPackage, exe));
        }
        bail!("此安装方式不支持自动更新，请使用 AppImage 或 DEB 安装版本");
    }
    #[allow(unreachable_code)]
    Err(anyhow::anyhow!("此操作系统不支持自动安装"))
}

fn select_asset<'a>(
    assets: &'a [ReleaseAsset],
    kind: &InstallKind,
    arch: &str,
) -> Result<&'a ReleaseAsset> {
    let asset = assets
        .iter()
        .find(|asset| {
            let name = asset.name.to_ascii_lowercase();
            match kind {
                InstallKind::MacBundle => name.ends_with(&format!("_{arch}.dmg")),
                InstallKind::WindowsInstaller => {
                    arch == "x86_64" && name.ends_with("_x64-setup.exe")
                }
                InstallKind::PortableBinary if cfg!(target_os = "windows") => {
                    arch == "x86_64" && name == "jlivertool.exe"
                }
                InstallKind::PortableBinary => name.ends_with(&format!("_{arch}.appimage")),
                InstallKind::DebianPackage => {
                    let deb_arch = match arch {
                        "x86_64" => "amd64",
                        "aarch64" => "arm64",
                        other => other,
                    };
                    name.ends_with(&format!("_{deb_arch}.deb"))
                }
            }
        })
        .context("该版本没有适用于当前系统和处理器的安装包")?;
    // File names and URLs come from the authenticated GitHub TLS response.
    ensure!(
        Path::new(&asset.name).file_name().and_then(|n| n.to_str()) == Some(&asset.name)
            && !asset.name.contains(['/', '\\']),
        "安装包文件名无效"
    );
    ensure!(
        asset
            .browser_download_url
            .starts_with("https://github.com/Xinrea/JLiverTool/releases/download/"),
        "安装包地址无效"
    );
    expected_digest(asset)?;
    Ok(asset)
}

fn expected_digest(asset: &ReleaseAsset) -> Result<&str> {
    let digest = asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
        .context("安装包缺少 SHA-256 校验信息，请查看更新内容并手动安装")?;
    ensure!(
        digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()),
        "安装包校验信息无效"
    );
    Ok(digest)
}

fn verify_download(asset: &ReleaseAsset, bytes: u64, digest: &str) -> Result<()> {
    ensure!(bytes == asset.size, "安装包下载不完整，请重试");
    ensure!(
        digest.eq_ignore_ascii_case(expected_digest(asset)?),
        "安装包校验失败，请重试"
    );
    Ok(())
}

pub async fn prepare_update(
    version: &str,
    progress: impl Fn(InstallProgress),
) -> Result<PreparedUpdate> {
    let (kind, target) = current_target()?;
    if matches!(kind, InstallKind::MacBundle | InstallKind::PortableBinary) {
        // Fail while the app is still running if its installation is read-only.
        tempfile::NamedTempFile::new_in(target.parent().context("应用路径无效")?)
            .context("应用目录不可写，请调整目录权限后重试")?;
    }
    let release = fetch_latest_release().await?;
    ensure!(
        release.tag_name.trim_start_matches('v') == version,
        "最新版本已变化，请重新检查更新"
    );
    ensure!(
        super::compare_versions(env!("CARGO_PKG_VERSION"), version),
        "该版本不是更新版本，请重新检查更新"
    );
    let asset = select_asset(&release.assets, &kind, std::env::consts::ARCH)?;
    let directory = tempfile::Builder::new()
        .prefix("jlivertool-update-")
        .tempdir()?;
    let downloaded = directory.path().join(&asset.name);
    let client = reqwest::Client::builder()
        .user_agent("JLiverTool")
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(600))
        .build()?;
    let mut response = client
        .get(&asset.browser_download_url)
        .send()
        .await?
        .error_for_status()?;
    let mut file = File::create(&downloaded)?;
    let mut hasher = Sha256::new();
    let mut bytes = 0;
    let mut last_progress = Instant::now();
    progress(InstallProgress::Downloading {
        downloaded: 0,
        total: asset.size,
    });
    while let Some(chunk) = response.chunk().await? {
        bytes += chunk.len() as u64;
        ensure!(bytes <= asset.size, "安装包大小与发布信息不一致");
        file.write_all(&chunk)?;
        hasher.update(&chunk);
        if last_progress.elapsed() >= Duration::from_millis(200) {
            progress(InstallProgress::Downloading {
                downloaded: bytes,
                total: asset.size,
            });
            last_progress = Instant::now();
        }
    }
    file.sync_all()?;
    drop(file);
    verify_download(asset, bytes, &format!("{:x}", hasher.finalize()))?;
    progress(InstallProgress::Preparing);
    let payload = if kind == InstallKind::MacBundle {
        unpack_dmg(&downloaded, directory.path())?
    } else {
        downloaded
    };
    #[cfg(unix)]
    if kind == InstallKind::PortableBinary {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&payload, std::fs::Permissions::from_mode(0o755))?;
    }
    let manifest = Manifest {
        kind,
        target,
        payload,
        version: version.into(),
        release_url: release.html_url,
    };
    std::fs::write(
        directory.path().join("manifest.json"),
        serde_json::to_vec(&manifest)?,
    )?;
    Ok(PreparedUpdate { directory })
}

fn checked(command: &mut Command) -> Result<()> {
    let output = command
        .output()
        .with_context(|| format!("无法启动 {:?}", command.get_program()))?;
    ensure!(
        output.status.success(),
        "命令执行失败 ({:?}, {}): {} {}",
        command.get_program(),
        output.status,
        String::from_utf8_lossy(&output.stderr).trim(),
        String::from_utf8_lossy(&output.stdout).trim()
    );
    Ok(())
}

fn unpack_dmg(dmg: &Path, directory: &Path) -> Result<PathBuf> {
    let mount = directory.join("mount");
    std::fs::create_dir(&mount)?;
    checked(
        Command::new("/usr/bin/hdiutil")
            .args(["attach", "-nobrowse", "-readonly", "-mountpoint"])
            .arg(&mount)
            .arg(dmg),
    )?;
    let result = (|| {
        let app = std::fs::read_dir(&mount)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|e| e == "app"))
            .context("磁盘映像中没有应用程序")?;
        ensure!(
            app.join("Contents/MacOS/jlivertool").is_file(),
            "更新包中缺少 JLiverTool 程序"
        );
        let staged = directory.join("JLiverTool.app");
        checked(Command::new("/usr/bin/ditto").arg(&app).arg(&staged))?;
        Ok(staged)
    })();
    let detached = checked(Command::new("/usr/bin/hdiutil").arg("detach").arg(&mount));
    match (result, detached) {
        (Ok(path), Ok(())) => Ok(path),
        (Err(e), _) | (_, Err(e)) => Err(e),
    }
}

impl PreparedUpdate {
    /// Confirm the copied helper can start before telling the UI to quit.
    pub fn launch(self) -> Result<()> {
        let helper = self.directory.path().join(if cfg!(windows) {
            "update-helper.exe"
        } else {
            "update-helper"
        });
        std::fs::copy(std::env::current_exe()?, &helper)?;
        let mut command = Command::new(&helper);
        command
            .arg(APPLY_UPDATE_FLAG)
            .arg(self.directory.path().join("manifest.json"))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        let mut child = command.spawn().context("无法启动更新安装程序")?;
        let ready = self.directory.path().join("ready");
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready.exists() {
            if child.try_wait()?.is_some() || Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                bail!("更新安装程序启动失败，请重试");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        // Retain stdin until the parent process exits, releasing the helper.
        *HELPER
            .lock()
            .map_err(|_| anyhow::anyhow!("更新程序状态无效"))? = Some(child);
        let _ = self.directory.keep();
        Ok(())
    }
}

/// Called before logging/GPUI initialization in the copied helper process.
pub fn apply_update(manifest_path: &Path) -> Result<()> {
    let manifest: Manifest = serde_json::from_slice(&std::fs::read(manifest_path)?)?;
    let directory = manifest_path.parent().context("更新目录无效")?;
    std::fs::write(directory.join("ready"), b"ready")?;
    std::io::stdin().read_to_end(&mut Vec::new())?;
    let result = install(&manifest);
    let mut outcome = InstallOutcome {
        version: manifest.version.clone(),
        release_url: manifest.release_url.clone(),
        error: result.err().map(|e| format!("{e:#}")),
    };
    let outcome_path = directory.join("result.json");
    std::fs::write(&outcome_path, serde_json::to_vec(&outcome)?)?;
    if let Err(error) = restart(&manifest.target, &manifest.kind, &outcome_path) {
        if matches!(
            manifest.kind,
            InstallKind::MacBundle | InstallKind::PortableBinary
        ) {
            restore_backup(directory, &manifest.target)?;
        }
        outcome.error = Some(format!("更新后启动失败：{error:#}"));
        std::fs::write(&outcome_path, serde_json::to_vec(&outcome)?)?;
        restart(&manifest.target, &manifest.kind, &outcome_path)?;
    }
    cleanup(directory);
    Ok(())
}

fn install(manifest: &Manifest) -> Result<()> {
    match manifest.kind {
        InstallKind::PortableBinary | InstallKind::MacBundle => replace_application(manifest),
        InstallKind::WindowsInstaller => install_windows(manifest),
        InstallKind::DebianPackage => checked(
            Command::new("pkexec")
                .args(["/usr/bin/apt-get", "install", "--yes", "--no-remove"])
                .arg(&manifest.payload),
        )
        .context("安装 DEB 更新失败"),
    }
}

fn replace_application(manifest: &Manifest) -> Result<()> {
    let parent = manifest.target.parent().context("应用目录无效")?;
    let staging = tempfile::Builder::new()
        .prefix(".jlivertool-update-")
        .tempdir_in(parent)?;
    let staged = staging.path().join("new");
    if manifest.kind == InstallKind::MacBundle {
        checked(
            Command::new("/usr/bin/ditto")
                .arg(&manifest.payload)
                .arg(&staged),
        )?;
    } else {
        std::fs::copy(&manifest.payload, &staged)?;
    }
    let backup = staging.path().join("previous");
    // Write recovery information before changing the installed application.
    std::fs::write(
        manifest
            .payload
            .parent()
            .context("更新目录无效")?
            .join("backup-path"),
        staging.path().to_string_lossy().as_bytes(),
    )?;
    // Once an old application may enter this directory, never delete it through
    // TempDir's destructor on an error. Keep recovery files until a restart ack.
    let _ = staging.keep();
    rename_with_retry(&manifest.target, &backup).context("无法备份当前应用")?;
    if let Err(error) = std::fs::rename(&staged, &manifest.target) {
        std::fs::rename(&backup, &manifest.target).context("恢复旧版本失败")?;
        return Err(error).context("无法替换应用，已恢复旧版本");
    }
    Ok(())
}

fn rename_with_retry(source: &Path, destination: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        // EOF can precede Windows releasing the exiting process's image lock.
        for _ in 0..20 {
            match std::fs::rename(source, destination) {
                Ok(()) => return Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(error) => return Err(error),
            }
        }
    }
    std::fs::rename(source, destination)
}

fn restore_backup(directory: &Path, target: &Path) -> Result<()> {
    let backup_directory = PathBuf::from(std::fs::read_to_string(directory.join("backup-path"))?);
    let backup = backup_directory.join("previous");
    let failed = backup_directory.join("failed");
    let had_target = target.exists();
    if had_target {
        rename_with_retry(target, &failed)?;
    }
    if let Err(error) = std::fs::rename(&backup, target) {
        if had_target {
            let _ = std::fs::rename(&failed, target);
        }
        return Err(error).context("恢复旧版本失败");
    }
    Ok(())
}

fn install_windows(manifest: &Manifest) -> Result<()> {
    // NSIS's /D override must be last and unquoted, including paths with spaces.
    // ShellExecute through Start-Process permits the installer's UAC prompt.
    let quote = |p: &Path| -> Result<String> {
        let value = p.to_str().context("应用路径编码无效")?;
        ensure!(!value.contains(['\r', '\n']), "应用路径无效");
        Ok(format!("'{}'", value.replace('\'', "''")))
    };
    let parent = manifest.target.parent().context("应用目录无效")?;
    quote(parent)?;
    let arguments = format!("/S /NS /D={}", parent.display());
    let script = format!(
        "$ErrorActionPreference='Stop'; $p=Start-Process -FilePath {} -ArgumentList '{}' -Wait -PassThru; exit $p.ExitCode",
        quote(&manifest.payload)?, arguments.replace('\'', "''")
    );
    checked(Command::new("powershell.exe").args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        &script,
    ]))
    .context("安装更新失败或已取消授权")
}

fn restart(target: &Path, kind: &InstallKind, outcome: &Path) -> Result<()> {
    // Start the bundle's executable directly so startup failure can be observed.
    // NSBundle still identifies the containing .app when launched this way.
    let executable = if *kind == InstallKind::MacBundle {
        target.join("Contents/MacOS/jlivertool")
    } else {
        target.to_path_buf()
    };
    let mut command = Command::new(executable);
    command
        .arg(UPDATE_RESULT_FLAG)
        .arg(outcome)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if cfg!(target_os = "linux") {
        command
            .env_remove("APPIMAGE")
            .env_remove("APPDIR")
            .env_remove("OWD");
    }
    let mut child = command.spawn().context("无法重新启动应用")?;
    let ack = outcome.parent().context("更新目录无效")?.join("ack");
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if ack.is_file() {
            return Ok(());
        }
        ensure!(child.try_wait()?.is_none(), "应用在完成启动前退出");
        if Instant::now() >= deadline {
            // Releases predating the updater ignore --update-result. An app
            // still running after the startup grace period is accepted too.
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Acknowledge that the new process can run; the helper then removes its backup.
pub fn read_outcome(path: &Path) -> Result<InstallOutcome> {
    let outcome = serde_json::from_slice(&std::fs::read(path)?)?;
    std::fs::write(
        path.parent().context("更新目录无效")?.join("ack"),
        b"started",
    )?;
    Ok(outcome)
}

fn cleanup(directory: &Path) {
    // Only clean directories produced by this updater.
    if let Some(directory) = Some(directory).filter(|p| {
        p.file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("jlivertool-update-"))
    }) {
        if let Ok(backup) = std::fs::read_to_string(directory.join("backup-path")) {
            let backup = Path::new(&backup);
            if backup
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with(".jlivertool-update-"))
            {
                let _ = std::fs::remove_dir_all(backup);
            }
        }
        // On Windows the helper may still be running; leave its file for the OS
        // temp cleanup if it is locked, but remove the downloaded payload.
        let _ = std::fs::remove_dir_all(directory);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(name: &str, body: &[u8]) -> ReleaseAsset {
        ReleaseAsset {
            name: name.into(),
            size: body.len() as u64,
            browser_download_url: format!(
                "https://github.com/Xinrea/JLiverTool/releases/download/v9.0.0/{name}"
            ),
            digest: Some(format!("sha256:{:x}", Sha256::digest(body))),
        }
    }

    #[test]
    fn selects_installer_for_architecture_and_distribution() {
        let assets = vec![
            asset("jlivertool.exe", b"exe"),
            asset("JLiverTool_9.0.0_aarch64.dmg", b"dmg"),
            asset("jlivertool_9.0.0_x64-setup.exe", b"nsis"),
            asset("jlivertool_9.0.0_amd64.deb", b"deb"),
        ];
        assert_eq!(
            select_asset(&assets, &InstallKind::WindowsInstaller, "x86_64")
                .unwrap()
                .name,
            "jlivertool_9.0.0_x64-setup.exe"
        );
        assert!(select_asset(&assets, &InstallKind::WindowsInstaller, "aarch64").is_err());
        assert!(select_asset(&assets, &InstallKind::MacBundle, "x86_64").is_err());
        assert_eq!(
            select_asset(&assets, &InstallKind::MacBundle, "aarch64")
                .unwrap()
                .name,
            "JLiverTool_9.0.0_aarch64.dmg"
        );
        assert!(select_asset(&assets, &InstallKind::DebianPackage, "x86_64").is_ok());
    }

    #[test]
    fn rejects_tampered_incomplete_or_unverified_downloads() {
        let mut asset = asset("JLiverTool_9.0.0_aarch64.dmg", b"original");
        let digest = format!("{:x}", Sha256::digest(b"original"));
        assert!(verify_download(&asset, 8, &digest).is_ok());
        assert!(verify_download(&asset, 7, &digest).is_err());
        assert!(verify_download(&asset, 8, &format!("{:x}", Sha256::digest(b"tampered"))).is_err());
        asset.digest = None;
        assert!(select_asset(&[asset], &InstallKind::MacBundle, "aarch64").is_err());
    }

    #[test]
    fn rejects_release_paths_outside_expected_repository() {
        let mut asset = asset("JLiverTool_9.0.0_aarch64.dmg", b"dmg");
        asset.browser_download_url = "https://example.com/update.dmg".into();
        assert!(select_asset(&[asset], &InstallKind::MacBundle, "aarch64").is_err());
    }

    #[test]
    fn replacement_keeps_old_binary_for_rollback() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("application with spaces");
        let payload = directory.path().join("download");
        std::fs::write(&target, b"old").unwrap();
        std::fs::write(&payload, b"new").unwrap();
        let manifest = Manifest {
            kind: InstallKind::PortableBinary,
            target: target.clone(),
            payload,
            version: "9.0.0".into(),
            release_url: String::new(),
        };
        replace_application(&manifest).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"new");
        let backup = std::fs::read_to_string(directory.path().join("backup-path")).unwrap();
        assert_eq!(
            std::fs::read(Path::new(&backup).join("previous")).unwrap(),
            b"old"
        );
        restore_backup(directory.path(), &target).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"old");
    }

    #[test]
    fn missing_payload_leaves_installed_binary_untouched() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("application");
        std::fs::write(&target, b"old").unwrap();
        let manifest = Manifest {
            kind: InstallKind::PortableBinary,
            target: target.clone(),
            payload: directory.path().join("missing"),
            version: "9.0.0".into(),
            release_url: String::new(),
        };
        assert!(replace_application(&manifest).is_err());
        assert_eq!(std::fs::read(target).unwrap(), b"old");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn extracts_macos_application_and_detaches_disk_image() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source");
        let binary = source.join("JLiverTool.app/Contents/MacOS/jlivertool");
        std::fs::create_dir_all(binary.parent().unwrap()).unwrap();
        std::fs::write(&binary, b"test application").unwrap();
        let dmg = directory.path().join("update.dmg");
        checked(
            Command::new("/usr/bin/hdiutil")
                .args(["create", "-quiet", "-srcfolder"])
                .arg(&source)
                .arg(&dmg),
        )
        .unwrap();
        let extracted = unpack_dmg(&dmg, directory.path()).unwrap();
        assert_eq!(
            std::fs::read(extracted.join("Contents/MacOS/jlivertool")).unwrap(),
            b"test application"
        );
        assert_eq!(
            std::fs::read_dir(directory.path().join("mount"))
                .unwrap()
                .count(),
            0,
            "disk image must be detached"
        );
    }
}
