//! Best-effort, bounded diagnostic logs. Plugin settings are never persisted.
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::mpsc::{self, SyncSender},
};

const LIMIT: u64 = 2 * 1024 * 1024;
const ARCHIVES: usize = 3;
const LINE_LIMIT: usize = 1024;

pub fn directory(config: &Path) -> PathBuf {
    config.with_file_name("logs")
}

fn rotate(path: &Path, extra: u64) -> std::io::Result<()> {
    if path
        .metadata()
        .is_ok_and(|m| m.len().saturating_add(extra) > LIMIT)
    {
        let oldest = path.with_extension(format!("log.{ARCHIVES}"));
        let _ = fs::remove_file(oldest);
        for n in (1..ARCHIVES).rev() {
            let _ = fs::rename(
                path.with_extension(format!("log.{n}")),
                path.with_extension(format!("log.{}", n + 1)),
            );
        }
        fs::rename(path, path.with_extension("log.1"))?;
    }
    Ok(())
}

fn append(path: &Path, line: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    rotate(path, line.len() as u64 + 1)?;
    writeln!(
        OpenOptions::new().create(true).append(true).open(path)?,
        "{line}"
    )
}

/// Remove recognizable credentials without discarding the surrounding diagnostic text.
/// The caller must never pass raw settings objects to this best-effort filter.
pub fn redact(message: &str) -> String {
    let input: String = message
        .chars()
        .take(LINE_LIMIT)
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let mut out = String::new();
    let mut i = 0;
    while i < input.len() {
        let rest = &input[i..];
        let lower = rest.to_ascii_lowercase();
        let key = [
            "authorization",
            "bearer",
            "password",
            "passwd",
            "token",
            "secret",
            "api_key",
            "apikey",
            "access_key",
        ]
        .into_iter()
        .find(|key| {
            lower.starts_with(key)
                && (i == 0 || !input.as_bytes()[i - 1].is_ascii_alphanumeric())
                && lower
                    .as_bytes()
                    .get(key.len())
                    .is_none_or(|c| !c.is_ascii_alphanumeric() && *c != b'_')
        });
        if let Some(key) = key {
            out.push_str(&rest[..key.len()]);
            i += key.len();
            while i < input.len()
                && matches!(
                    input.as_bytes()[i],
                    b' ' | b'\t' | b'"' | b'\'' | b':' | b'='
                )
            {
                out.push(input.as_bytes()[i] as char);
                i += 1;
            }
            let start = i;
            if key == "authorization" && input[i..].to_ascii_lowercase().starts_with("bearer ") {
                i += "bearer ".len();
            }
            while i < input.len()
                && !matches!(
                    input.as_bytes()[i],
                    b' ' | b'\t' | b',' | b';' | b'"' | b'\'' | b'}' | b']'
                )
            {
                i += 1;
            }
            if i > start {
                out.push_str("[masqué]");
            }
            continue;
        }
        let end = rest
            .find(|c: char| {
                c.is_whitespace()
                    || matches!(
                        c,
                        ',' | ';' | '"' | '\'' | '{' | '}' | '[' | ']' | '(' | ')'
                    )
            })
            .unwrap_or(rest.len());
        if end > 0 {
            let word = &rest[..end];
            let hex = word.len() >= 32 && word.bytes().all(|b| b.is_ascii_hexdigit());
            let jwt = word.split('.').count() == 3
                && word.len() >= 32
                && word
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'));
            if hex || jwt {
                out.push_str("[masqué]");
            } else {
                out.push_str(word);
            }
            i += end;
        } else {
            let c = rest.chars().next().unwrap();
            out.push(c);
            i += c.len_utf8();
        }
    }
    out
}

/// Raw plugin settings are not passed to this function; only the UI's diagnostic line.
pub fn safe_event(message: &str) -> String {
    if message.contains("setSettings") {
        "plugin setSettings (détails masqués)".into()
    } else {
        redact(message)
    }
}

#[cfg(windows)]
fn local_stamp() -> String {
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;
    let mut t = std::mem::MaybeUninit::uninit();
    unsafe {
        GetLocalTime(t.as_mut_ptr());
        let t: windows_sys::Win32::Foundation::SYSTEMTIME = t.assume_init();
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond
        )
    }
}

