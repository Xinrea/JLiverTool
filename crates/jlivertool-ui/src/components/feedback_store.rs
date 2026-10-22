//! Private, durable storage for a frozen feedback submission and its last receipt.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use jlivertool_core::feedback::{FeedbackDraft, FeedbackReceipt};
use serde::{de::DeserializeOwned, Serialize};

const DRAFT_FILE: &str = "feedback-draft.json";
const RECEIPT_FILE: &str = "feedback-receipt.json";
// JSON byte arrays can occupy four times the size of the two 5 MiB attachments.
const MAX_STORED_BYTES: u64 = 48 * 1024 * 1024;
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub struct FeedbackStore {
    directory: PathBuf,
}

impl FeedbackStore {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub fn load_draft(&self) -> Result<Option<FeedbackDraft>, String> {
        if !self.directory.exists() {
            return Ok(None);
        }
        self.ensure_private_directory()?;
        let draft: Option<FeedbackDraft> = self.read_json(&self.directory.join(DRAFT_FILE))?;
        if let Some(draft) = &draft {
            draft.validate().map_err(|error| error.to_string())?;
        }
        Ok(draft)
    }

    pub fn save_draft(&self, draft: &FeedbackDraft) -> Result<(), String> {
        draft.validate().map_err(|error| error.to_string())?;
        self.write_json(DRAFT_FILE, draft)
    }

    /// Persist the receipt before removing the draft, so a failed write remains retryable.
    pub fn complete(&self, receipt: &FeedbackReceipt) -> Result<(), String> {
        self.write_json(RECEIPT_FILE, receipt)?;
        match fs::remove_file(self.directory.join(DRAFT_FILE)) {
            Ok(()) => self.sync_directory(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("无法清理已提交的反馈草稿：{error}")),
        }
    }

