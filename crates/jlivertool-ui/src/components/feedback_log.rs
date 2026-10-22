use std::{
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use jlivertool_core::feedback::{FeedbackAttachment, MAX_FEEDBACK_FILE_BYTES};

/// Captures the latest app log as fixed attachment bytes for a new submission.
pub(super) fn capture_app_log(directory: &Path) -> Result<Option<FeedbackAttachment>, String> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("无法读取应用日志目录：{error}")),
    };
    let mut latest: Option<(String, PathBuf)> = None;
    let mut legacy = None;
    for entry in entries {
        let entry = entry.map_err(|error| format!("无法读取应用日志目录：{error}"))?;
        let filename = entry.file_name();
        let Some(filename) = filename.to_str() else {
            continue;
        };
        let date = app_log_date(filename);
        if date.is_none() && filename != "jlivertool.log" {
            continue;
        }
        if !entry
            .file_type()
            .map_err(|error| format!("无法读取应用日志信息：{error}"))?
            .is_file()
        {
            continue;
        }
        if let Some(date) = date {
            // Daily rolling log names use UTC dates, so filename order is more
            // reliable than modification times (including after a file copy).
            if latest
                .as_ref()
                .is_none_or(|(newest, _)| date > newest.as_str())
            {
                latest = Some((date.to_owned(), entry.path()));
            }
        } else {
            legacy = Some(entry.path());
        }
    }
    let Some(path) = latest.map(|(_, path)| path).or(legacy) else {
        return Ok(None);
    };
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("无法读取应用日志：{error}")),
    };
    let metadata = file
        .metadata()
        .map_err(|error| format!("无法读取应用日志信息：{error}"))?;
    if !metadata.is_file() {
        return Err("应用日志不是普通文件".into());
    }
    // Freeze the length before reading so concurrent appends are left for the
    // next submission, and retries retain exactly these attachment bytes.
    let bytes = read_log_snapshot(&mut file, metadata.len())
        .map_err(|error| format!("无法读取应用日志：{error}"))?;
    if bytes.is_empty() {
        return Ok(None);
    }
    FeedbackAttachment::log("jlivertool.log", bytes)
        .map(Some)
        .map_err(|error| format!("无法附加应用日志：{}", error.user_message()))
}

fn app_log_date(filename: &str) -> Option<&str> {
    let date = filename.strip_prefix("jlivertool.log.")?;
    let bytes = date.as_bytes();
    if bytes.len() == 10
        && bytes.iter().enumerate().all(|(index, byte)| {
            if index == 4 || index == 7 {
                *byte == b'-'
            } else {
                byte.is_ascii_digit()
            }
        })
        && chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok()
    {
        Some(date)
    } else {
        None
    }
}

