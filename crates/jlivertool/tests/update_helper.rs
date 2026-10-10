//! Exercise the real helper without touching an installed application.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn executable(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn exercise_helper(broken_update: bool, legacy_release: bool) {
    let directory = tempfile::Builder::new()
        .prefix("jl-helper-test-")
        .tempdir()
        .unwrap();
    let root = directory.path();
    let target = root.join("application with spaces");
    let payload = root.join("downloaded update");
    let script = |label: &str| {
        format!(
            "#!/bin/sh\nprintf '{label}:%s' \"$1\" > '{}'\ntouch '{}'\n",
            root.join("restarted").display(),
            root.join("ack").display()
        )
    };
    executable(&target, &script("old"));
    let new_script = if legacy_release {
        format!(
            "#!/bin/sh\nprintf 'new:%s' \"$1\" > '{}'\nexec sleep 5\n",
            root.join("restarted").display()
        )
    } else {
        script("new")
    };
    executable(
        &payload,
        if broken_update {
            "invalid executable"
        } else {
            &new_script
        },
    );
    let manifest = root.join("manifest.json");
    fs::write(&manifest, serde_json::to_vec(&serde_json::json!({
        "kind": "PortableBinary", "target": target, "payload": payload,
        "version": "9.0.0", "release_url": "https://github.com/Xinrea/JLiverTool/releases/tag/v9.0.0"
    })).unwrap()).unwrap();
    let mut helper = Command::new(env!("CARGO_BIN_EXE_jlivertool"))
        .arg("--apply-update")
        .arg(manifest)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !root.join("ready").exists() {
        assert!(
            helper.try_wait().unwrap().is_none(),
            "helper exited before ready"
        );
        assert!(Instant::now() < deadline, "helper did not become ready");
        std::thread::sleep(Duration::from_millis(25));
    }
    assert_eq!(fs::read_to_string(&target).unwrap(), script("old"));
    assert!(
        !root.join("restarted").exists(),
        "must wait for parent exit"
    );
    drop(helper.stdin.take()); // The parent's exit closes this pipe in production.
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = helper.try_wait().unwrap() {
            assert!(status.success(), "helper failed: {status}");
            break;
        }
        if Instant::now() >= deadline {
            let _ = helper.kill();
            let _ = helper.wait();
            panic!("helper did not install and restart");
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    let label = if broken_update { "old" } else { "new" };
    assert_eq!(
        fs::read_to_string(root.join("restarted")).unwrap(),
        format!("{label}:--update-result")
    );
    let result: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("result.json")).unwrap()).unwrap();
    assert_eq!(!result["error"].is_null(), broken_update);
}

#[test]
fn waits_for_exit_installs_and_restarts_new_application() {
    exercise_helper(false, false);
}

#[test]
fn broken_update_restarts_previous_application_with_error() {
    exercise_helper(true, false);
}

#[test]
fn accepts_releases_without_update_result_acknowledgement() {
    exercise_helper(false, true);
}
