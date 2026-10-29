//! Transactions de plugins installés. Appeler hors du thread UI, après pause acquittée du runtime.
use std::{fs,path::{Path,PathBuf}};
use anyhow::{Result,Context};

fn version(s:&str)->Result<Vec<u64>> {
    anyhow::ensure!(!s.is_empty() && s.len()<=64 && s.split('.').count()<=8,"Version absente ou invalide");
    s.split('.').map(|n| {anyhow::ensure!(!n.is_empty() && n.bytes().all(|c|c.is_ascii_digit()),"Version invalide");Ok(n.parse::<u64>()?)}).collect()
}
pub(crate) fn newer(next:&str,old:&str)->Result<bool>{let mut a=version(next)?;let mut b=version(old)?;let size=a.len().max(b.len());a.resize(size,0);b.resize(size,0);Ok(a>b)}
fn uuid(id:&str)->Result<()> {
    anyhow::ensure!(!id.is_empty() && id.len()<=128 && id.bytes().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||matches!(c,b'.'|b'-')) && !id.starts_with('.') && !id.ends_with('.') && !id.contains(".."),"UUID invalide");Ok(())
}
fn metadata(path:&Path)->Result<fs::Metadata> {
    let meta=fs::symlink_metadata(path)?;
    anyhow::ensure!(!meta.file_type().is_symlink(),"Lien refusé");
    #[cfg(windows)] {use std::os::windows::fs::MetadataExt;anyhow::ensure!(meta.file_attributes() & 0x400 == 0,"Point de réanalyse refusé");}
    Ok(meta)
}
fn no_links(path:&Path)->Result<()> {
    let meta=metadata(path)?;
    if meta.is_dir(){for entry in fs::read_dir(path)? {no_links(&entry?.path())?;}}
    else {anyhow::ensure!(meta.is_file(),"Type de fichier refusé");}Ok(())
}
fn local(root:&Path,id:&str)->Result<(PathBuf,crate::plugins::Plugin)> {
    uuid(id)?;anyhow::ensure!(metadata(root)?.is_dir(),"Racine invalide");
    let mut found=None;let mut count=0;
    for entry in fs::read_dir(root)? {
        let entry=entry?;let name=entry.file_name();let name=name.to_string_lossy();
        if name.starts_with('.') || name.starts_with("__") {continue;}
        count+=1;anyhow::ensure!(count<=512,"Trop de dossiers de plugins");
        let folder=entry.path();let ty=entry.file_type()?;
        // Never dereference another plugin's junction to inspect its manifest.
        if ty.is_symlink() || !ty.is_dir(){continue;}
        #[cfg(windows)] if metadata(&folder).is_err(){continue;}
        let manifest=folder.join("manifest.json");
        let Ok(meta)=metadata(&manifest) else {continue;};
        if !meta.is_file(){continue;}
        anyhow::ensure!(meta.len()<=1024*1024,"Manifeste trop volumineux");
        let bytes=fs::read(&manifest)?;
        let Ok(p)=crate::plugins::parse(&folder,&bytes) else {continue;};
        if p.uuid==id {anyhow::ensure!(found.is_none(),"UUID installé en double");no_links(&folder)?;let p=crate::plugin_import::validated_folder(&folder)?;found=Some((folder,p));}
    }
    found.ok_or_else(||anyhow::anyhow!("Plugin installé introuvable"))
}
/// Préparation hors UI : les octets doivent avoir été vérifiés SHA-256 avant cet appel.
/// La fonction ne touche jamais à l'installation existante avant validation complète.
/// Retourne le chemin du backup intégral, laissé en place pour restauration manuelle.
/// Update without UI-provided protected paths. Unknown old files missing from the new package
/// are retained unless they are executable/package artifacts; divergent packaged assets
/// win over unprotected local copies, which are archived inside the plugin data folder.
pub fn replace_staged(root:&Path,id:&str,source:&Path)->Result<PathBuf>{replace_staged_preserving(root,id,source,&[])}
/// `protected` lists exact relative *files* owned by the user's profile (e.g. an image
/// referenced by a StateFace). The caller must provide all such collisions explicitly.
/// Settings under data/config/settings/userdata and root config.json/settings.json are local.
/// Packaged code, manifest, package metadata, schemas and localization always come from the update.
pub fn replace_staged_preserving(root:&Path,id:&str,source:&Path,protected:&[PathBuf])->Result<PathBuf>{replace_staged_impl_with_paths(root,id,source,protected,false)}
fn replace_staged_impl(root:&Path,id:&str,source:&Path,fail_commit:bool)->Result<PathBuf>{replace_staged_impl_with_paths(root,id,source,&[],fail_commit)}
fn packaged(path:&Path)->bool {
    let name=path.file_name().and_then(|s|s.to_str()).unwrap_or("").to_ascii_lowercase();
    let ext=path.extension().and_then(|s|s.to_str()).unwrap_or("").to_ascii_lowercase();
    name=="manifest.json" || name=="package.json" || name=="package-lock.json" || name==".deckforge-import.json"
        || ["readme.md","changelog.md","license"].contains(&name.as_str())
        || name.starts_with("i18n.") || name.starts_with("schema.")
        || ["js","mjs","cjs","dll","exe","node","html","css","wasm","map"].contains(&ext.as_str())
        || ["schema","schemas","localization","localizations","i18n","node_modules"].iter().any(|dir|path.components().any(|c|c.as_os_str().to_string_lossy().eq_ignore_ascii_case(dir)))
}
fn mutable(path:&Path)->bool {
    let first=path.components().next().and_then(|c|c.as_os_str().to_str()).unwrap_or("");
    let name=path.file_name().and_then(|s|s.to_str()).unwrap_or("");
    (path.components().count()==1 && (name.eq_ignore_ascii_case("config.json") || name.eq_ignore_ascii_case("settings.json")))
        || ["data","config","settings","userdata"].iter().any(|d|first.eq_ignore_ascii_case(d))
}
fn same_contents(left:&Path,right:&Path)->Result<bool> {
    use std::io::Read;
    let a=metadata(left)?;let b=metadata(right)?;
    if !a.is_file() || !b.is_file() || a.len()!=b.len(){return Ok(false);}
    anyhow::ensure!(a.len()<=100*1024*1024,"Donnée locale trop volumineuse");
    let mut a=fs::File::open(left)?;let mut b=fs::File::open(right)?;
    let mut left_bytes=[0_u8;8192];let mut right_bytes=[0_u8;8192];
    loop {
        let count=a.read(&mut left_bytes)?;let other=b.read(&mut right_bytes)?;
        if count!=other || left_bytes[..count]!=right_bytes[..other]{return Ok(false);}
        if count==0{return Ok(true);}
    }
}
fn preserve(old:&Path,candidate:&Path,protected:&[PathBuf])->Result<()> {
    fn visit(base:&Path,candidate:&Path,relative:&Path,protected:&[PathBuf],count:&mut usize,collisions:&mut Option<PathBuf>)->Result<()> {
        for entry in fs::read_dir(base.join(relative))? {
            let entry=entry?;*count+=1;anyhow::ensure!(*count<=4096,"Trop de fichiers locaux");
            let path=relative.join(entry.file_name());let ty=metadata(&entry.path())?;
            if ty.is_dir(){visit(base,candidate,&path,protected,count,collisions)?;continue;}
            anyhow::ensure!(ty.is_file(),"Type local refusé");
            let target=candidate.join(&path);let explicit=protected.iter().any(|p|p==&path);
            if !packaged(&path) && !explicit && !mutable(&path) && target.exists(){
                anyhow::ensure!(metadata(&target)?.is_file(),"Conflit fichier/dossier : {}",path.display());
                if same_contents(&entry.path(),&target)? {continue;}
                anyhow::ensure!(ty.len()<=100*1024*1024,"Donnée locale trop volumineuse");
                let archive=if let Some(dir)=collisions.as_ref(){dir.clone()}else{
                    let data=candidate.join("data");fs::create_dir_all(&data)?;
                    // Keep the user's archives; refuse further archival rather than silently
                    // discarding history or growing without a bound.
                    let prior=base.join("data");
                    let existing=if prior.exists(){fs::read_dir(&prior)?.filter_map(|e|e.ok()).filter(|e|e.file_name().to_string_lossy().starts_with(".update-collisions-")).count()}else{0};
                    anyhow::ensure!(existing<32,"Trop d'archives de collisions ; sauvegarder puis nettoyer data/.update-collisions-* avant la mise à jour");
                    let dir=tempfile::Builder::new().prefix(".update-collisions-").tempdir_in(data)?.keep();
                    *collisions=Some(dir.clone());dir
                };
                let saved=archive.join(&path);fs::create_dir_all(saved.parent().context("Parent absent")?)?;
                fs::copy(entry.path(),&saved)?;
                eprintln!("Mise à jour du plugin : {} remplacé par le paquet ; copie locale sauvegardée dans {}",path.display(),saved.display());
                continue;
            }
            let keep=!packaged(&path) && (explicit || mutable(&path) || !target.exists());
            if !keep {continue;}
            if target.exists(){anyhow::ensure!(metadata(&target)?.is_file(),"Conflit fichier/dossier : {}",path.display());}
            anyhow::ensure!(ty.len()<=100*1024*1024,"Donnée locale trop volumineuse");
            fs::create_dir_all(target.parent().context("Parent absent")?)?;
            fs::copy(entry.path(),target)?;
        }
        Ok(())
    }
    let mut count=0;let mut collisions=None;visit(old,candidate,Path::new(""),protected,&mut count,&mut collisions)
}
fn replace_staged_impl_with_paths(root:&Path,id:&str,source:&Path,protected:&[PathBuf],fail_commit:bool)->Result<PathBuf>{
    for path in protected {
        anyhow::ensure!(!path.as_os_str().is_empty() && path.components().all(|c|matches!(c,std::path::Component::Normal(_)))
            && !path.to_string_lossy().contains([':', '\0']) && !packaged(path),"Chemin protégé non confiné ou code packagé : {}",path.display());
    }
    let (old,previous)=local(root,id)?;no_links(source)?;
    let incoming=crate::plugin_import::validated_folder(source)?;
    let bytes=fs::read(source.join("manifest.json"))?;
    let manifest:serde_json::Value=serde_json::from_slice(bytes.strip_prefix(&[239,187,191]).unwrap_or(&bytes))?;
    // The official Stream Deck packages identify their plugin by the canonical
    // `{id}.sdPlugin` directory and have no root UUID field. Permit that exact
    // fallback only; an explicit UUID must still match the requested plugin.
    let explicit=match manifest.get("UUID") {
        None=>None,
        Some(serde_json::Value::String(value))=>Some(value.as_str()),
        Some(_)=>anyhow::bail!("UUID du manifeste invalide"),
    };
    let canonical=format!("{id}.sdPlugin");
    anyhow::ensure!(match explicit {Some(value)=>value==id,None=>source.file_name().is_some_and(|name|name==canonical.as_str())},
        "UUID explicite ou dossier canonique requis dans la mise à jour");
    if explicit.is_none(){anyhow::ensure!(old.file_name().is_some_and(|name|name==canonical.as_str()),
        "Dossier installé non canonique pour une archive sans UUID : conserver les chemins utilisateur impose une intervention manuelle");}
    uuid(&incoming.uuid)?;
    anyhow::ensure!(incoming.uuid==id && newer(&incoming.version,&previous.version)?,"UUID différent ou version non supérieure");
    // Copy into a private sibling first, with the importer's file-count/size limits.
    let staging=tempfile::Builder::new().prefix(".update-").tempdir_in(root)?;
    let candidate=staging.path().join(old.file_name().context("Nom de dossier absent")?);
    crate::plugin_import::copy_verified_tree(source,&candidate)?;
    let verified=crate::plugin_import::validated_folder(&candidate)?;
    anyhow::ensure!(verified.uuid==id && verified.version==incoming.version,"Staging modifié pendant la copie");
    for path in protected {anyhow::ensure!(metadata(&old.join(path))?.is_file(),"Fichier protégé absent ou invalide : {}",path.display());}
    preserve(&old,&candidate,protected)?;
    no_links(&old)?;no_links(&candidate)?;
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
        fs::create_dir(old.join("data")).unwrap();fs::write(old.join("data/config.json"),b"local").unwrap();
        fs::write(old.join("i18n.json"),b"old translation").unwrap();
        let stage=d.path().join("stage");fixture(&stage,"a.sdPlugin","test.a","2.0");
        let src=stage.join("a.sdPlugin");fs::create_dir(src.join("images")).unwrap();
        fs::write(src.join("images/user.png"),b"new image").unwrap();fs::write(src.join("i18n.json"),b"new translation").unwrap();
        fs::create_dir(src.join("data")).unwrap();fs::write(src.join("data/config.json"),b"defaults").unwrap();
        let backup=replace_staged_preserving(&root,"test.a",&src,&[PathBuf::from("images").join("user.png")]).unwrap();
        assert_eq!(fs::read(old.join("data/config.json")).unwrap(),b"local");
        assert_eq!(fs::read(old.join("images/user.png")).unwrap(),b"mine");
        assert_eq!(fs::read(old.join("i18n.json")).unwrap(),b"new translation");
        assert_eq!(fs::read(old.join("main.js")).unwrap(),b"2.0");
        assert_eq!(fs::read(backup.join("main.js")).unwrap(),b"old code");
    }
    #[test] fn unchanged_assets_do_not_require_explicit_protection() {
        let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");fixture(&root,"a.sdPlugin","test.a","1.0");
        let stage=d.path().join("stage");fixture(&stage,"a.sdPlugin","test.a","2.0");
        for folder in [root.join("a.sdPlugin"),stage.join("a.sdPlugin")] {
            fs::create_dir(folder.join("images")).unwrap();fs::write(folder.join("images/icon.png"),b"same unchanged packaged icon").unwrap();
        }
        replace_staged(&root,"test.a",&stage.join("a.sdPlugin")).unwrap();
        assert_eq!(fs::read(root.join("a.sdPlugin/main.js")).unwrap(),b"2.0");
        assert_eq!(fs::read(root.join("a.sdPlugin/images/icon.png")).unwrap(),b"same unchanged packaged icon");
    }
    #[test] fn divergent_asset_uses_package_and_archives_local_copy() {
        let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");fixture(&root,"a.sdPlugin","test.a","1.0");
        let old=root.join("a.sdPlugin");fs::create_dir(old.join("images")).unwrap();fs::write(old.join("images/user.png"),b"mine").unwrap();
        fs::write(old.join("README.md"),b"old readme").unwrap();
        let stage=d.path().join("stage");fixture(&stage,"a.sdPlugin","test.a","2.0");
        let incoming=stage.join("a.sdPlugin");fs::create_dir(incoming.join("images")).unwrap();fs::write(incoming.join("images/user.png"),b"new").unwrap();
        fs::write(incoming.join("README.md"),b"new readme").unwrap();
        let backup=replace_staged(&root,"test.a",&incoming).unwrap();
        assert_eq!(fs::read(old.join("images/user.png")).unwrap(),b"new");
        assert_eq!(fs::read(old.join("README.md")).unwrap(),b"new readme");
        assert_eq!(fs::read(backup.join("images/user.png")).unwrap(),b"mine");
        let archives=fs::read_dir(old.join("data")).unwrap().map(|e|e.unwrap().path()).collect::<Vec<_>>();
        assert_eq!(archives.len(),1);
        assert_eq!(fs::read(archives[0].join("images/user.png")).unwrap(),b"mine");
        assert!(!archives[0].join("README.md").exists());
        let next=d.path().join("next");fixture(&next,"a.sdPlugin","test.a","3.0");
        let newer=next.join("a.sdPlugin");fs::create_dir(newer.join("images")).unwrap();
        fs::write(newer.join("images/user.png"),b"newest").unwrap();
        replace_staged(&root,"test.a",&newer).unwrap();
        assert_eq!(fs::read(old.join("images/user.png")).unwrap(),b"newest");
        assert_eq!(fs::read(archives[0].join("images/user.png")).unwrap(),b"mine");
        let all=fs::read_dir(old.join("data")).unwrap().count();
        assert_eq!(all,2,"la copie initiale reste accessible après une nouvelle mise à jour");
    }
    #[test] fn collision_commit_failure_restores_exact_original() {
        let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");fixture(&root,"a.sdPlugin","test.a","1.0");
        let old=root.join("a.sdPlugin");fs::create_dir(old.join("images")).unwrap();fs::write(old.join("images/user.png"),b"mine").unwrap();
        let stage=d.path().join("stage");fixture(&stage,"a.sdPlugin","test.a","2.0");
        let incoming=stage.join("a.sdPlugin");fs::create_dir(incoming.join("images")).unwrap();fs::write(incoming.join("images/user.png"),b"new").unwrap();
        assert!(replace_staged_impl(&root,"test.a",&incoming,true).is_err());
        assert_eq!(fs::read(old.join("images/user.png")).unwrap(),b"mine");
        assert_eq!(fs::read(old.join("main.js")).unwrap(),b"1.0");
        assert!(!old.join("data").exists());
    }
    #[test] fn invalid_protection_never_mutates() {
        for path in ["../outside","/absolute","manifest.json","main.js","images/../main.js"] {
            let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");fixture(&root,"a.sdPlugin","test.a","1.0");
            let stage=d.path().join("stage");fixture(&stage,"a.sdPlugin","test.a","2.0");
            assert!(replace_staged_preserving(&root,"test.a",&stage.join("a.sdPlugin"),&[PathBuf::from(path)]).is_err(),"{path}");
            assert_eq!(fs::read(root.join("a.sdPlugin/main.js")).unwrap(),b"1.0");
        }
    }
    #[test] fn unrelated_link_does_not_block_targeted_update() {
        let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");fixture(&root,"a.sdPlugin","test.a","1.0");
        let outside=d.path().join("outside");fixture(&outside,"other.sdPlugin","test.other","1.0");
        #[cfg(windows)] {
            let link=root.join("other.sdPlugin");
            let status=std::process::Command::new("cmd").args(["/C","mklink","/J"]).arg(&link).arg(outside.join("other.sdPlugin")).status().unwrap();
            assert!(status.success(),"junction fixture required");
        }
        #[cfg(not(windows))] std::os::unix::fs::symlink(outside.join("other.sdPlugin"),root.join("other.sdPlugin")).unwrap();
        let stage=d.path().join("stage");fixture(&stage,"a.sdPlugin","test.a","2.0");
        replace_staged(&root,"test.a",&stage.join("a.sdPlugin")).unwrap();
        assert_eq!(fs::read(root.join("a.sdPlugin/main.js")).unwrap(),b"2.0");
        assert_eq!(fs::read(outside.join("other.sdPlugin/main.js")).unwrap(),b"1.0");
    }
    #[test] fn implicit_identity_uses_only_exact_canonical_directory() {
        let d=tempfile::tempdir().unwrap();let root=d.path().join("plugins");
        fixture(&root,"test.a.sdPlugin","test.a","1.0");
        let stage=d.path().join("stage");
        let candidate=fixture_without_uuid(&stage,"test.a.sdPlugin","2.0");
        // This is how the published first-party archives identify their plugin.
        let wrong=stage.join("other.sdPlugin");fs::rename(&candidate,&wrong).unwrap();
        assert!(replace_staged(&root,"test.a",&wrong).is_err());
        assert_eq!(fs::read_to_string(root.join("test.a.sdPlugin/main.js")).unwrap(),"1.0");
        fs::rename(&wrong,&candidate).unwrap();
        let manifest=candidate.join("manifest.json");
        let original=fs::read(&manifest).unwrap();
        let mut malformed:serde_json::Value=serde_json::from_slice(&original).unwrap();
        malformed["UUID"]=serde_json::json!(42);
        fs::write(&manifest,serde_json::to_vec(&malformed).unwrap()).unwrap();
        assert!(replace_staged(&root,"test.a",&candidate).unwrap_err().to_string().contains("UUID du manifeste invalide"));
        assert_eq!(fs::read_to_string(root.join("test.a.sdPlugin/main.js")).unwrap(),"1.0");
        fs::write(&manifest,original).unwrap();
        let legacy=d.path().join("legacy-root");fixture(&legacy,"legacy.sdPlugin","test.a","1.0");
        assert!(replace_staged(&legacy,"test.a",&candidate).unwrap_err().to_string().contains("non canonique"));
        assert_eq!(fs::read_to_string(legacy.join("legacy.sdPlugin/main.js")).unwrap(),"1.0");
        let backup=replace_staged(&root,"test.a",&candidate).unwrap();
        assert_eq!(fs::read_to_string(root.join("test.a.sdPlugin/main.js")).unwrap(),"2.0");
        assert_eq!(fs::read_to_string(backup.join("main.js")).unwrap(),"1.0");
    }
    fn fixture_without_uuid(root:&Path,name:&str,version:&str)->PathBuf {
        let folder=root.join(name);fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("manifest.json"),format!(r#"{{"Version":"{version}","CodePath":"main.js","Actions":[{{"UUID":"a"}}]}}"#)).unwrap();
        fs::write(folder.join("main.js"),version).unwrap();folder
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
