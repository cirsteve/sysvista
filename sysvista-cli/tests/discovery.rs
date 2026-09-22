#[cfg(unix)]
mod unix_tests {
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    use sysvista_cli::{
        discovery::{Config, InventoryOutcome},
        scanner,
    };

    struct TempDir(PathBuf);
    impl TempDir {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir()
                .join(format!("sysvista-discovery-{}-{nonce}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn copy_fixture(destination: &Path) {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/discovery");
        fs::create_dir_all(destination.join("excluded")).unwrap();
        for name in [
            "sysvista.toml",
            "included.rs",
            "unknown.xyz",
            "custom.custom",
        ] {
            fs::copy(fixture.join(name), destination.join(name)).unwrap();
        }
        fs::copy(
            fixture.join("excluded/ignored.rs"),
            destination.join("excluded/ignored.rs"),
        )
        .unwrap();
    }

    #[test]
    fn inventory_classifies_every_path_and_scan_survives_unreadable_files() {
        let temp = TempDir::new();
        let root = temp.0.join("project");
        copy_fixture(&root);
        let unreadable = root.join("secret.rs");
        fs::write(&unreadable, "pub struct Secret;").unwrap();
        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000)).unwrap();
        let outside = temp.0.join("outside.rs");
        fs::write(&outside, "pub struct Outside;").unwrap();
        symlink(&outside, root.join("outside-link.rs")).unwrap();
        fs::write(root.join(".hidden.rs"), "pub struct Hidden;").unwrap();
        fs::write(root.join("ignored-by-git.rs"), "pub struct IgnoredByGit;").unwrap();
        fs::write(root.join(".gitignore"), "ignored-by-git.rs\n").unwrap();

        let config = Config::load(&root).unwrap();
        let snapshot = scanner::scan_v2(&root, &config).unwrap();
        let counts = &snapshot.manifest.inventory;
        let sum = counts.included
            + counts.excluded
            + counts.unsupported
            + counts.unreadable
            + counts.failed;
        assert_eq!(sum as usize, snapshot.manifest.inventory_entries.len());
        assert!(
            snapshot
                .manifest
                .inventory_entries
                .iter()
                .any(|entry| entry.path == "outside-link.rs"
                    && entry.outcome
                        == InventoryOutcome::Excluded {
                            rule: "symlink".into()
                        })
        );
        assert!(snapshot.manifest.inventory_entries.iter().any(|entry| {
            entry.path == ".hidden.rs"
                && entry.outcome
                    == InventoryOutcome::Excluded {
                        rule: "hidden".into(),
                    }
        }));
        assert!(snapshot.manifest.inventory_entries.iter().any(|entry| {
            entry.path == "ignored-by-git.rs"
                && entry.outcome
                    == InventoryOutcome::Excluded {
                        rule: "gitignore".into(),
                    }
        }));
        assert!(
            snapshot
                .manifest
                .inventory_entries
                .iter()
                .any(|entry| entry.path == "excluded/ignored.rs"
                    && matches!(entry.outcome, InventoryOutcome::Excluded { .. }))
        );
        assert!(
            snapshot
                .source_files
                .iter()
                .any(|file| file.path == "unknown.xyz"
                    && matches!(
                        file.analysis,
                        sysvista_cli::output::v2::AnalysisStatus::Unsupported
                    ))
        );
        assert!(
            snapshot
                .source_files
                .iter()
                .any(|file| file.path == "custom.custom"
                    && file.language.as_deref() == Some("custom"))
        );
        assert_eq!(snapshot.diagnostics.iter().filter(|diagnostic| matches!(diagnostic, sysvista_cli::output::v2::Diagnostic::UnreadableFile { path, .. } if path == "secret.rs")).count(), 1);

        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o644)).unwrap();
    }

    #[test]
    fn invalid_config_fails_before_discovery() {
        let temp = TempDir::new();
        fs::write(
            temp.0.join("sysvista.toml"),
            "[discovery]\ninclude = ['[']\n",
        )
        .unwrap();
        assert!(Config::load(&temp.0).is_err());
    }

    #[test]
    fn reserved_and_duplicate_module_names_are_rejected() {
        for config in [
            "[[modules]]\nname = 'Unassigned'\nselectors = ['src/**']\n",
            "[[modules]]\nname = 'ui'\nselectors = ['src/**']\n\n[[modules]]\nname = 'ui'\nselectors = ['tests/**']\n",
        ] {
            let temp = TempDir::new();
            fs::write(temp.0.join("sysvista.toml"), config).unwrap();
            assert!(Config::load(&temp.0).is_err());
        }
    }

    #[test]
    fn invalid_v2_config_does_not_block_explicit_v1_scan() {
        let temp = TempDir::new();
        fs::write(
            temp.0.join("sysvista.toml"),
            "[discovery]\ninclude = ['[']\n",
        )
        .unwrap();
        fs::write(temp.0.join("model.rs"), "pub struct Model;").unwrap();
        let output = temp.0.join("legacy.json");
        let status = Command::new(env!("CARGO_BIN_EXE_sysvista-cli"))
            .args([
                "scan",
                temp.0.to_str().unwrap(),
                "--format",
                "v1",
                "--output",
            ])
            .arg(&output)
            .status()
            .unwrap();
        assert!(status.success());
        assert!(output.is_file());
    }
}