fn read_log_snapshot(file: &mut File, snapshot_len: u64) -> io::Result<Vec<u8>> {
    let start = snapshot_len.saturating_sub(MAX_FEEDBACK_FILE_BYTES as u64);
    // Read one preceding byte to retain a complete first line when the size
    // boundary happens to fall directly after a newline.
    let read_start = start.saturating_sub(1);
    let read_len = snapshot_len - read_start;
    file.seek(SeekFrom::Start(read_start))?;
    let mut bytes = Vec::with_capacity(read_len as usize);
    file.take(read_len).read_to_end(&mut bytes)?;
    if start > 0 && !bytes.is_empty() {
        let mut discard = 1;
        if bytes[0] != b'\n' {
            if let Some(newline) = bytes[1..]
                .iter()
                .position(|byte| *byte == b'\n')
                .filter(|newline| newline + 2 < bytes.len())
            {
                discard = newline + 2;
            } else {
                // A single very long line can exceed the limit. Keep its tail
                // and remove only a code point split by the byte boundary.
                while discard < bytes.len() && discard < 4 && bytes[discard] & 0xc0 == 0x80 {
                    discard += 1;
                }
            }
        }
        bytes.drain(..discard);
    }
    // A writer can be partway through a UTF-8 character at the frozen end.
    // Interior encoding errors remain errors during attachment validation.
    if let Err(error) = std::str::from_utf8(&bytes) {
        if error.error_len().is_none() {
            bytes.truncate(error.valid_up_to());
        }
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Write,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "jlivertool-feedback-log-test-{}-{now}-{id}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn write(&self, name: &str, bytes: &[u8]) {
            fs::write(self.0.join(name), bytes).unwrap();
        }

        fn capture(&self) -> Option<FeedbackAttachment> {
            capture_app_log(&self.0).unwrap()
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn selects_latest_daily_app_log_and_ignores_other_files() {
        let directory = TestDirectory::new();
        directory.write("jlivertool.log.2026-10-23", b"latest app log\n");
        directory.write("jlivertool.log.2026-10-22", b"older app log\n");
        directory.write("jlivertool.log", b"legacy log written later\n");
        directory.write("other.log.2026-10-24", b"another application\n");
        directory.write("jlivertool.log.2026-99-99", b"invalid date\n");
        directory.write("jlivertool.log.2026-10-24.backup", b"backup\n");
        fs::create_dir(directory.0.join("jlivertool.log.2026-10-25")).unwrap();

        let attachment = directory.capture().unwrap();
        assert_eq!(attachment.filename(), "jlivertool.log");
        assert_eq!(attachment.bytes(), b"latest app log\n");
    }

    #[test]
    fn uses_legacy_app_log_when_no_daily_log_exists() {
        let directory = TestDirectory::new();
        directory.write("unrelated.log", b"unrelated\n");
        assert!(directory.capture().is_none());
        directory.write("jlivertool.log", b"legacy app log\n");
        assert_eq!(directory.capture().unwrap().bytes(), b"legacy app log\n");
    }

    #[test]
    fn absent_or_empty_app_logs_do_not_add_an_attachment() {
        let directory = TestDirectory::new();
        assert!(
            capture_app_log(&directory.0.join("missing"))
                .unwrap()
                .is_none()
        );
        assert!(directory.capture().is_none());
        directory.write("jlivertool.log.2026-10-22", b"old content\n");
        directory.write("jlivertool.log.2026-10-23", b"");
        assert!(directory.capture().is_none());
    }

    #[test]
    fn includes_recent_complete_lines_within_the_attachment_limit() {
        let directory = TestDirectory::new();
        let line = b"old line\n";
        let mut contents = line.repeat(MAX_FEEDBACK_FILE_BYTES / line.len() + 100);
        contents.extend_from_slice(b"latest line\n");
        directory.write("jlivertool.log", &contents);

        let attachment = directory.capture().unwrap();
        assert!(attachment.size() <= MAX_FEEDBACK_FILE_BYTES);
        assert!(attachment.size() > MAX_FEEDBACK_FILE_BYTES - line.len());
        assert!(attachment.bytes().starts_with(line));
        assert!(attachment.bytes().ends_with(b"latest line\n"));
    }

    #[test]
    fn long_line_tail_starts_on_a_utf8_character_boundary() {
        let directory = TestDirectory::new();
        let contents = format!("{}\n", "中".repeat(MAX_FEEDBACK_FILE_BYTES / 3 + 100));
        directory.write("jlivertool.log", contents.as_bytes());

        let attachment = directory.capture().unwrap();
        let text = std::str::from_utf8(attachment.bytes()).unwrap();
        assert_eq!(attachment.size(), MAX_FEEDBACK_FILE_BYTES - 1);
        assert!(text.ends_with('\n'));
        assert!(
            text.trim_end_matches('\n')
                .chars()
                .all(|character| character == '中')
        );
    }

    #[test]
    fn frozen_length_excludes_bytes_appended_before_the_read() {
        let directory = TestDirectory::new();
        directory.write("jlivertool.log", b"original log\n");
        let path = directory.0.join("jlivertool.log");
        let mut reader = File::open(&path).unwrap();
        let frozen_length = reader.metadata().unwrap().len();
        let mut writer = fs::OpenOptions::new().append(true).open(path).unwrap();
        writer.write_all(b"new log after snapshot\n").unwrap();

        assert_eq!(
            read_log_snapshot(&mut reader, frozen_length).unwrap(),
            b"original log\n"
        );
    }

    #[test]
    fn incomplete_utf8_at_the_frozen_end_is_omitted() {
        let directory = TestDirectory::new();
        directory.write("jlivertool.log", b"complete line\n\xe4\xb8");
        assert_eq!(directory.capture().unwrap().bytes(), b"complete line\n");
    }

    #[test]
    fn invalid_log_directory_returns_an_error() {
        let directory = TestDirectory::new();
        directory.write("ordinary-file", b"content");
        assert!(capture_app_log(&directory.0.join("ordinary-file")).is_err());
    }
}
