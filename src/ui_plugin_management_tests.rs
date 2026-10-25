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

#[test]
fn management_removal_does_not_report_expected_missing_plugin_as_restart_failure() {
    for (remove, disk_ok, error, warning) in [
        (true,true,"Plugin installé introuvable",false),
        (true,true,"Runtime arrêté",true),
        (true,false,"Plugin installé introuvable",true),
        (false,true,"Plugin installé introuvable",true),
    ] {
        let mut h = Harness::new();
        crate::i18n::set_language("fr");
        let (tx,rx)=std::sync::mpsc::channel();
        h.app.plugin_management.working=Some(rx);
        tx.send(plugin_management_ui::JobResult::Resume {
            result:Err(error.into()),
            disk:if disk_ok {Ok(h.dir.path().join("backup"))} else {Err("transaction refusée".into())},
            remove,
        }).unwrap();
        h.app.management_poll(&h.ctx);
        assert!(h.app.plugin_management.working.is_none());
        assert_eq!(h.app.plugin_management.message.contains("Remise en route non confirmée"),warning,"{}",h.app.plugin_management.message);
        if remove && disk_ok && !warning {assert!(h.app.plugin_management.message.contains("Références conservées"));}
    }
}

#[test]
fn management_disconnected_worker_does_not_leave_interface_blocked() {
    let mut h=Harness::new();
    let (tx,rx)=std::sync::mpsc::channel::<plugin_management_ui::JobResult>();
    h.app.plugin_management.working=Some(rx);
    drop(tx);
    h.app.management_poll(&h.ctx);
    assert!(h.app.plugin_management.working.is_none(),"a dead worker must release the blocking progress layer");
    assert!(!h.app.plugin_management.message.is_empty(),"the interrupted operation must be reported, not silently accepted");
}

#[test]
fn management_confirmation_and_progress_block_background_edits() {
    for working in [false,true] {
        let mut h=Harness::new();
        crate::i18n::set_language("fr");
        let size=vec2(1366.0,768.0);
        let initial=h.warm(size);
        let theme=text_position(&initial.shapes,"Thème clair").or_else(||text_position(&initial.shapes,"Thème sombre")).expect("theme control in header");
        let dark=h.app.preferences.dark;
        let (_sender,rx)=std::sync::mpsc::channel::<plugin_management_ui::JobResult>();
        if working {h.app.plugin_management.working=Some(rx);} else {
            h.app.plugin_management.confirmation=Some(plugin_management_ui::Request::Remove{id:"test.a".into(),name:"Test".into(),version:"1.0".into()});
        }
        h.warm(size);
        h.click(size,theme);
        h.warm(size);
        assert_eq!(h.app.preferences.dark,dark,"background editing must be blocked during confirmation/progress, working={working}");
    }
}

struct ManagementMarketReset;
impl Drop for ManagementMarketReset {
    fn drop(&mut self) {
        super::plugin_market_ui::test_close();
        *super::plugin_store_ui::community_state().lock().unwrap_or_else(|e| e.into_inner()) = Default::default();
    }
}

