use super::*;

fn with_recents(paths: &[&str]) -> AppSettings {
    let mut s = AppSettings::default();
    // push_recent inserts at front, so push in reverse to get the given order.
    for p in paths.iter().rev() {
        s.push_recent(p);
    }
    s
}

#[test]
fn push_recent_inserts_at_front() {
    let mut s = AppSettings::default();
    s.push_recent("/a.md");
    s.push_recent("/b.md");
    assert_eq!(s.recent_files, vec!["/b.md", "/a.md"]);
}

#[test]
fn push_recent_deduplicates_and_promotes_to_front() {
    let mut s = with_recents(&["/a.md", "/b.md", "/c.md"]);
    // Re-opening an existing file moves it to the front without duplicating.
    s.push_recent("/c.md");
    assert_eq!(s.recent_files, vec!["/c.md", "/a.md", "/b.md"]);
    assert_eq!(s.recent_files.iter().filter(|p| *p == "/c.md").count(), 1);
}

#[test]
fn push_recent_caps_at_max() {
    let mut s = AppSettings::default();
    for i in 0..(MAX_RECENT + 5) {
        s.push_recent(&format!("/file{i}.md"));
    }
    assert_eq!(s.recent_files.len(), MAX_RECENT);
    // The most recently pushed survives; the oldest are dropped.
    assert_eq!(s.recent_files[0], format!("/file{}.md", MAX_RECENT + 4));
}

#[test]
fn remove_recent_drops_only_the_named_path() {
    let mut s = with_recents(&["/a.md", "/b.md", "/c.md"]);
    s.remove_recent("/b.md");
    assert_eq!(s.recent_files, vec!["/a.md", "/c.md"]);
}

// ── RFC-068 §7.1: settings go through the durable write ──────────────────────

mod durable {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn scratch(label: &str) -> std::path::PathBuf {
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "omriss-settings-{}-{label}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn save_creates_the_folder_and_what_is_read_back_is_what_was_saved() {
        let dir = scratch("roundtrip");
        let path = dir.join("nested").join("settings.toml");
        let mut s = AppSettings::default();
        s.push_recent("/a.md");
        s.font_size = 18;
        s.save_to(&path).unwrap();
        let back: AppSettings = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(back.recent_files, vec!["/a.md"]);
        assert_eq!(back.font_size, 18);
        // Nothing but the file itself: no temporary beside it.
        let left: Vec<_> = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(left, vec![std::ffi::OsString::from("settings.toml")]);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn a_save_that_cannot_complete_leaves_the_old_settings_whole() {
        // The old code truncated the file first (`fs::write`), so a failure part-way
        // left unparseable TOML, which `load()` discards: every preference and the
        // recent-files list, silently. Here the target is a directory: the write fails,
        // and a settings file beside it is not touched.
        let dir = scratch("intact");
        fs::create_dir_all(&dir).unwrap();
        let good = dir.join("settings.toml");
        let mut old = AppSettings::default();
        old.push_recent("/kept.md");
        old.save_to(&good).unwrap();
        let before = fs::read(&good).unwrap();

        let blocked = dir.join("as-a-directory");
        fs::create_dir_all(&blocked).unwrap();
        assert!(AppSettings::default().save_to(&blocked).is_err());

        assert_eq!(fs::read(&good).unwrap(), before);
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_save_into_a_folder_that_cannot_be_written_keeps_the_old_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = scratch("readonly");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.toml");
        let mut old = AppSettings::default();
        old.push_recent("/kept.md");
        old.save_to(&path).unwrap();
        let before = fs::read(&path).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).unwrap();
        let probe = fs::File::create(dir.join("probe")).is_ok();
        if !probe {
            let new = AppSettings {
                font_size: 30,
                ..AppSettings::default()
            };
            assert!(new.save_to(&path).is_err());
            assert_eq!(
                fs::read(&path).unwrap(),
                before,
                "the old settings must survive"
            );
        }
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        let _ = fs::remove_dir_all(dir);
    }
}
