//! Explicit plugin lifecycle confirmation and off-UI disk work.
use super::App;
use crate::{community_plugins::{self,Entry}, model::{Config,Step}, plugin_runtime::PluginControl};
use eframe::egui::{self,RichText};
use std::{path::{Path,PathBuf},sync::mpsc::{self,Receiver}};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super) enum Choice { Install, Installed, Update, Invalid }
pub(super) fn choice(next:&str,old:Option<&str>,compatible:bool)->Choice {
    match old {
        None => if compatible {Choice::Install} else {Choice::Invalid},
        Some(old) if !compatible => {let _=old;Choice::Installed},
        Some(old) => match crate::plugin_management::newer(next,old) {Ok(true)=>Choice::Update,Ok(false)=>Choice::Installed,Err(_)=>Choice::Invalid},
    }
}
#[derive(Clone)] pub(super) enum Request { Update {entry:Entry, old:String, folder:PathBuf}, Remove {id:String,name:String,version:String} }
impl Request {
    fn id(&self)->&str {match self {Self::Update{entry,..}=>entry.uuid.as_deref().unwrap_or(""),Self::Remove{id,..}=>id}}
    fn name(&self)->&str {match self {Self::Update{entry,..}=>&entry.name,Self::Remove{name,..}=>name}}
}
pub(super) enum Operation {Update {source:PathBuf,protected:Vec<PathBuf>},Remove}
pub(super) fn disk_operation(root:&Path,id:&str,operation:Operation)->anyhow::Result<PathBuf>{
    match operation {Operation::Update{source,protected}=>crate::plugin_management::replace_staged_preserving(root,id,&source,&protected),Operation::Remove=>crate::plugin_management::uninstall(root,id)}
}
fn walk(steps:&[Step],id:&str)->usize {
    steps.iter().map(|step|match step {
        Step::PluginAction{plugin,..}=>usize::from(plugin==id),
        Step::Disabled{step}=>walk(std::slice::from_ref(step.as_ref()),id),
        Step::Repeat{steps,..}=>walk(steps,id),
        Step::Toggle{a,b}=>walk(a,id)+walk(b,id),
        Step::IfCondition{then_steps,else_steps,..}|Step::IfWindow{then_steps,else_steps,..}|Step::IfVariable{then_steps,else_steps,..}=>walk(then_steps,id)+walk(else_steps,id),
        _=>0,
    }).sum()
}
pub(super) fn usages(config:&Config,id:&str)->Vec<String>{
    let mut result=Vec::new();
    for profile in &config.profiles {
        for (page,keys) in std::iter::once(("",&profile.keys)).chain(profile.pages.iter().map(|p|(p.name.as_str(),&p.keys))) {
            for (index,key) in keys.iter().enumerate() {
                let location=format!("{} / {} / {}",profile.name,if page.is_empty(){crate::i18n::tr("Accueil")}else{page},index+1);
                if key.plugin.as_ref().is_some_and(|binding|binding.plugin==id){result.push(format!("{location} · {}",crate::i18n::tr("touche")));}
                for (label,steps) in [(crate::i18n::tr("séquence"),&key.steps),(crate::i18n::tr("appui long"),&key.long_steps)] {
                    let count=walk(steps,id);if count>0 {result.push(format!("{location} · {label} ({count})"));}
                }
            }
        }
    }
    result
}
fn image_paths(config:&Config)->Vec<String>{
    let mut images=Vec::new();
    for profile in &config.profiles {for keys in std::iter::once(&profile.keys).chain(profile.pages.iter().map(|p|&p.keys)) {for key in keys {
        images.push(key.image.clone());
        if let Some(state)=&key.state_display {for face in [&state.on,&state.off] {if let Some(image)=&face.image {images.push(image.clone());}}}
    }}}
    images
}
fn protected_images(folder:&Path,images:&[String])->Vec<PathBuf>{
    let Ok(base)=folder.canonicalize() else{return vec![]};
    images.iter().filter_map(|image| {
        if image.starts_with("builtin:") || image.is_empty(){return None;}
        let native=image.replace('\\',std::path::MAIN_SEPARATOR_STR);
        let path=PathBuf::from(native);
        let absolute=path.canonicalize().ok()?;
        let relative=absolute.strip_prefix(&base).ok()?;
        (absolute.is_file() && !relative.as_os_str().is_empty()).then(||relative.to_path_buf())
    }).collect()
}
pub(super) enum JobResult { Disk {id:String,remove:bool,paused:bool,result:Result<PathBuf,String>,control:PluginControl}, Resume {result:Result<(),String>,disk:Result<PathBuf,String>,remove:bool} }
#[derive(Default)] pub(super) struct State {pub confirmation:Option<Request>,pub working:Option<Receiver<JobResult>>,pub message:String}
impl App {
    pub(super) fn management_start(&mut self,ctx:&egui::Context){
        let Some(request)=self.plugin_management.confirmation.take() else{return};
        if self.plugin_management.working.is_some(){return;}
        let Some(control)=self.plugins.as_ref().map(|m|m.control()) else {self.plugin_management.message=crate::i18n::tr("Runtime indisponible : opération annulée.").into();return};
        let images=image_paths(&self.config);
        let (tx,rx)=mpsc::channel();self.plugin_management.working=Some(rx);
        let ctx=ctx.clone();std::thread::spawn(move||{
            let id=request.id().to_owned();let remove=matches!(request,Request::Remove{..});
            let result=control.pause_plugin(&id).and_then(|_|{
                let outcome=(||->anyhow::Result<PathBuf>{
                    let root=crate::plugin_import::installed_root()?;
                    let operation=match request {
                        Request::Update{entry,folder,..}=>{
                            let (_staging,source)=community_plugins::stage_verified_update(&entry,&root)?;
                            let protected=protected_images(&folder,&images);
                            return disk_operation(&root,&id,Operation::Update{source,protected});
                        }
                        Request::Remove{..}=>Operation::Remove,
                    };
                    disk_operation(&root,&id,operation)
                })();
                Ok(outcome.map_err(|e|e.to_string()))
            });
            // Pause failure must not trigger a catalog refresh or disk mutation.
            match result {Ok(result)=>{let _=tx.send(JobResult::Disk{id,remove,paused:true,result,control});},Err(error)=>{let _=tx.send(JobResult::Disk{id,remove,paused:false,result:Err(format!("Pause refusée : {error}")),control});}}
            ctx.request_repaint();
        });
    }
    pub(super) fn management_poll(&mut self,ctx:&egui::Context){
        let received=self.plugin_management.working.as_ref().map(|rx|rx.try_recv());
        let result=match received {
            None|Some(Err(std::sync::mpsc::TryRecvError::Empty))=>return,
            Some(Ok(result))=>result,
            Some(Err(std::sync::mpsc::TryRecvError::Disconnected))=>{
                self.plugin_management.working=None;
                self.plugin_management.message=crate::i18n::tr("Le traitement du plugin a été interrompu.").into();
                return;
            }
        };
        self.plugin_management.working=None;
        match result {
            JobResult::Disk{id,remove,paused,result,control}=>{
                if !paused {self.plugin_management.message=format!("{}",result.unwrap_err());return;}
                self.plugin_states.clear_plugin(&id);
                // Never restart against an obsolete catalog. Even a failed disk transaction may have rolled back.
                self.refresh_plugin_catalog();
                let (tx,rx)=mpsc::channel();self.plugin_management.working=Some(rx);let ctx=ctx.clone();
                std::thread::spawn(move||{let resume=control.resume_plugin(&id);let _=tx.send(JobResult::Resume{result:resume,disk:result,remove});ctx.request_repaint();});
            }
            JobResult::Resume{result,disk,remove}=>{
                let outcome=match &disk {Ok(backup)=>format!("{} · {}: {}",if remove {crate::i18n::tr("Plugin désinstallé")}else{crate::i18n::tr("Plugin mis à jour")},crate::i18n::tr("Sauvegarde"),backup.display()),Err(error)=>format!("{}: {error}",crate::i18n::tr("Opération impossible"))};
                let resume=match result {
                    Ok(()) if remove && disk.is_ok()=>format!(" · {}",crate::i18n::tr("Références conservées ; plugin indisponible. Réinstallez-le ou réaffectez les touches.")),
                    Ok(())=>String::new(),
                    Err(error) if remove && disk.is_ok() && error=="Plugin installé introuvable"=>format!(" · {}",crate::i18n::tr("Références conservées ; plugin indisponible. Réinstallez-le ou réaffectez les touches.")),
                    Err(error)=>format!(" · {}: {error}",crate::i18n::tr("Remise en route non confirmée")),
                };
                self.plugin_management.message=format!("{outcome}{resume}");
            }
        }
    }
    pub(super) fn management_modal(&mut self,ctx:&egui::Context){
        let active=self.plugin_management.confirmation.is_some()||self.plugin_management.working.is_some();
        if !active{return;}
        egui::Area::new(egui::Id::new("plugin_management_block")).order(egui::Order::Middle).fixed_pos(ctx.screen_rect().min).show(ctx,|ui|{
            let rect=ctx.screen_rect();ui.allocate_rect(rect,egui::Sense::click());
            ui.painter().rect_filled(rect,0.0,egui::Color32::from_black_alpha(110));
        });
        if let Some(request)=self.plugin_management.confirmation.clone(){
            let mut open=true;let mut confirm=false;let mut cancel=false;
            let shown=crate::theme::fixed_window(crate::i18n::tr("Confirmer la gestion du plugin")).id(egui::Id::new("plugin_management_confirmation")).order(egui::Order::Foreground).open(&mut open).collapsible(false).movable(false).resizable(false).anchor(egui::Align2::CENTER_CENTER,egui::Vec2::ZERO).default_width(440.0).show(ctx,|ui|{
                ui.label(RichText::new(request.name()).strong());
                match &request {
                    Request::Update{entry,old,..}=>{ui.label(format!("{old} → {}",entry.version));ui.label(crate::i18n::tr("Ce plugin exécute du code avec vos droits utilisateur."));if community_plugins::badges(entry,&super::plugin_store_ui::community_state().lock().unwrap_or_else(|e|e.into_inner()).scans).warning {ui.label(crate::i18n::tr("Analyse de sécurité à examiner avant mise à jour."));}},
                    Request::Remove{id,version,..}=>{
                        ui.label(format!("v{version}"));
                        ui.add(egui::Label::new(crate::i18n::tr("Les profils, pages, images, macros et réglages restent inchangés ; les actions seront indisponibles jusqu'à réinstallation ou réaffectation.")).wrap());
                        let uses=usages(&self.config,id);
                        ui.label(crate::i18n::trf("Usages détectés : {count}",&[("count",&uses.len())]));
                        egui::ScrollArea::vertical().max_height(160.0).show(ui,|ui|{for item in uses {ui.label(item);}});
                    }
                }
                ui.horizontal(|ui|{if ui.button(crate::i18n::tr("Annuler")).clicked(){cancel=true;}
                    if ui.button(if matches!(request,Request::Remove{..}){crate::i18n::tr("Confirmer la désinstallation")}else{crate::i18n::tr("Confirmer la mise à jour")}).clicked(){confirm=true;}});
            });
            if let Some(shown)=shown {let layer=shown.response.layer_id;ctx.move_to_top(layer);ctx.memory_mut(|memory|memory.set_modal_layer(layer));}
            if !open || cancel {self.plugin_management.confirmation=None;}else if confirm {self.management_start(ctx);}
        }
        if self.plugin_management.working.is_some(){
            let shown=crate::theme::fixed_window(crate::i18n::tr("Gestion du plugin en cours")).id(egui::Id::new("plugin_management_progress")).order(egui::Order::Foreground).collapsible(false).movable(false).resizable(false).anchor(egui::Align2::CENTER_CENTER,egui::Vec2::ZERO).show(ctx,|ui|{ui.horizontal(|ui|{ui.spinner();ui.label(crate::i18n::tr("Veuillez patienter : préparation, sauvegarde et reprise du plugin."));});});
            if let Some(shown)=shown {let layer=shown.response.layer_id;ctx.move_to_top(layer);ctx.memory_mut(|memory|memory.set_modal_layer(layer));}
        }
    }
}
