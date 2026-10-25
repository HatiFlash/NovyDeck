//! Transactions de plugins installés. Appeler hors du thread UI, après pause acquittée du runtime.
#[cfg(test)] mod tests {
    use super::*;
    use std::{fs,path::Path};
    fn fixture(root:&Path, name:&str, uuid:&str, version:&str) {
        let dir=root.join(name);fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("manifest.json"),format!(r#"{{"UUID":"{uuid}","Version":"{version}","CodePath":"main.js","Actions":[{{"UUID":"a"}}]}}"#)).unwrap();
        fs::write(dir.join("main.js"),version).unwrap();
    }
    #[test] fn update_preserves_unknown_files_in_backup_and_external_state() {
        let d=tempfile::tempdir().unwrap();let installed=d.path().join("plugins");fixture(&installed,"a.sdPlugin","test.a","1.0.0");
        let old=installed.join("a.sdPlugin");fs::write(old.join("custom.data"),"keep").unwrap();fs::create_dir_all(d.path().join("profiles")).unwrap();fs::write(d.path().join("profiles/settings.json"),"saved").unwrap();
        let staged=d.path().join("staging");fixture(&staged,"a.sdPlugin","test.a","2.0.0");
        let backup=replace_staged(&installed,"test.a",&staged.join("a.sdPlugin")).unwrap();
        assert_eq!(fs::read_to_string(installed.join("a.sdPlugin/main.js")).unwrap(),"2.0.0");
        assert_eq!(fs::read_to_string(backup.join("custom.data")).unwrap(),"keep");
        assert!(!installed.join("a.sdPlugin/custom.data").exists());
        assert_eq!(fs::read_to_string(d.path().join("profiles/settings.json")).unwrap(),"saved");
    }
    #[test] fn invalid_identity_and_version_never_mutate() {
        let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");fixture(&root,"a.sdPlugin","test.a","2.0");
        let stage=d.path().join("stage");fixture(&stage,"a.sdPlugin","test.b","3.0");
        assert!(replace_staged(&root,"test.a",&stage.join("a.sdPlugin")).is_err());
        fixture(&stage,"a.sdPlugin","test.a","1.0");
        assert!(replace_staged(&root,"test.a",&stage.join("a.sdPlugin")).is_err());
        assert_eq!(fs::read_to_string(root.join("a.sdPlugin/main.js")).unwrap(),"2.0");
    }
    #[test] fn uninstall_retains_backup_and_profiles() {
        let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");fixture(&root,"a.sdPlugin","test.a","1.0");
        fs::write(d.path().join("settings.json"),"unchanged").unwrap();
        let backup=uninstall(&root,"test.a").unwrap();
        assert!(backup.join("manifest.json").is_file());assert!(!root.join("a.sdPlugin").exists());
        assert_eq!(fs::read_to_string(d.path().join("settings.json")).unwrap(),"unchanged");
    }
}