fn local_fixture(h: &Harness) -> crate::plugins::Plugin {
    let folder = fixture(h.dir.path(), "local.sdPlugin", "test.a", "1.0");
    let mut plugin = crate::plugins::parse(&folder, &std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
    plugin.name = "Plugin local de test".into();
    plugin
}

#[test]
fn management_successful_update_refreshes_disk_inventory_and_disables_button() {
    let _reset = ManagementMarketReset;
    let mut h = Harness::new();
    crate::i18n::set_language("fr");
    let local = local_fixture(&h);
    let root = h.dir.path().join("plugins");
    std::fs::create_dir_all(&root).unwrap();
    let installed = fixture(&root, "local.sdPlugin", "test.a", "1.0");
    let incoming = fixture(&h.dir.path().join("stage"), "local.sdPlugin", "test.a", "2.0");
    let registry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/readme/registry.json");
    let mut entry = crate::community_plugins::parse(&std::fs::read(registry).unwrap()).unwrap().remove(0);
    entry.uuid = Some(local.uuid.clone());
    entry.version = "2.0".into();
    entry.compatible = true;
    entry.name = "Plugin de test du magasin".into();
    entry.icon = None;
    entry.screenshots.clear();
    h.app.plugins=Some(crate::plugin_runtime::Manager::new(vec![]));
    h.app.plugin_inventory=Some(std::sync::Arc::new(super::plugin_store_ui::Inventory{installed:vec![local.clone()],..Default::default()}));
    let control=h.app.plugins.as_ref().unwrap().control();
    control.pause_plugin("test.a").unwrap();
    let backup = plugin_management_ui::disk_operation(&root, "test.a", Operation::Update {source:incoming, protected:vec![]}).unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    h.app.plugin_management.working = Some(rx);
    let installed_plugin=crate::plugin_import::validated_folder(&installed).unwrap();
    tx.send(plugin_management_ui::JobResult::Disk{id:"test.a".into(),remove:false,paused:true,result:Ok((backup,Some(installed_plugin))),control}).unwrap();
    h.app.management_poll(&h.ctx);
    let on_disk = crate::plugin_import::inventory(&root);
    assert_eq!(on_disk.iter().find(|plugin|plugin.uuid=="test.a").unwrap().version,"2.0");
    for _ in 0..40 {if h.app.plugin_management.working.is_none(){break;}std::thread::sleep(std::time::Duration::from_millis(10));h.app.management_poll(&h.ctx);}
    assert!(h.app.plugin_management.working.is_none(),"le runtime doit reprendre");
    assert!(h.app.plugin_management.message.contains("Plugin mis à jour"),"{}",h.app.plugin_management.message);
    let cached=h.app.plugin_inventory.as_ref().unwrap();
    assert_eq!(cached.installed.iter().find(|p|p.uuid=="test.a").unwrap().version,"2.0");
    let mut state = super::plugin_store_ui::community_state().lock().unwrap();
    *state = Default::default();
    state.consent = true;
    state.entries = vec![entry.clone()];
    drop(state);
    super::plugin_market_ui::test_view(Some(&entry.id));
    let output = h.warm(vec2(1600.0,1000.0));
    assert!(text_position(&output.shapes, crate::i18n::tr("À jour")).is_some());
    assert!(text_position(&output.shapes, crate::i18n::tr("Mettre à jour")).is_none());
    let reloaded = crate::plugin_import::inventory(&root);
    assert_eq!(reloaded.iter().find(|plugin|plugin.uuid=="test.a").unwrap().version,"2.0");
}

#[test]
#[ignore = "télécharge les archives publiques : contrôle explicite sur la VM"]
fn management_live_store_archives_upgrade_without_touching_installed_plugins() {
    // All writes are confined to a temporary root, never the VM's installed_root.
    let entries=crate::community_plugins::fetch().unwrap();
    for (id,old) in [("com.novydeck.homeassistant","1.0.2"),("com.novydeck.googlehome","1.0.4")] {
        let entry=entries.iter().find(|entry|entry.id==id).unwrap();
        let root=tempfile::tempdir().unwrap();
        let plugin=fixture(root.path(),&format!("{id}.sdPlugin"),id,old);
        let (_owner,source)=crate::community_plugins::stage_verified_update(entry,root.path()).unwrap();
        let backup=plugin_management_ui::disk_operation(root.path(),id,Operation::Update{source,protected:vec![]}).unwrap();
        let observed=crate::plugin_import::validated_folder(&plugin).unwrap();
        assert_eq!(observed.version,entry.version,"{id}: mise à jour sur disque");
        assert_eq!(crate::plugin_import::inventory(root.path()).iter().find(|p|p.uuid==id).unwrap().version,entry.version,"{id}: inventaire rechargé");
        assert_eq!(plugin_management_ui::choice(&entry.version,Some(&observed.version),entry.compatible),plugin_management_ui::Choice::Installed,"{id}: bouton à jour");
        assert_eq!(crate::plugin_import::validated_folder(&backup).unwrap().version,old,"{id}: sauvegarde conservée");
    }
}

#[test]
fn management_store_widgets_show_installed_versions_and_refuse_downgrades_in_both_languages(){
    let _reset = ManagementMarketReset;
    let size = vec2(1600.0, 1000.0);
    for lang in ["fr", "en"] {
        let mut h = Harness::new();
        crate::i18n::set_language(lang);
        let local = local_fixture(&h);
        let registry = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/readme/registry.json");
        let mut entry = crate::community_plugins::parse(&std::fs::read(registry).unwrap()).unwrap().remove(0);
        entry.uuid = Some(local.uuid.clone());
        entry.version = "2.0".into();
        entry.compatible = true;
        entry.name = "Plugin de test du magasin".into();
        entry.icon = None;
        entry.screenshots.clear();
        {
            let mut state = super::plugin_store_ui::community_state().lock().unwrap();
            *state = Default::default();
            state.consent = true;
            state.entries = vec![entry.clone()];
        }
        for version in ["1.0", "2.0", "3.0"] {
            let mut plugin = local.clone();
            plugin.version = version.into();
            h.app.plugin_inventory = Some(std::sync::Arc::new(super::plugin_store_ui::Inventory { installed: vec![plugin], ..Default::default() }));
            super::plugin_market_ui::test_view(Some(&entry.id));
            let output = h.warm(size);
            if version == "1.0" {
                if let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT") {
                    h.png(size, output.clone(), &PathBuf::from(out).join(format!("management-store-{lang}.png")));
                }
                assert!(text_position(&output.shapes, crate::i18n::tr("Mettre à jour")).is_some(), "update button {lang}");
            } else {
                assert!(text_position(&output.shapes, crate::i18n::tr("Mettre à jour")).is_none(), "no same-version update or downgrade {lang}/{version}");
            }
            let installed = crate::i18n::trf("Version installée : {version}", &[("version", &version)]);
            let available = crate::i18n::trf("Version disponible : {version}", &[("version", &"2.0")]);
            assert!(text_position(&output.shapes, &installed).is_some(), "installed version {lang}/{version}");
            assert!(text_position(&output.shapes, &available).is_some(), "available version {lang}");
        }
        super::plugin_market_ui::test_close();
    }
}

