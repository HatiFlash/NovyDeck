use super::*;
use super::plugin_management_ui::{self, Choice, Operation};

fn fixture(root: &std::path::Path, name: &str, id: &str, version: &str) -> std::path::PathBuf {
    let folder = root.join(name);
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("manifest.json"), format!(r#"{{"UUID":"{id}","Version":"{version}","CodePath":"main.js","Actions":[{{"UUID":"a"}}]}}"#)).unwrap();
    std::fs::write(folder.join("main.js"), version).unwrap();
    folder
}

#[test]
fn management_version_is_strict_and_never_downgrades() {
    assert_eq!(plugin_management_ui::choice("2.0", Some("1.9"), true), Choice::Update);
    assert_eq!(plugin_management_ui::choice("2.0", Some("2.0.0"), true), Choice::Installed);
    assert_eq!(plugin_management_ui::choice("2.0", Some("3.0"), true), Choice::Installed);
    assert_eq!(plugin_management_ui::choice("broken", Some("1.0"), true), Choice::Invalid);
    assert_eq!(plugin_management_ui::choice("2.0", Some("1.0"), false), Choice::Installed);
}

#[test]
fn management_usage_finds_other_profile_nested_long_and_never_edits_config() {
    let mut h = Harness::new();
    let mut profile = h.app.config.profiles[0].clone();
    profile.name = "Second profile".into();
    profile.keys[1].plugin = Some(crate::model::PluginBinding { plugin:"test.a".into(), action:"a".into(), settings:serde_json::json!({}),state:0 });
    profile.pages.push(crate::model::Page::new("folder".into(), "Subfolder".into(), true));
    profile.pages[0].keys[2].long_steps.push(Step::Disabled {step:Box::new(Step::Repeat {count:2, steps:vec![Step::PluginAction {plugin:"test.a".into(),action:"a".into(),settings:serde_json::json!({}),label:String::new()}]})});
    h.app.config.profiles.push(profile);
    let before = serde_json::to_vec(&h.app.config).unwrap();
    let uses = plugin_management_ui::usages(&h.app.config, "test.a");
    assert!(uses.iter().any(|u| u.contains("Second profile") && u.contains("Subfolder") && u.contains("long")), "{uses:?}");
    assert!(uses.len() >= 2, "{uses:?}");
    assert_eq!(serde_json::to_vec(&h.app.config).unwrap(), before);
}

#[test]
fn management_fixture_update_and_removal_preserve_unrelated_and_user_data() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("plugins");
    let old = fixture(&root, "a.sdPlugin", "test.a", "1.0");
    let other = fixture(&root, "b.sdPlugin", "test.b", "1.0");
    std::fs::create_dir_all(old.join("images")).unwrap();
    std::fs::write(old.join("images/user.png"), b"user").unwrap();
    std::fs::write(old.join("settings.json"), b"private").unwrap();
    let incoming = fixture(&dir.path().join("stage"), "a.sdPlugin", "test.a", "2.0");
    std::fs::create_dir_all(incoming.join("images")).unwrap();
    std::fs::write(incoming.join("images/user.png"), b"packaged").unwrap();
    let backup = plugin_management_ui::disk_operation(&root,"test.a",Operation::Update {source:incoming, protected:vec![std::path::PathBuf::from("images/user.png")]}).unwrap();
    assert_eq!(std::fs::read(old.join("main.js")).unwrap(),b"2.0");
    assert_eq!(std::fs::read(old.join("settings.json")).unwrap(),b"private");
    assert_eq!(std::fs::read(old.join("images/user.png")).unwrap(),b"user");
    assert_eq!(std::fs::read(backup.join("main.js")).unwrap(),b"1.0");
    assert_eq!(std::fs::read(other.join("main.js")).unwrap(),b"1.0");
    let backup = plugin_management_ui::disk_operation(&root,"test.a",Operation::Remove).unwrap();
    assert!(!old.exists());
    assert_eq!(std::fs::read(backup.join("settings.json")).unwrap(),b"private");
    assert_eq!(std::fs::read(other.join("main.js")).unwrap(),b"1.0");
}