    fn ensure_private_directory(&self) -> Result<(), String> {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder
            .create(&self.directory)
            .map_err(|error| format!("无法创建反馈存储目录：{error}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.directory, fs::Permissions::from_mode(0o700))
                .map_err(|error| format!("无法保护反馈存储目录：{error}"))?;
        }
        Ok(())
    }

    fn read_json<T: DeserializeOwned>(&self, path: &Path) -> Result<Option<T>, String> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("无法读取已保存的反馈：{error}")),
        };
        if file
            .metadata()
            .map_err(|error| format!("无法读取已保存的反馈：{error}"))?
            .len()
            > MAX_STORED_BYTES
        {
            return Err("已保存的反馈超出大小限制".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(0o600))
                .map_err(|error| format!("无法保护已保存的反馈：{error}"))?;
        }
        let mut bytes = Vec::new();
        file.take(MAX_STORED_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| format!("无法读取已保存的反馈：{error}"))?;
        if bytes.len() as u64 > MAX_STORED_BYTES {
            return Err("已保存的反馈超出大小限制".into());
        }
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|error| format!("无法解析已保存的反馈：{error}"))
    }

    fn write_json<T: Serialize>(&self, filename: &str, value: &T) -> Result<(), String> {
        let bytes = serde_json::to_vec(value).map_err(|error| format!("无法保存反馈：{error}"))?;
        if bytes.len() as u64 > MAX_STORED_BYTES {
            return Err("反馈草稿超出本地存储大小限制".into());
        }
        self.ensure_private_directory()?;

        let (temporary_path, mut file) = self.create_temporary_file(filename)?;
        let result = (|| {
            file.write_all(&bytes)
                .and_then(|()| file.sync_all())
                .map_err(|error| format!("无法写入反馈：{error}"))?;
            drop(file);
            fs::rename(&temporary_path, self.directory.join(filename))
                .map_err(|error| format!("无法保存反馈：{error}"))?;
            self.sync_directory()
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        result
    }

    fn create_temporary_file(&self, filename: &str) -> Result<(PathBuf, File), String> {
        for _ in 0..100 {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path = self
                .directory
                .join(format!(".{filename}.{}.{id}.tmp", std::process::id()));
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            match options.open(&path) {
                Ok(file) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Err(error) = file.set_permissions(fs::Permissions::from_mode(0o600))
                        {
                            drop(file);
                            let _ = fs::remove_file(&path);
                            return Err(format!("无法保护反馈临时文件：{error}"));
                        }
                    }
                    return Ok((path, file));
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("无法创建反馈临时文件：{error}")),
            }
        }
        Err("无法创建反馈临时文件，请清理反馈存储目录中的临时文件后重试".into())
    }

    fn sync_directory(&self) -> Result<(), String> {
        #[cfg(unix)]
        File::open(&self.directory)
            .and_then(|file| file.sync_all())
            .map_err(|error| format!("无法同步反馈存储目录：{error}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jlivertool_core::feedback::FeedbackAttachment;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            Self(std::env::temp_dir().join(format!(
                "jlivertool-feedback-store-test-{}-{now}-{id}",
                std::process::id()
            )))
        }

        fn store(&self) -> FeedbackStore {
            FeedbackStore::new(self.0.clone())
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn draft() -> FeedbackDraft {
        FeedbackDraft::new(
            "测试反馈内容",
            "test@example.com",
            "1.0.0",
            "test operating system",
            None,
            Some(FeedbackAttachment::log("test.log", b"frozen log snapshot\n".to_vec()).unwrap()),
        )
        .unwrap()
    }

    fn receipt() -> FeedbackReceipt {
        FeedbackReceipt {
            id: "a91c047b-78aa-4029-9e0e-bf8bca87e153".into(),
            message: "反馈已收到".into(),
        }
    }

    #[test]
    fn restart_retains_submission_id_and_frozen_attachment_bytes() {
        let directory = TestDirectory::new();
        assert!(directory.store().load_draft().unwrap().is_none());
        let original = draft();
        directory.store().save_draft(&original).unwrap();

        let restored = directory.store().load_draft().unwrap().unwrap();
        assert!(restored == original);
        assert_eq!(restored.submission_id(), original.submission_id());
        assert_eq!(restored.log().unwrap().bytes(), b"frozen log snapshot\n");

        directory.store().save_draft(&restored).unwrap();
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
    }

    #[test]
    fn successful_completion_saves_receipt_and_removes_pending_draft() {
        let directory = TestDirectory::new();
        directory.store().save_draft(&draft()).unwrap();
        directory.store().complete(&receipt()).unwrap();

        assert!(directory.store().load_draft().unwrap().is_none());
        let saved: FeedbackReceipt =
            serde_json::from_slice(&fs::read(directory.0.join(RECEIPT_FILE)).unwrap()).unwrap();
        assert_eq!(saved, receipt());
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 1);
    }

    #[test]
    fn failed_receipt_write_keeps_retryable_draft_and_cleans_temporary_file() {
        let directory = TestDirectory::new();
        let original = draft();
        directory.store().save_draft(&original).unwrap();
        fs::create_dir(directory.0.join(RECEIPT_FILE)).unwrap();

        assert!(directory.store().complete(&receipt()).is_err());
        assert!(directory.store().load_draft().unwrap().unwrap() == original);
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 2);
    }

    #[test]
    fn oversized_saved_file_is_rejected_without_removing_it() {
        let directory = TestDirectory::new();
        directory.store().ensure_private_directory().unwrap();
        let path = directory.0.join(DRAFT_FILE);
        File::create(&path)
            .unwrap()
            .set_len(MAX_STORED_BYTES + 1)
            .unwrap();

        assert!(directory.store().load_draft().is_err());
        assert!(path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn stored_feedback_is_accessible_only_to_its_owner() {
        use std::os::unix::fs::PermissionsExt;

        let directory = TestDirectory::new();
        directory.store().save_draft(&draft()).unwrap();
        assert_eq!(
            fs::metadata(&directory.0).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(directory.0.join(DRAFT_FILE))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        directory.store().complete(&receipt()).unwrap();
        assert_eq!(
            fs::metadata(directory.0.join(RECEIPT_FILE))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