#[test]
fn management_local_uninstall_is_visible_and_cancel_preserves_everything_in_both_languages() {
    let _reset = ManagementMarketReset;
    super::plugin_market_ui::test_close();
    let size = vec2(1366.0, 768.0);
    for lang in ["fr", "en"] {
        let mut h = Harness::new();
        crate::i18n::set_language(lang);
        let local = local_fixture(&h);
        let folder = local.folder.clone();
        h.app.plugin_inventory = Some(std::sync::Arc::new(super::plugin_store_ui::Inventory { installed: vec![local.clone()], ..Default::default() }));
        h.app.catalog_tab = 3;
        let before = serde_json::to_value(&h.app.config).unwrap();
        h.warm(size);
        let panel = egui::containers::panel::PanelState::load(&h.ctx, egui::Id::new("action_catalog")).unwrap().rect;
        super::navigation_seek(&mut h, size, crate::i18n::tr("Désinstaller"), panel.center(), -80.0);
        let output = h.warm(size);
        if let Some(out) = std::env::var_os("NOVYDECK_UI_OUTPUT") {
            h.png(size, output.clone(), &PathBuf::from(out).join(format!("management-local-{lang}.png")));
        }
        let button = text_position(&output.shapes, crate::i18n::tr("Désinstaller")).expect("uninstall is reachable in the compact local card");
        for shape in &output.shapes {
            if let egui::Shape::Text(text) = &shape.shape {
                if text.galley.job.text == crate::i18n::tr("Désinstaller") {
                    let bounds = egui::Rect::from_min_size(text.pos, text.galley.size());
                    assert!(bounds.min.x >= shape.clip_rect.min.x && bounds.max.x + h.ctx.style().spacing.button_padding.x <= shape.clip_rect.max.x + 1.0, "uninstall button clipped horizontally: {bounds:?} in {:?}", shape.clip_rect);
                }
            }
        }
        h.click(size, button);
        let output = h.warm(size);
        assert!(matches!(&h.app.plugin_management.confirmation, Some(plugin_management_ui::Request::Remove{id,..}) if id == &local.uuid));
        assert!(text_position(&output.shapes, crate::i18n::tr("Confirmer la désinstallation")).is_some());
        let cancel = output.shapes.iter().filter_map(|shape| {
            if let egui::Shape::Text(text) = &shape.shape {
                let pos = text.pos + text.galley.size() * 0.5;
                if text.galley.job.text == crate::i18n::tr("Annuler") && shape.clip_rect.contains(pos) { return Some(pos); }
            }
            None
        }).max_by(|a,b| a.y.total_cmp(&b.y)).expect("cancel confirmation, not the Undo control in the header");
        h.click(size, cancel);
        h.warm(size);
        assert!(h.app.plugin_management.confirmation.is_none());
        assert!(h.app.plugin_management.working.is_none());
        assert_eq!(serde_json::to_value(&h.app.config).unwrap(), before);
        assert!(folder.join("manifest.json").is_file(), "cancel never removes the plugin");
    }
}

