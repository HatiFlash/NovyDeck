//! Transactions de plugins installés. Appeler hors du thread UI, après pause acquittée du runtime.
use std::{fs,path::{Path,PathBuf}};
use anyhow::{Result,Context};

fn version(s:&str)->Result<Vec<u64>> {
    anyhow::ensure!(!s.is_empty() && s.len()<=64 && s.split('.').count()<=8,"Version absente ou invalide");
    s.split('.').map(|n| {anyhow::ensure!(!n.is_empty() && n.bytes().all(|c|c.is_ascii_digit()),"Version invalide");Ok(n.parse::<u64>()?)}).collect()
}
fn newer(next:&str,old:&str)->Result<bool>{let mut a=version(next)?;let mut b=version(old)?;let size=a.len().max(b.len());a.resize(size,0);b.resize(size,0);Ok(a>b)}
fn uuid(id:&str)->Result<()> {
    anyhow::ensure!(!id.is_empty() && id.len()<=128 && id.bytes().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||matches!(c,b'.'|b'-')) && !id.starts_with('.') && !id.ends_with('.') && !id.contains(".."),"UUID invalide");Ok(())
}
fn no_links(path:&Path)->Result<()> {
    let meta=fs::symlink_metadata(path)?;
    anyhow::ensure!(!meta.file_type().is_symlink(),"Lien refusé");
    #[cfg(windows)] {use std::os::windows::fs::MetadataExt;anyhow::ensure!(meta.file_attributes() & 0x400 == 0,"Point de réanalyse refusé");}
    if meta.is_dir(){for entry in fs::read_dir(path)? {no_links(&entry?.path())?;}}
    else {anyhow::ensure!(meta.is_file(),"Type de fichier refusé");}Ok(())
}
fn local(root:&Path,id:&str)->Result<(PathBuf,crate::plugins::Plugin)> {
    uuid(id)?;no_links(root)?;
    let mut found=None;
    for entry in fs::read_dir(root)?.take(513) {
        let entry=entry?;let name=entry.file_name();let name=name.to_string_lossy();
        if name.starts_with('.') || name.starts_with("__") {continue;}
        let folder=entry.path();no_links(&folder)?;
        if !folder.is_dir(){continue;}
        let manifest=folder.join("manifest.json");if !manifest.is_file(){continue;}
        let p=crate::plugin_import::validated_folder(&folder)?;
        if p.uuid==id {anyhow::ensure!(found.is_none(),"UUID installé en double");found=Some((folder,p));}
    }
    found.ok_or_else(||anyhow::anyhow!("Plugin installé introuvable"))
}
/// Préparation hors UI : les octets doivent avoir été vérifiés SHA-256 avant cet appel.
/// La fonction ne touche jamais à l'installation existante avant validation complète.
/// Retourne le chemin du backup intégral, laissé en place pour restauration manuelle.
pub fn replace_staged(root:&Path,id:&str,source:&Path)->Result<PathBuf>{replace_staged_impl(root,id,source,false)}
fn replace_staged_impl(root:&Path,id:&str,source:&Path,fail_commit:bool)->Result<PathBuf>{
    let (old,previous)=local(root,id)?;no_links(source)?;
    let incoming=crate::plugin_import::validated_folder(source)?;
    let bytes=fs::read(source.join("manifest.json"))?;
    let manifest:serde_json::Value=serde_json::from_slice(bytes.strip_prefix(&[239,187,191]).unwrap_or(&bytes))?;
    anyhow::ensure!(manifest["UUID"].as_str()==Some(id),"UUID explicite requis dans le manifeste de mise à jour");
    uuid(&incoming.uuid)?;
    anyhow::ensure!(incoming.uuid==id && newer(&incoming.version,&previous.version)?,"UUID différent ou version non supérieure");
    // Copy into a private sibling first, with the importer's file-count/size limits.
    let staging=tempfile::Builder::new().prefix(".update-").tempdir_in(root)?;
    let candidate=staging.path().join(old.file_name().context("Nom de dossier absent")?);
    crate::plugin_import::copy_verified_tree(source,&candidate)?;
    let verified=crate::plugin_import::validated_folder(&candidate)?;
    anyhow::ensure!(verified.uuid==id && verified.version==incoming.version,"Staging modifié pendant la copie");
    no_links(&old)?;no_links(root)?;
    let (again,installed)=local(root,id)?;
    anyhow::ensure!(again==old && installed.version==previous.version,"Installation modifiée pendant la préparation");
    let backup_dir=tempfile::Builder::new().prefix(".plugin-backup-").tempdir_in(root)?.keep();
    let backup=backup_dir.join(old.file_name().unwrap());
    fs::rename(&old,&backup).context("Sauvegarde de l'installation impossible")?;
    let commit=if fail_commit {Err(std::io::Error::other("panne de commit simulée"))}else{fs::rename(&candidate,&old)};
    if let Err(error)=commit {
        fs::rename(&backup,&old).context("ROLLBACK ÉCHOUÉ : sauvegarde conservée, intervention nécessaire")?;
        return Err(error).context("Commit échoué, installation restaurée");
    }
    Ok(backup)
}
/// Désinstallation ciblée : déplace l'intégralité du dossier en sauvegarde ; aucun fichier externe n'est touché.
pub fn uninstall(root:&Path,id:&str)->Result<PathBuf>{
    let (old,_)=local(root,id)?;no_links(&old)?;
    let backup_dir=tempfile::Builder::new().prefix(".plugin-backup-").tempdir_in(root)?.keep();
    let backup=backup_dir.join(old.file_name().context("Nom de dossier absent")?);
    fs::rename(&old,&backup)?;Ok(backup)
}
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
        let old=installed.join("a.sdPlugin");fs::write(old.join("custom.data"),"keep").unwrap();
        fs::create_dir(old.join("images")).unwrap();fs::write(old.join("images/user.png"),b"image").unwrap();
        fs::write(old.join("config.json"),b"configuration").unwrap();
        fs::create_dir_all(d.path().join("profiles")).unwrap();fs::write(d.path().join("profiles/settings.json"),"saved").unwrap();
        let staged=d.path().join("staging");fixture(&staged,"a.sdPlugin","test.a","2.0.0");
        let backup=replace_staged(&installed,"test.a",&staged.join("a.sdPlugin")).unwrap();
        assert_eq!(fs::read_to_string(installed.join("a.sdPlugin/main.js")).unwrap(),"2.0.0");
        assert_eq!(fs::read_to_string(backup.join("custom.data")).unwrap(),"keep");
        assert_eq!(fs::read(backup.join("images/user.png")).unwrap(),b"image");
        assert_eq!(fs::read(backup.join("config.json")).unwrap(),b"configuration");
        assert_eq!(fs::read(installed.join("a.sdPlugin/custom.data")).unwrap(),b"keep");
        assert_eq!(fs::read(installed.join("a.sdPlugin/images/user.png")).unwrap(),b"image");
        assert_eq!(fs::read(installed.join("a.sdPlugin/config.json")).unwrap(),b"configuration");
        assert_eq!(fs::read_to_string(d.path().join("profiles/settings.json")).unwrap(),"saved");
    }
    #[test] fn protected_collision_and_packaged_code() {
        let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");fixture(&root,"a.sdPlugin","test.a","1.0");
        let old=root.join("a.sdPlugin");fs::create_dir(old.join("images")).unwrap();
        fs::write(old.join("images/user.png"),b"mine").unwrap();fs::write(old.join("main.js"),b"old code").unwrap();
        fs::write(old.join("i18n.json"),b"old translation").unwrap();
        let stage=d.path().join("stage");fixture(&stage,"a.sdPlugin","test.a","2.0");
        let src=stage.join("a.sdPlugin");fs::create_dir(src.join("images")).unwrap();
        fs::write(src.join("images/user.png"),b"new image").unwrap();fs::write(src.join("i18n.json"),b"new translation").unwrap();
        let backup=replace_staged_preserving(&root,"test.a",&src,&[PathBuf::from("images/user.png")]).unwrap();
        assert_eq!(fs::read(old.join("images/user.png")).unwrap(),b"mine");
        assert_eq!(fs::read(old.join("i18n.json")).unwrap(),b"new translation");
        assert_eq!(fs::read(old.join("main.js")).unwrap(),b"2.0");
        assert_eq!(fs::read(backup.join("main.js")).unwrap(),b"old code");
    }
    #[test] fn invalid_protection_never_mutates() {
        for path in ["../outside","/absolute","manifest.json","main.js","images/../main.js"] {
            let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");fixture(&root,"a.sdPlugin","test.a","1.0");
            let stage=d.path().join("stage");fixture(&stage,"a.sdPlugin","test.a","2.0");
            assert!(replace_staged_preserving(&root,"test.a",&stage.join("a.sdPlugin"),&[PathBuf::from(path)]).is_err(),"{path}");
            assert_eq!(fs::read(root.join("a.sdPlugin/main.js")).unwrap(),b"1.0");
        }
    }
    #[test] fn invalid_identity_and_version_never_mutate() {
        let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");fixture(&root,"a.sdPlugin","test.a","2.0");
        let stage=d.path().join("stage");fixture(&stage,"a.sdPlugin","test.b","3.0");
        assert!(replace_staged(&root,"test.a",&stage.join("a.sdPlugin")).is_err());
        fixture(&stage,"a.sdPlugin","test.a","1.0");
        assert!(replace_staged(&root,"test.a",&stage.join("a.sdPlugin")).is_err());
        assert_eq!(fs::read_to_string(root.join("a.sdPlugin/main.js")).unwrap(),"2.0");
    }
    #[test] fn failed_commit_rolls_back_exact_folder() {
        let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");fixture(&root,"a.sdPlugin","test.a","1.0");
        fs::write(root.join("a.sdPlugin/custom.data"),"old").unwrap();
        let stage=d.path().join("stage");fixture(&stage,"a.sdPlugin","test.a","2.0");
        assert!(replace_staged_impl(&root,"test.a",&stage.join("a.sdPlugin"),true).is_err());
        assert_eq!(fs::read_to_string(root.join("a.sdPlugin/main.js")).unwrap(),"1.0");
        assert_eq!(fs::read_to_string(root.join("a.sdPlugin/custom.data")).unwrap(),"old");
    }
    #[test] fn links_and_escape_rejected() {
        let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");fixture(&root,"a.sdPlugin","test.a","1.0");
        assert!(uninstall(&root,"../test.a").is_err());
        #[cfg(windows)] {use std::os::windows::fs::symlink_file;
            let file=root.join("a.sdPlugin/link");
            if symlink_file(root.join("a.sdPlugin/main.js"),&file).is_ok(){assert!(uninstall(&root,"test.a").is_err());}
        }
    }
    #[test] fn uninstall_retains_backup_and_profiles() {
        let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");fixture(&root,"a.sdPlugin","test.a","1.0");
        fs::write(d.path().join("settings.json"),"unchanged").unwrap();
        let backup=uninstall(&root,"test.a").unwrap();
        assert!(backup.join("manifest.json").is_file());assert!(!root.join("a.sdPlugin").exists());
        assert_eq!(fs::read_to_string(d.path().join("settings.json")).unwrap(),"unchanged");
    }
}