#[cfg(not(windows))]
fn local_stamp() -> String {
    // Windows is the supported runtime.
    format!("{:?}", std::time::SystemTime::now())
}

pub struct Journal {
    tx: SyncSender<String>,
}
impl Journal {
    pub fn start(config: &Path) -> Self {
        let path = directory(config).join("novydeck.log");
        let (tx, rx) = mpsc::sync_channel::<String>(1024);
        std::thread::spawn(move || {
            while let Ok(line) = rx.recv() {
                let _ = append(&path, &line);
            }
        });
        Self { tx }
    }

    pub fn record(&self, elapsed: f32, message: &str) {
        self.event(elapsed, &safe_event(message));
    }
    pub fn plugin_settings(&self, elapsed: f32, uuid: &str) {
        self.event(
            elapsed,
            &format!(
                "plugin setSettings {} : paramètres modifiés",
                safe_uuid(uuid)
            ),
        );
    }
    pub fn event(&self, elapsed: f32, event: &str) {
        let _ = self.tx.try_send(format!(
            "{}  {:>8.1}s  {}",
            local_stamp(),
            elapsed,
            redact(event)
        ));
    }
}

fn safe_uuid(uuid: &str) -> String {
    uuid.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(100)
        .collect()
}

/// Consume stderr in bounded chunks, retaining sanitized lines even on filesystem failure.
pub fn plugin_stderr(config: &Path, uuid: &str, stderr: impl std::io::Read + Send + 'static) {
    let path = directory(config)
        .join("plugins")
        .join(format!("{}.log", safe_uuid(uuid)));
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut line = Vec::new();
        loop {
            let buf = match reader.fill_buf() {
                Ok(b) if !b.is_empty() => b,
                _ => break,
            };
            let count = buf.len();
            for &b in buf {
                if b == b'\n' {
                    let text = String::from_utf8_lossy(&line);
                    let _ = append(
                        &path,
                        &format!("{} stderr {}", local_stamp(), redact(&text)),
                    );
                    line.clear();
                } else if line.len() < LINE_LIMIT {
                    line.push(b);
                }
            }
            reader.consume(count);
        }
        if !line.is_empty() {
            let _ = append(
                &path,
                &format!(
                    "{} stderr {}",
                    local_stamp(),
                    redact(&String::from_utf8_lossy(&line))
                ),
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn log_rotation_keeps_three_archives() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("novydeck.log");
        for i in 0..5 {
            fs::write(&p, vec![b'x'; LIMIT as usize]).unwrap();
            append(&p, &format!("line {i}")).unwrap();
        }
        assert_eq!(fs::read_to_string(&p).unwrap(), "line 4\n");
        assert!(p.with_extension("log.3").exists());
        assert!(!p.with_extension("log.4").exists());
    }
    #[test]
    fn secrets_redacted_and_diagnostics_retained() {
        let raw = "error Bearer abc token=xyz password=hunter2 Authorization: secretvalue 0123456789abcdef0123456789abcdef failed";
        let clean = redact(raw);
        for secret in ["abc", "xyz", "hunter2", "secretvalue", "0123456789abcdef"] {
            assert!(!clean.contains(secret));
        }
        assert!(clean.contains("error") && clean.contains("failed"));
        assert!(
            safe_event("Plugin uuid : novydeckTrigger refusé (édition en cours)")
                .contains("novydeckTrigger refusé")
        );
        assert_eq!(
            safe_event("dirty : utilisateur/UI"),
            "dirty : utilisateur/UI"
        );
        assert_eq!(
            safe_event("plugin setSettings settings: {secret}"),
            "plugin setSettings (détails masqués)"
        );
    }
    #[test]
    fn plugin_stderr_preserves_useful_text_without_secret() {
        let d = tempfile::tempdir().unwrap();
        let config = d.path().join("config.json");
        let payload = b"failed token=private123 retry later\n";
        plugin_stderr(&config, "uuid", &payload[..]);
        let p = directory(&config).join("plugins/uuid.log");
        for _ in 0..100 {
            if p.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let text = fs::read_to_string(p).unwrap();
        assert!(
            text.contains("failed") && text.contains("retry later") && !text.contains("private123")
        );
    }
}
