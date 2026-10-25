//! Cycle de vie des actions Stream Deck visibles (aucun I/O dans le verrou App).
use std::{collections::{HashMap,HashSet}, sync::mpsc::{self, Receiver, Sender}, thread, time::{Duration, Instant}};
use serde_json::{json, Value};
use crate::{model::Config, plugin_host::Host, plugins::Plugin};

/// Identité SDK d'un appareil physique, ou de l'appareil virtuel d'essai.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device { pub id:String, pub name:String, pub columns:usize, pub rows:usize, pub kind:u8 }
impl Device {
    pub fn virtual_device()->Self { let model=&crate::devices::MODELS[0];Self{id:"deckforge-virtual".into(),name:format!("{} (virtuel)",model.name),columns:model.cols as usize,rows:model.rows as usize,kind:0} }
    pub fn from_grid(name:String,columns:usize,rows:usize)->Self {
        let kind=crate::devices::MODELS.iter().find(|m|m.name==name && m.cols as usize==columns && m.rows as usize==rows).map_or(0,|m| match m.keys {6=>1,32=>2,_=>0});
        Self{id:format!("deckforge-{}x{}",columns,rows),name,columns,rows,kind}
    }
    pub fn info(&self)->Value {json!({"id":self.id,"name":self.name,"size":{"columns":self.columns,"rows":self.rows},"type":self.kind})}
    fn event(&self,name:&str)->Value {json!({"event":name,"device":self.id,"deviceInfo":{"name":self.name,"size":{"columns":self.columns,"rows":self.rows},"type":self.kind}})}
}
#[derive(Clone, Debug, PartialEq)]
pub struct Visible {
    pub context: String,
    pub plugin: String,
    pub action: String,
    pub index: usize,
    pub settings: Value,
    pub state: u8,
}
pub fn visible(config: &Config) -> HashMap<String, Visible> {
    config.keys().iter().enumerate().filter_map(|(index,key)| {
        let binding=key.plugin.as_ref()?;
        let context=format!("deckforge-{}-{}-{}",config.active,config.page,index);
        Some((context.clone(),Visible{context,plugin:binding.plugin.clone(),action:binding.action.clone(),index,settings:binding.settings.clone(),state:binding.state}))
    }).collect()
}
fn same_instance(a:&Visible,b:&Visible)->bool {
    a.context==b.context && a.plugin==b.plugin && a.action==b.action && a.index==b.index
}
/// `configured` records binding defaults, never plugin reports. A report survives a
/// redundant config sync only for the same action/settings/default; a changed binding resets it.
fn reconcile_state(next:&mut Visible,old:Option<&Visible>,configured:&mut HashMap<String,Visible>) {
    let previous=configured.get(&next.context);
    let keep=old.zip(previous).is_some_and(|(old,previous)|
        same_instance(old,next) && old.settings==next.settings && previous.settings==next.settings
        && previous.state==next.state);
    if keep {next.state=old.unwrap().state;}
    configured.insert(next.context.clone(),Visible{state:if keep {previous.unwrap().state}else{next.state},..next.clone()});
}
fn state_owner<'a>(current:&'a HashMap<String,Visible>,plugin:&str,value:&Value)->Option<&'a Visible>{
    current.get(value["context"].as_str()?).filter(|v|v.plugin==plugin &&
        value.get("action").is_none_or(|action|action.as_str()==Some(&v.action)))
}
pub fn diff(old:&HashMap<String,Visible>,new:&HashMap<String,Visible>) -> (Vec<Visible>,Vec<Visible>) {
    let gone=old.iter().filter(|(id,v)|!new.get(*id).is_some_and(|next|same_instance(v,next))).map(|(_,v)|v.clone()).collect();
    let added=new.iter().filter(|(id,v)|!old.get(*id).is_some_and(|before|same_instance(before,v))).map(|(_,v)|v.clone()).collect();
    (gone,added)
}
pub(crate) fn event(name:&str,v:&Visible,device:&Device)->Value {
    let mut payload=json!({"settings":v.settings,"coordinates":{"column":v.index%device.columns,"row":v.index/device.columns},"state":v.state,"isInMultiAction":false,"resources":{}});
    if matches!(name,"willAppear"|"willDisappear"|"didReceiveSettings"|"keyDown"|"keyUp") {payload["controller"]=json!("Keypad");}
    json!({"event":name,"action":v.action,"context":v.context,"device":device.id,"payload":payload})
}
pub(crate) fn title_event(v:&Visible,device:&Device,title:&str)->Value {
    json!({"event":"titleParametersDidChange","action":v.action,"context":v.context,"device":device.id,
        "payload":{"controller":"Keypad","settings":v.settings,"resources":{},"title":title,"titleParameters":{"fontFamily":"","fontSize":14,"fontStyle":"Regular","fontUnderline":false,"showTitle":true,"titleAlignment":"middle","titleColor":"#ffffff"},"state":v.state,"coordinates":{"column":v.index%device.columns,"row":v.index/device.columns}}})
}
fn settings_notifications(old:&HashMap<String,Visible>,new:&HashMap<String,Visible>,inspectors:&HashMap<String,Visible>,device:&Device,origin:Option<(&str,bool)>)->Vec<(String,Value)> {
    let mut result=Vec::new();
    for (context,v) in new {
        if !old.get(context).is_some_and(|previous|same_instance(previous,v) && previous.settings!=v.settings){continue;}
        let source=origin.filter(|(id,_)|*id==context).map(|(_,from_pi)|from_pi);
        if source!=Some(false) {result.push((v.plugin.clone(),event("didReceiveSettings",v,device)));}
        if source!=Some(true) {
            for (id,inspector) in inspectors {
                if same_instance(v,inspector) {
                    let mut message=event("didReceiveSettings",v,device);
                    message["__pi"]=json!(id);
                    result.push((v.plugin.clone(),message));
                }
            }
        }
    }
    result
}
/// ApplicationsToMonitor : compare les exécutables surveillés aux processus en cours (noms
/// insensibles à la casse) et produit applicationDidLaunch/Terminate, comme Stream Deck.
pub(crate) fn application_events(watch:&[String],running:&[String],before:&std::collections::BTreeSet<String>)->(std::collections::BTreeSet<String>,Vec<Value>){
    let now=watch.iter().filter(|w|running.iter().any(|r|r.eq_ignore_ascii_case(w))).cloned().collect::<std::collections::BTreeSet<_>>();
    let mut events=Vec::new();
    for app in now.difference(before){events.push(json!({"event":"applicationDidLaunch","payload":{"application":app}}));}
    for app in before.difference(&now){events.push(json!({"event":"applicationDidTerminate","payload":{"application":app}}));}
    (now,events)
}
enum Request { Sync(HashMap<String,Visible>,HashMap<String,(String,String,Value)>,Option<(String,bool)>), Virtual(Visible), VirtualSettings(String,Value), OpenVirtualInspector(Visible), CloseVirtualInspector(String), #[cfg(test)] SimulateInspectors(Sender<()>), #[cfg(test)] TestSessions(Sender<Vec<(String,String,u16)>>), Device(Device), Key(String,bool), Title(String,String), Send(String,Value), Open(String), Catalog(Vec<Plugin>), Pause(String,Sender<Result<(),String>>,std::sync::Arc<std::sync::atomic::AtomicBool>), Resume(String,Sender<Result<(),String>>), Stop }
static VIRTUAL_SENDER: std::sync::OnceLock<std::sync::Mutex<Option<Sender<Request>>>> = std::sync::OnceLock::new();
fn virtual_sender() -> &'static std::sync::Mutex<Option<Sender<Request>>> {
    VIRTUAL_SENDER.get_or_init(|| std::sync::Mutex::new(None))
}
/// Le moteur tourne sur un autre thread ; aucune exécution du plugin dans ce thread.
pub fn press_virtual(v: Visible) -> Result<(), String> {
    let tx = virtual_sender().lock().map_err(|e| e.to_string())?.clone().ok_or("Hôte de plugins indisponible")?;
    tx.send(Request::Virtual(v)).map_err(|_| "Hôte de plugins arrêté".into())
}
/// Contexte distinct des touches physiques, stable tant que touche et chemin de bloc ne changent pas.
pub fn virtual_context(key: &str, path: &[usize]) -> String {
    let hex = key.as_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>();
    format!("novydeck-block-{hex}-{}", path.iter().map(usize::to_string).collect::<Vec<_>>().join("."))
}
fn virtual_bindings(config: &Config) -> HashMap<String, (String,String,Value)> {
    fn walk(steps: &[crate::model::Step], key: &str, path: &[usize], out: &mut HashMap<String,(String,String,Value)>) {
        use crate::model::Step;
        for (index, step) in steps.iter().enumerate() {
            if step.is_disabled() { continue; }
            let mut p = path.to_vec(); p.push(index);
            match step {
                Step::PluginAction { plugin, action, settings, .. } => {out.insert(virtual_context(key,&p),(plugin.clone(),action.clone(),settings.clone()));},
                Step::Repeat { steps, .. } => {p.push(0);walk(steps,key,&p,out);},
                Step::Toggle { a,b } => {let mut branch=p.clone();branch.push(0);walk(a,key,&branch,out);p.push(1);walk(b,key,&p,out);},
                Step::IfCondition { then_steps,else_steps,.. } | Step::IfWindow { then_steps,else_steps,.. } | Step::IfVariable { then_steps,else_steps,.. } => {
                    let mut branch=p.clone();branch.push(0);walk(then_steps,key,&branch,out);p.push(1);walk(else_steps,key,&p,out);
                },
                _ => {},
            }
        }
    }
    let mut out=HashMap::new();
    let profile=&config.profiles[config.active];
    for (index,key) in config.keys().iter().enumerate() {
        walk(&key.steps,&crate::engine::identity(&profile.name,&config.page,index),&[],&mut out);
    }
    out
}
pub fn virtual_step_path(config:&Config,context:&str) -> Option<(usize,Vec<usize>)> {
    fn walk(steps:&[crate::model::Step], key:&str, parent:&[usize], context:&str) -> Option<Vec<usize>> {
        use crate::model::Step;
        for (i,step) in steps.iter().enumerate() {
            if step.is_disabled() {continue;}
            let mut path=parent.to_vec();path.push(i);
            if matches!(step,Step::PluginAction { .. }) && virtual_context(key,&path)==context {return Some(path);}
            let branches:Vec<&[Step]>=match step {
                Step::Repeat {steps,..}=>vec![steps], Step::Toggle {a,b}=>vec![a,b],
                Step::IfWindow {then_steps,else_steps,..}|Step::IfVariable {then_steps,else_steps,..}|Step::IfCondition {then_steps,else_steps,..}=>vec![then_steps,else_steps],
                _=>vec![],
            };
            for (branch,steps) in branches.into_iter().enumerate() {let mut nested=path.clone();nested.push(branch);if let Some(p)=walk(steps,key,&nested,context){return Some(p);}}
        }
        None
    }
    for (index,key) in config.keys().iter().enumerate() {
        let identity=crate::engine::identity(&config.profiles[config.active].name,&config.page,index);
        if let Some(path)=walk(&key.steps,&identity,&[],context){return Some((index,path));}
    }
    None
}
pub enum Reply { Message(String,Value), Error(String,String) }
/// Handle clonable pour l'opération disque sur un thread de travail. Ne garde pas le gestionnaire UI verrouillé.
#[derive(Clone)] pub struct PluginControl {tx:Sender<Request>}
impl PluginControl {
    fn command(&self,id:&str,pause:bool)->Result<(),String>{
        if id.is_empty(){return Err("UUID vide".into());}
        let (tx,rx)=mpsc::channel();
        let cancelled=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.tx.send(if pause{Request::Pause(id.into(),tx,cancelled.clone())}else{Request::Resume(id.into(),tx)}).map_err(|_|"Runtime arrêté".to_string())?;
        match rx.recv_timeout(Duration::from_secs(10)) {
            Ok(result)=>result,
            Err(_)=>{
                if pause {
                    cancelled.store(true,std::sync::atomic::Ordering::SeqCst);
                    // A pause completing at the timeout boundary must be undone as well.
                    let (ack,_rx)=mpsc::channel();let _=self.tx.send(Request::Resume(id.into(),ack));
                }
                Err("Acquittement runtime absent : ne pas modifier le disque".into())
            }
        }
    }
    /// Succès = processus ciblé et inspecteurs terminés ; échec/timeout = AUCUNE mutation disque permise.
    pub fn pause_plugin(&self,id:&str)->Result<(),String>{self.command(id,true)}
    /// À appeler après set_catalog (ou rollback), pour reprendre uniquement ce plugin.
    pub fn resume_plugin(&self,id:&str)->Result<(),String>{self.command(id,false)}
}
pub struct Manager { tx:Sender<Request>,pub rx:Receiver<Reply>,visible:HashMap<String,Visible>,titles:HashMap<String,String>,pub catalog:Vec<Plugin> }
impl Manager {
    pub fn new(plugins:Vec<Plugin>)->Self {
        let (tx,requests)=mpsc::channel();let (reply,rx)=mpsc::channel();
        let catalog=plugins.clone();thread::spawn(move||run(plugins,requests,reply));
        if let Ok(mut sender) = virtual_sender().lock() { *sender = Some(tx.clone()); }
        Self{tx,rx,visible:HashMap::new(),titles:HashMap::new(),catalog}
    }
    pub fn control(&self)->PluginControl{PluginControl{tx:self.tx.clone()}}
    pub fn pause_plugin(&self,id:&str)->Result<(),String>{self.control().pause_plugin(id)}
    pub fn resume_plugin(&self,id:&str)->Result<(),String>{self.control().resume_plugin(id)}
    pub fn device(&self,device:Device){let _=self.tx.send(Request::Device(device));}
    /// Plugins importés pendant l'exécution : visibles sans redémarrer NovyDeck.
    pub fn set_catalog(&mut self,plugins:Vec<Plugin>){self.catalog=plugins.clone();let _=self.tx.send(Request::Catalog(plugins));}
    pub fn sync(&mut self, config:&Config) {self.sync_with_origin(config,None);}
    pub fn sync_settings(&mut self,config:&Config,context:&str,from_pi:bool){self.sync_with_origin(config,Some((context.to_owned(),from_pi)));}
    fn sync_with_origin(&mut self, config:&Config, origin:Option<(String,bool)>) {
        let next=visible(config);
        let bindings=virtual_bindings(config);
        if next!=self.visible || !bindings.is_empty() {
            self.visible=next.clone();let _=self.tx.send(Request::Sync(next.clone(),bindings,origin));
        }
        let titles=next.iter().map(|(id,v)|(id.clone(),config.keys()[v.index].label.clone())).collect::<HashMap<_,_>>();
        for (id,title) in &titles {if self.titles.get(id)!=Some(title){let _=self.tx.send(Request::Title(id.clone(),title.clone()));}}
        self.titles=titles;
    }
    pub fn key(&self,index:usize,down:bool) {
        if let Some(v)=self.visible.values().find(|v|v.index==index){let _=self.tx.send(Request::Key(v.context.clone(),down));}
    }
    pub fn inspector_path(&self,index:usize)->Option<String>{let v=self.visible.values().find(|v|v.index==index)?;let p=self.catalog.iter().find(|p|p.uuid==v.plugin)?;p.actions.iter().find(|a|a.uuid==v.action).and_then(|a|a.inspector.clone()).or_else(||p.inspector.clone())}
    pub fn open_inspector(&self,index:usize){if let Some(v)=self.visible.values().find(|v|v.index==index){let _=self.tx.send(Request::Open(v.context.clone()));}}
    pub fn send(&self,plugin:&str,value:Value){let _=self.tx.send(Request::Send(plugin.to_owned(),value));}
    pub fn virtual_settings(&self,context:&str,settings:Value){let _=self.tx.send(Request::VirtualSettings(context.to_owned(),settings));}
    /// Enqueues a new block's PI; success does not imply that the WebView opened.
    /// The caller owns the unique `novydeck-inspector-block-` context until close.
    pub fn open_virtual_inspector(&self,v:Visible)->Result<(),String>{
        if !v.context.starts_with("novydeck-inspector-block-") || v.context.len()<"novydeck-inspector-block-".len()+1 || v.index>=Device::virtual_device().columns*Device::virtual_device().rows {
            return Err("Contexte d'inspecteur de bloc invalide".into());
        }
        self.tx.send(Request::OpenVirtualInspector(v)).map_err(|_|"Runtime arrêté".into())
    }
    pub fn close_virtual_inspector(&self,context:&str)->Result<(),String>{
        if !context.starts_with("novydeck-inspector-block-"){return Err("Contexte d'inspecteur de bloc invalide".into());}
        self.tx.send(Request::CloseVirtualInspector(context.into())).map_err(|_|"Runtime arrêté".into())
    }
    pub fn visible(&self)->&HashMap<String,Visible>{&self.visible}
}
impl Drop for Manager {fn drop(&mut self){let _=self.tx.send(Request::Stop);}}
fn run(mut plugins:Vec<Plugin>,requests:Receiver<Request>,reply:Sender<Reply>){
    let mut hosts:HashMap<String,Host>=HashMap::new();let mut current:HashMap<String,Visible>=HashMap::new();
    let mut configured:HashMap<String,Visible>=HashMap::new();
    let mut drafts:HashMap<String,Visible>=HashMap::new();
    #[cfg(test)] let mut simulate_inspectors=false;
    let mut device=Device::virtual_device();
    let mut inspectors:HashMap<String,Visible>=HashMap::new();
    let mut idle:HashMap<String,Instant>=HashMap::new();
    let mut restarts:HashMap<String,Vec<Instant>>=HashMap::new();
    let mut monitored:HashMap<String,std::collections::BTreeSet<String>>=HashMap::new();let mut last_poll:Option<Instant>=None;
    let mut paused=HashSet::<String>::new();
    loop {
        match requests.recv_timeout(Duration::from_millis(50)) {
            Ok(Request::Stop)|Err(mpsc::RecvTimeoutError::Disconnected)=>break,
            Ok(Request::Pause(id,ack,cancelled))=>{
                if cancelled.load(std::sync::atomic::Ordering::SeqCst) {continue;}
                paused.insert(id.clone());
                drafts.retain(|_,v|v.plugin!=id);
                current.retain(|ctx,v| !ctx.starts_with("novydeck-inspector-block-") || v.plugin!=id);
                configured.retain(|ctx,_|current.contains_key(ctx));
                inspectors.retain(|_,v|v.plugin!=id);
                monitored.remove(&id);idle.remove(&id);restarts.remove(&id);
                drop(hosts.remove(&id)); // Targeted child and inspectors only.
                if ack.send(Ok(())).is_err() || cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                    paused.remove(&id);
                    if current.values().any(|v|v.plugin==id) {
                        if let Some(p)=plugins.iter().find(|p|p.uuid==id) {
                            match Host::start(p,&device) {
                                Ok(h)=>{let _=h.send(device.event("deviceDidConnect"));for v in current.values().filter(|v|v.plugin==id){let _=h.send(event("willAppear",v,&device));}hosts.insert(id.clone(),h);},
                                Err(e)=>{let _=reply.send(Reply::Error(id.clone(),format!("Reprise après pause abandonnée impossible : {e}")));}
                            }
                        }
                    }
                }
            }
            Ok(Request::Resume(id,ack))=>{
                if !paused.remove(&id){let _=ack.send(Err("Plugin non suspendu".into()));continue;}
                let mut result=Ok(());
                if current.values().any(|v|v.plugin==id){
                    if let Some(p)=plugins.iter().find(|p|p.uuid==id){
                        match Host::start(p,&device){
                            Ok(h)=>{let _=h.send(device.event("deviceDidConnect"));for v in current.values().filter(|v|v.plugin==id){let _=h.send(event("willAppear",v,&device));}hosts.insert(id.clone(),h);},
                            Err(e)=>{let message=e.to_string();let _=reply.send(Reply::Error(id.clone(),message.clone()));result=Err(message);}
                        }
                    } else {result=Err("Plugin installé introuvable".into());}
                }
                let _=ack.send(result);
            }
            #[cfg(test)] Ok(Request::SimulateInspectors(ack))=>{simulate_inspectors=true;for host in hosts.values(){host.simulate_inspectors();}let _=ack.send(());}
            #[cfg(test)] Ok(Request::TestSessions(ack))=>{let _=ack.send(inspectors.iter().filter_map(|(id,v)|hosts.get(&v.plugin).map(|h|(id.clone(),v.context.clone(),h.ws_port))).collect());}
            Ok(Request::OpenVirtualInspector(mut v))=>{
                if !v.context.starts_with("novydeck-inspector-block-") || v.context.len()<"novydeck-inspector-block-".len()+1 || v.index>=device.columns*device.rows || current.contains_key(&v.context) && !drafts.contains_key(&v.context) {
                    let _=reply.send(Reply::Error(v.plugin.clone(),"Contexte de brouillon invalide ou déjà utilisé".into()));continue;
                }
                if paused.contains(&v.plugin){let _=reply.send(Reply::Error(v.plugin.clone(),"Plugin suspendu".into()));continue;}
                let Some(p)=plugins.iter().find(|p|p.uuid==v.plugin) else {let _=reply.send(Reply::Error(v.plugin.clone(),"Plugin installé introuvable".into()));continue;};
                let Some(action)=p.actions.iter().find(|a|a.uuid==v.action) else {let _=reply.send(Reply::Error(v.plugin.clone(),"Action introuvable".into()));continue;};
                let Some(path)=action.inspector.as_ref().or(p.inspector.as_ref()) else {let _=reply.send(Reply::Error(v.plugin.clone(),"Action sans inspecteur".into()));continue;};
                if let Some(previous)=drafts.get(&v.context) {
                    if previous.plugin!=v.plugin || previous.action!=v.action {let _=reply.send(Reply::Error(v.plugin.clone(),"Contexte déjà réservé à une autre action".into()));continue;}
                    // The active WebView already owns this context. Do not overwrite its settings.
                    if inspectors.iter().any(|(id,seen)|seen.context==v.context && hosts.get(&v.plugin).is_some_and(|h|h.inspector_active(id))) {continue;}
                    v.settings=previous.settings.clone();v.state=current.get(&v.context).map_or(v.state,|old|old.state);
                }
                if !hosts.contains_key(&v.plugin){match Host::start(p,&device){
                    Ok(h)=>{#[cfg(test)] if simulate_inspectors {h.simulate_inspectors();}let _=h.send(device.event("deviceDidConnect"));hosts.insert(v.plugin.clone(),h);},
                    Err(e)=>{let _=reply.send(Reply::Error(v.plugin.clone(),format!("Plugin indisponible : {e}")));continue;}
                }}
                let host=&hosts[&v.plugin];
                let info=json!({"action":v.action,"context":v.context,"device":device.id,"payload":{"settings":v.settings,"coordinates":{"column":v.index%device.columns,"row":v.index/device.columns}}});
                let title=crate::plugin_host::inspector_title(&p.name,&action.name);
                match host.open_inspector_with_title(path,info,&title){
                    Ok(id)=>{
                        if !current.contains_key(&v.context){reconcile_state(&mut v,None,&mut configured);let _=host.send(event("willAppear",&v,&device));}
                        idle.remove(&v.plugin);current.insert(v.context.clone(),v.clone());drafts.insert(v.context.clone(),v.clone());inspectors.insert(id,v);
                    },Err(e)=>{let _=reply.send(Reply::Error(v.plugin.clone(),format!("Inspecteur : {e}")));}
                }
            }
            Ok(Request::CloseVirtualInspector(context))=>{
                if let Some(v)=drafts.remove(&context){
                    if let Some(host)=hosts.get(&v.plugin){for (id,_) in inspectors.iter().filter(|(_,seen)|seen.context==context){host.close_inspector(id);}let _=host.send(event("willDisappear",&v,&device));}
                    inspectors.retain(|_,seen|seen.context!=context);current.remove(&context);configured.remove(&context);
                    if !current.values().any(|seen|seen.plugin==v.plugin){idle.insert(v.plugin,Instant::now());}
                }
            }
            Ok(Request::Title(context,title))=>{if let Some(v)=current.get(&context){if let Some(host)=hosts.get(&v.plugin){let _=host.send(title_event(v,&device,&title));}}}
            Ok(Request::Device(next))=>{if next!=device {for host in hosts.values(){let _=host.send(device.event("deviceDidDisconnect"));let _=host.send(next.event("deviceDidConnect"));}device=next;}}
            Ok(Request::Sync(mut next,bindings,origin))=>{
                for (context,(plugin,action,settings)) in bindings {
                    if let Some(v)=current.get(&context).filter(|v|v.plugin==plugin && v.action==action) {
                        let mut v=v.clone();v.settings=settings;
                        v.state=configured.get(&context).map_or(v.state,|c|c.state);
                        next.insert(context,v);
                    }
                }
                for (context,v) in &drafts {next.insert(context.clone(),current.get(context).cloned().unwrap_or_else(||v.clone()));}
                for v in next.values_mut(){let context=v.context.clone();reconcile_state(v,current.get(&context),&mut configured);}
                configured.retain(|context,_|next.contains_key(context));
                let notifications=settings_notifications(&current,&next,&inspectors,&device,origin.as_ref().map(|(id,from_pi)|(id.as_str(),*from_pi)));
                let (gone,added)=diff(&current,&next);
                for v in gone {if let Some(host)=hosts.get(&v.plugin){let _=host.send(event("willDisappear",&v,&device));}}
                for v in added {if paused.contains(&v.plugin){continue;}if !hosts.contains_key(&v.plugin){
                    if let Some(p)=plugins.iter().find(|p|p.uuid==v.plugin){match Host::start(p,&device){Ok(h)=>{let _=h.send(device.event("deviceDidConnect"));hosts.insert(v.plugin.clone(),h);},Err(e)=>{let _=reply.send(Reply::Error(v.plugin.clone(),e.to_string()));}}}
                    else{let _=reply.send(Reply::Error(v.plugin.clone(),"Plugin installé introuvable".into()));}
                }
                if let Some(host)=hosts.get(&v.plugin){let _=host.send(event("willAppear",&v,&device));}}
                current=next;
                let stale=inspectors.iter().filter(|(_,v)|!current.get(&v.context).is_some_and(|now|same_instance(v,now))).map(|(id,v)|(id.clone(),v.plugin.clone())).collect::<Vec<_>>();
                for (id,plugin) in stale {if let Some(host)=hosts.get(&plugin){host.close_inspector(&id);}inspectors.remove(&id);}
                for v in inspectors.values_mut(){if let Some(now)=current.get(&v.context){*v=now.clone();}}
                for (plugin,message) in notifications {if let Some(host)=hosts.get(&plugin){let _=host.send(message);}}
                for plugin in hosts.keys(){if current.values().any(|v|&v.plugin==plugin){idle.remove(plugin);}else{idle.entry(plugin.clone()).or_insert_with(Instant::now);}}
            }
            Ok(Request::Virtual(mut v))=>{
                if paused.contains(&v.plugin){let _=reply.send(Reply::Error(v.plugin.clone(),"Plugin suspendu".into()));continue;}
                let Some(p)=plugins.iter().find(|p|p.uuid==v.plugin && p.actions.iter().any(|a|a.uuid==v.action)) else {
                    let _=reply.send(Reply::Error(v.plugin.clone(),format!("Action de plugin introuvable ou désactivée : {}",v.action)));continue;
                };
                if !hosts.contains_key(&v.plugin) {
                    match Host::start(p,&device) {
                        Ok(h)=>{let _=h.send(device.event("deviceDidConnect"));hosts.insert(v.plugin.clone(),h);},
                        Err(e)=>{let _=reply.send(Reply::Error(v.plugin.clone(),format!("Action de plugin indisponible : {e}")));continue;}
                    }
                }
                let old=current.get(&v.context).cloned();
                if let Some(old)=old.as_ref().filter(|old|old.plugin!=v.plugin || old.action!=v.action) {
                    if let Some(host)=hosts.get(&old.plugin){let _=host.send(event("willDisappear",old,&device));}
                }
                if let Some(old)=old.as_ref().filter(|old|old.plugin==v.plugin && old.action==v.action) {
                    v.settings=old.settings.clone();
                    v.state=configured.get(&v.context).map_or(v.state,|c|c.state);
                }
                reconcile_state(&mut v,old.as_ref(),&mut configured);
                let host=&hosts[&v.plugin];
                if !old.as_ref().is_some_and(|old|same_instance(old,&v)) {let _=host.send(event("willAppear",&v,&device));}
                for name in ["keyDown","keyUp"] {
                    if let Err(e)=host.send(event(name,&v,&device)) {let _=reply.send(Reply::Error(v.plugin.clone(),format!("Action de plugin non envoyée : {e}")));break;}
                }
                idle.remove(&v.plugin);
                current.insert(v.context.clone(),v);
            }
            Ok(Request::VirtualSettings(context,settings))=>{
                if let Some(v)=current.get_mut(&context) {
                    let before=v.clone();
                    if v.settings!=settings {v.state=configured.get(&context).map_or(v.state,|c|c.state);}
                    v.settings=settings.clone();
                    if let Some(default)=configured.get_mut(&context){default.settings=settings;}
                    if drafts.contains_key(&context) {
                        drafts.insert(context.clone(),v.clone());
                        for (plugin,message) in settings_notifications(&HashMap::from([(context.clone(),before)]),&HashMap::from([(context.clone(),v.clone())]),&inspectors,&device,None){if let Some(host)=hosts.get(&plugin){let _=host.send(message);}}
                    }
                }
            }
            Ok(Request::Open(context))=>{if let Some(v)=current.get(&context){if let (Some(p),Some(host))=(plugins.iter().find(|p|p.uuid==v.plugin),hosts.get(&v.plugin)){
                let path=p.actions.iter().find(|a|a.uuid==v.action).and_then(|a|a.inspector.as_ref()).or(p.inspector.as_ref());
                if let Some(path)=path {let action_info=json!({"action":v.action,"context":v.context,"device":device.id,"payload":{"settings":v.settings,"coordinates":{"column":v.index%device.columns,"row":v.index/device.columns}}});
                    let title=crate::plugin_host::inspector_title(&p.name,&p.actions.iter().find(|a|a.uuid==v.action).map(|a|a.name.as_str()).unwrap_or(""));
                    match host.open_inspector_with_title(path,action_info,&title){Ok(id)=>{inspectors.insert(id,v.clone());},Err(e)=>{let _=reply.send(Reply::Error(v.plugin.clone(),format!("Inspecteur : {e}")));}}
                }
            }}}
            Ok(Request::Key(context,down))=>{if let Some(v)=current.get(&context){if let Some(host)=hosts.get(&v.plugin){let _=host.send(event(if down{"keyDown"}else{"keyUp"},v,&device));}}}
            Ok(Request::Send(plugin,value))=>{if let Some(host)=hosts.get(&plugin){
                if value["__pi"].as_str()==Some("*") {
                    for (id,v) in &inspectors {if v.plugin==plugin {let mut m=value.clone();m["__pi"]=json!(id);let _=host.send(m);}}
                } else { let _=host.send(value); }
            }}
            Ok(Request::Catalog(next))=>{
                plugins=next;
                // Une touche liée avant l'import démarre maintenant, avec son willAppear.
                let waiting=current.values().filter(|v|!paused.contains(&v.plugin) && !hosts.contains_key(&v.plugin)).map(|v|v.plugin.clone()).collect::<std::collections::BTreeSet<_>>();
                for id in waiting {if let Some(p)=plugins.iter().find(|p|p.uuid==id){match Host::start(p,&device){
                    Ok(h)=>{let _=h.send(device.event("deviceDidConnect"));for v in current.values().filter(|v|v.plugin==id){let _=h.send(event("willAppear",v,&device));}hosts.insert(id,h);},
                    Err(e)=>{let _=reply.send(Reply::Error(id,e.to_string()));}
                }}}
            }
            Err(mpsc::RecvTimeoutError::Timeout)=>{}
        }
        for (plugin,host) in &hosts {while let Ok(mut value)=host.incoming.try_recv(){
            let name=value["event"].as_str().unwrap_or("").to_owned();
            if name=="__hostError" {let _=reply.send(Reply::Error(plugin.clone(),value["reason"].as_str().unwrap_or("Message invalide").into()));continue;}
            // Commandes des molettes/écrans tactiles : sans objet pour une touche, ignorées sans erreur.
            if matches!(name.as_str(),"setFeedback"|"setFeedbackLayout"|"setTriggerDescription") {continue;}
            if !matches!(name.as_str(),"registerPlugin"|"logMessage"|"setTitle"|"setImage"|"setState"|"novydeckStateUnknown"|"showOk"|"showAlert"|"setSettings"|"getSettings"|"setGlobalSettings"|"getGlobalSettings"|"sendToPlugin"|"sendToPropertyInspector"|"switchToProfile"|"propertyInspectorDidAppear"|"propertyInspectorDidDisappear"|"openUrl") {
                let _=reply.send(Reply::Error(plugin.clone(),format!("Evenement plugin non pris en charge : {name}")));continue;
            }
            if let Some(id)=value["__pi"].as_str().map(str::to_owned){
                let Some(v)=inspectors.get(&id).filter(|v|v.plugin==*plugin && host.inspector_active(&id)).cloned() else {inspectors.remove(&id);continue};
                if !value.get("action").is_none_or(|action|action.as_str()==Some(&v.action)) {
                    let _=reply.send(Reply::Error(plugin.clone(),"Action étrangère".into()));continue;
                }
                if name=="propertyInspectorDidDisappear" {inspectors.remove(&id);}
                if name=="propertyInspectorDidAppear" {
                    // Comme Stream Dock/Stream Deck : le PI reçoit tout de suite les réglages de sa touche.
                    let _=host.send(json!({"event":"didReceiveSettings","action":v.action,"context":v.context,"device":device.id,"__pi":id,
                        "payload":event("didReceiveSettings",&v,&device)["payload"]}));
                }
                if !matches!(name.as_str(),"setState"|"novydeckStateUnknown"|"setSettings"|"getSettings"|"setGlobalSettings"|"getGlobalSettings"|"sendToPlugin"|"propertyInspectorDidAppear"|"propertyInspectorDidDisappear"){continue;}
                value["context"]=json!(v.context);value["action"]=json!(v.action);
                if name=="sendToPlugin" || name.starts_with("propertyInspectorDid"){
                    let mut command=value.clone();if let Some(obj)=command.as_object_mut(){obj.remove("__pi");}
                    let _=host.send(command);continue;
                }
            } else if name=="sendToPropertyInspector" {
                if let Some(ctx)=value["context"].as_str(){if current.get(ctx).is_some_and(|v|v.plugin==*plugin && value.get("action").is_none_or(|a|a.as_str()==Some(&v.action))){for (id,v) in &inspectors {if v.context==ctx && v.plugin==*plugin && host.inspector_active(id) {let mut message=value.clone();message["action"]=json!(v.action);message["__pi"]=json!(id);let _=host.send(message);}}}}continue;
            }
            // PI messages carry a session ID, not necessarily a canonical action context.
            // Validate that session and rewrite its context above BEFORE checking ownership.
            if matches!(name.as_str(),"setTitle"|"setImage"|"setState"|"novydeckStateUnknown"|"showOk"|"showAlert"|"setSettings"|"getSettings")
                && !current.get(value["context"].as_str().unwrap_or("")).is_some_and(|v:&Visible|v.plugin==*plugin) {
                let _=reply.send(Reply::Error(plugin.clone(),"Contexte étranger ou absent".into()));continue;
            }
            if name=="getSettings" {
                if let Some(v)=current.get(value["context"].as_str().unwrap_or("")) {
                    let mut message=event("didReceiveSettings",v,&device);
                    if let Some(id)=value["__pi"].as_str(){message["__pi"]=json!(id);}
                    let _=host.send(message);
                }
            }
            if name=="setSettings" {
                if let Some(context)=value["context"].as_str().map(str::to_owned) {
                    if drafts.contains_key(&context){
                        if let Some(v)=current.get_mut(&context){
                            let before=v.clone();v.settings=value["payload"].clone();
                            if before.settings!=v.settings{v.state=configured.get(&context).map_or(v.state,|c|c.state);}
                            if let Some(default)=configured.get_mut(&context){default.settings=v.settings.clone();}
                            drafts.insert(context.clone(),v.clone());
                            let origin=value.get("__pi").map(|_|(context.as_str(),true)).or(Some((context.as_str(),false)));
                            for (owner,message) in settings_notifications(&HashMap::from([(context.clone(),before)]),&HashMap::from([(context.clone(),v.clone())]),&inspectors,&device,origin){if let Some(h)=hosts.get(&owner){let _=h.send(message);}}
                        }
                    }
                }
            }
            if matches!(name.as_str(),"setState"|"novydeckStateUnknown") {
                let Some(v)=state_owner(&current,plugin,&value) else {
                    let _=reply.send(Reply::Error(plugin.clone(),"Action ou contexte étranger".into()));continue;
                };
                let context=v.context.clone();
                if name=="novydeckStateUnknown" {
                    if let (Some(now),Some(default))=(current.get_mut(&context),configured.get(&context)) {now.state=default.state;}
                } else if let Some(state)=value["payload"]["state"].as_u64().filter(|s|*s<=u8::MAX as u64) {
                    if let Some(now)=current.get_mut(&context){now.state=state as u8;}
                }
            }
            if name=="switchToProfile" {let _=reply.send(Reply::Error(plugin.clone(),"switchToProfile non pris en charge : profil non vérifié".into()));continue;}
            let _=reply.send(Reply::Message(plugin.clone(),value));
        }}
        // A draft is alive only while one of its PI sessions remains valid.
        let invalid=drafts.iter().filter(|(context,v)|!inspectors.iter().any(|(id,seen)|seen.context==**context && hosts.get(&v.plugin).is_some_and(|host|host.inspector_active(id)))).map(|(context,_)|context.clone()).collect::<Vec<_>>();
        for context in invalid {if let Some(v)=drafts.remove(&context){if let Some(host)=hosts.get(&v.plugin){let _=host.send(event("willDisappear",&v,&device));}inspectors.retain(|_,seen|seen.context!=context);current.remove(&context);configured.remove(&context);}}
        let exited=hosts.iter_mut().filter_map(|(id,host)|{
            if current.values().any(|v|v.plugin==*id) && host.exited() {Some(id.clone())}else{None}
        }).collect::<Vec<_>>();
        for id in exited {
            let _=reply.send(Reply::Error(id.clone(),"Plugin termine inopinement; redemarrage tente".into()));
            hosts.remove(&id);inspectors.retain(|_,v|v.plugin!=id); // Drop kills any remaining descendants before restart.
            let attempts=restarts.entry(id.clone()).or_default();
            attempts.retain(|when|when.elapsed()<Duration::from_secs(300));
            if attempts.len()>=3 {
                let _=reply.send(Reply::Error(id.clone(),"Erreur : plugin arrêté après 3 redémarrages en 5 minutes".into()));
                continue;
            }
            attempts.push(Instant::now());
            if let Some(p)=plugins.iter().find(|p|p.uuid==id) {
                match Host::start(p,&device) {
                    Ok(host)=>{
                        let _=host.send(device.event("deviceDidConnect"));
                        for v in current.values().filter(|v|v.plugin==id) {let _=host.send(event("willAppear",v,&device));}
                        hosts.insert(id.clone(),host);
                    },
                    Err(e)=>{let _=reply.send(Reply::Error(id.clone(),format!("Erreur : redémarrage impossible : {e}")));}
                }
            }
        }
        monitored.retain(|id,_|hosts.contains_key(id));
        if last_poll.is_none_or(|t|t.elapsed()>=Duration::from_secs(2)) && hosts.keys().any(|id|plugins.iter().any(|p|&p.uuid==id&&!p.applications.is_empty())) {
            last_poll=Some(Instant::now());
            #[cfg(windows)] let running=crate::conditions::running_processes().map(|list|list.into_iter().map(|(_,name)|name).collect::<Vec<_>>()).unwrap_or_default();
            #[cfg(not(windows))] let running:Vec<String>=Vec::new();
            for (id,host) in &hosts {if let Some(p)=plugins.iter().find(|p|&p.uuid==id&&!p.applications.is_empty()){
                let before=monitored.remove(id).unwrap_or_default();
                let (now,events)=application_events(&p.applications,&running,&before);
                for message in events {let _=host.send(message);}
                monitored.insert(id.clone(),now);
            }}
        }
        hosts.retain(|id,_|!idle.get(id).is_some_and(|t|t.elapsed()>=Duration::from_secs(30)));
        idle.retain(|id,_|hosts.contains_key(id));
    }
    // Host::drop termine tout l'arbre de chaque processus.
}
#[cfg(test)]mod tests {use super::*;use crate::model::{Config,PluginBinding};
    #[test] fn draft_inspectors_are_independent_and_survive_sync_without_keypress() {
        use tungstenite::Message;
        let fixture=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sdplugin");
        let mut plugin=crate::plugins::parse(&fixture,&std::fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
        let dir=tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("pi.html"),"<html></html>").unwrap();
        std::fs::write(dir.path().join("witness.js"),r#"
const args=process.argv.slice(2),val=k=>args[args.indexOf(k)+1];
const ws=new WebSocket(`ws://127.0.0.1:${val('-port')}`);
ws.addEventListener('open',()=>ws.send(JSON.stringify({event:val('-registerEvent'),uuid:val('-pluginUUID')})));
ws.addEventListener('message',m=>{let v=JSON.parse(m.data);if(v.event==='willAppear'||v.event==='willDisappear'||v.event==='keyDown'||v.event==='didReceiveSettings')ws.send(JSON.stringify({event:'logMessage',context:v.context,payload:{message:v.event,settings:v.payload.settings}}));});
"#).unwrap();
        plugin.folder=dir.path().into();plugin.runtime=crate::plugins::Runtime::Node(dir.path().join("witness.js"));plugin.actions[0].inspector=Some("pi.html".into());
        let manager=Manager::new(vec![plugin.clone()]);
        let (ack,wait)=mpsc::channel();manager.tx.send(Request::SimulateInspectors(ack)).unwrap();wait.recv_timeout(Duration::from_secs(2)).unwrap();
        let contexts=["novydeck-inspector-block-a","novydeck-inspector-block-b"];
        let sessions=||{let (tx,rx)=mpsc::channel();manager.tx.send(Request::TestSessions(tx)).unwrap();rx.recv_timeout(Duration::from_secs(3)).unwrap()};
        for (i,ctx) in contexts.iter().enumerate(){manager.open_virtual_inspector(Visible{context:(*ctx).into(),plugin:plugin.uuid.clone(),action:plugin.actions[0].uuid.clone(),index:i,settings:if i==0 {Value::Null}else{json!({"value":2})},state:0}).unwrap();}
        let initial=sessions();assert_eq!(initial.len(),2);
        let (first,_,port)=initial.iter().find(|(_,ctx,_)|ctx==contexts[0]).unwrap().clone();
        let (second,_,_)=initial.iter().find(|(_,ctx,_)|ctx==contexts[1]).unwrap().clone();
        // A test seam simulates ONLY WebView spawn; real Host HTTP path, registry, random sessions and sockets are used.
        let mut peers=Vec::new();
        for id in [&first,&second] {let (mut ws,_)=tungstenite::connect(format!("ws://127.0.0.1:{port}")).unwrap();ws.send(Message::Text(json!({"event":"registerPropertyInspector","uuid":id}).to_string().into())).unwrap();peers.push(ws);}
        peers[0].send(Message::Text(json!({"event":"setSettings","action":"foreign.action","payload":{"rejected":true}}).to_string().into())).unwrap();
        for (index,settings) in [(0,json!({"x":10})),(1,json!({"x":20}))] {
            peers[index].send(Message::Text(json!({"event":"setSettings","context":"wrong","payload":settings}).to_string().into())).unwrap();
        }
        let until=Instant::now()+Duration::from_secs(12);let mut changed=HashMap::new();let mut appeared=HashSet::new();let mut foreign_rejected=false;
        while Instant::now()<until && (changed.len()<2 || appeared.len()<2 || !foreign_rejected) {
            match manager.rx.recv_timeout(Duration::from_millis(200)) {
                Ok(Reply::Message(_,v)) if v["event"]=="setSettings"=>{changed.insert(v["context"].as_str().unwrap().to_owned(),v["payload"].clone());},
                Ok(Reply::Message(_,v)) if v["event"]=="logMessage" && v["payload"]["message"]=="willAppear"=>{appeared.insert(v["context"].as_str().unwrap().to_owned());},
                Ok(Reply::Message(_,v)) if v["event"]=="logMessage"=>{assert_ne!(v["payload"]["message"],"keyDown");},
                Ok(Reply::Error(_,e)) if e.contains("Action étrangère")=>{foreign_rejected=true;},
                Ok(Reply::Error(_,e))=>panic!("unexpected: {e}"),_=>{}
            }
        }
        assert!(foreign_rejected);assert_eq!(appeared.len(),2);assert_eq!(changed[contexts[0]],json!({"x":10}));assert_eq!(changed[contexts[1]],json!({"x":20}));
        let mut config=Config::default();manager.tx.send(Request::Sync(visible(&config),virtual_bindings(&config),None)).unwrap();
        assert_eq!(sessions().len(),2,"sync with no applied blocks must preserve drafts");
        manager.open_virtual_inspector(Visible{context:contexts[0].into(),plugin:plugin.uuid.clone(),action:plugin.actions[0].uuid.clone(),index:0,settings:Value::Null,state:0}).unwrap();
        assert_eq!(sessions().len(),2,"repeated open must reuse existing PI");
        manager.close_virtual_inspector(contexts[0]).unwrap();assert_eq!(sessions().len(),1);
        peers[0].send(Message::Text(json!({"event":"setSettings","payload":{"stale":true}}).to_string().into())).ok();
        manager.open_virtual_inspector(Visible{context:contexts[0].into(),plugin:plugin.uuid.clone(),action:plugin.actions[0].uuid.clone(),index:0,settings:Value::Null,state:0}).unwrap();
        let reopened=sessions();assert_eq!(reopened.len(),2);assert!(!reopened.iter().any(|(id,_,_)|id==&first));assert!(reopened.iter().any(|(id,_,_)|id==&second));
        for i in 2..4 {manager.open_virtual_inspector(Visible{context:format!("novydeck-inspector-block-{i}"),plugin:plugin.uuid.clone(),action:plugin.actions[0].uuid.clone(),index:i,settings:Value::Null,state:0}).unwrap();}
        assert_eq!(sessions().len(),4);
        manager.open_virtual_inspector(Visible{context:"novydeck-inspector-block-overflow".into(),plugin:plugin.uuid.clone(),action:plugin.actions[0].uuid.clone(),index:4,settings:Value::Null,state:0}).unwrap();
        assert_eq!(sessions().len(),4);
        let deadline=Instant::now()+Duration::from_secs(4);let mut limited=false;
        while Instant::now()<deadline {if let Ok(Reply::Error(_,e))=manager.rx.recv_timeout(Duration::from_millis(100)){if e.contains("quatre inspecteurs"){limited=true;break;}}}
        assert!(limited);
        config.page="main".into(); // no applied plugin binding or physical key was required
    }
    #[test] fn pause_targets_one_plugin_and_blocks_virtual_restart() {
        let folder=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sdplugin");
        let first=crate::plugins::parse(&folder,&std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
        let mut second=first.clone();second.uuid="test.other.fixture".into();
        let mut manager=Manager::new(vec![first.clone(),second.clone()]);
        let mut config=Config::default();
        for (index,p) in [first.clone(),second.clone()].iter().enumerate(){
            config.keys_mut()[index].plugin=Some(PluginBinding{plugin:p.uuid.clone(),action:p.actions[0].uuid.clone(),settings:json!({}),state:0});
        }
        manager.sync(&config);
        let until=Instant::now()+Duration::from_secs(20);let mut registered=0;
        while registered<2 && Instant::now()<until {
            if let Ok(Reply::Message(_,value))=manager.rx.recv_timeout(Duration::from_millis(200)){
                if value["event"]=="registerPlugin" {registered+=1;}
            }
        }
        assert_eq!(registered,2,"both fixtures must be active before pause");
        let control=manager.control();control.pause_plugin(&first.uuid).unwrap();
        let first_visible=manager.visible().values().find(|v|v.plugin==first.uuid).unwrap().clone();
        let second_visible=manager.visible().values().find(|v|v.plugin==second.uuid).unwrap().clone();
        manager.tx.send(Request::Virtual(first_visible)).unwrap();
        assert!(matches!(manager.rx.recv_timeout(Duration::from_secs(3)),Ok(Reply::Error(id,e)) if id==first.uuid && e.contains("suspendu")));
        manager.tx.send(Request::Virtual(second_visible)).unwrap();
        let until=Instant::now()+Duration::from_secs(6);let mut survived=false;
        while Instant::now()<until {if let Ok(Reply::Message(id,value))=manager.rx.recv_timeout(Duration::from_millis(200)) {
            if id==second.uuid && value["event"]=="setTitle" {survived=true;break;}
        }}
        assert!(survived,"other plugin must keep responding");
        control.resume_plugin(&first.uuid).unwrap();
    }
    #[test] fn resume_reports_start_failure_to_caller() {
        let folder=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sdplugin");
        let first=crate::plugins::parse(&folder,&std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
        let mut second=first.clone();second.uuid="test.other.fixture".into();
        let mut manager=Manager::new(vec![first.clone(),second.clone()]);
        let mut config=Config::default();
        for (index,p) in [&first,&second].iter().enumerate(){config.keys_mut()[index].plugin=Some(PluginBinding{plugin:p.uuid.clone(),action:p.actions[0].uuid.clone(),settings:json!({}),state:0});}
        manager.sync(&config);
        let until=Instant::now()+Duration::from_secs(20);let mut registered=0;
        while registered<2 && Instant::now()<until {if let Ok(Reply::Message(_,v))=manager.rx.recv_timeout(Duration::from_millis(200)){if v["event"]=="registerPlugin"{registered+=1;}}}
        assert_eq!(registered,2);
        let control=manager.control();control.pause_plugin(&first.uuid).unwrap();
        let mut broken=first.clone();broken.runtime=crate::plugins::Runtime::Exe(folder.join("nonexistent-runtime-code.exe"));
        manager.set_catalog(vec![broken,second.clone()]);
        assert!(control.resume_plugin(&first.uuid).is_err(),"failed start must fail ack");
        let v=manager.visible().values().find(|v|v.plugin==second.uuid).unwrap().clone();
        manager.tx.send(Request::Virtual(v)).unwrap();
        let until=Instant::now()+Duration::from_secs(6);let mut survived=false;
        while Instant::now()<until {if let Ok(Reply::Message(id,value))=manager.rx.recv_timeout(Duration::from_millis(200)){if id==second.uuid && value["event"]=="setTitle" {survived=true;break;}}}
        assert!(survived,"other plugin remains alive");
    }
    #[test] fn abandoned_pause_ack_does_not_suspend_plugin() {
        let folder=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sdplugin");
        let first=crate::plugins::parse(&folder,&std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
        let mut second=first.clone();second.uuid="test.other.fixture".into();
        let mut manager=Manager::new(vec![first.clone(),second.clone()]);
        let mut config=Config::default();for (index,p) in [&first,&second].iter().enumerate(){config.keys_mut()[index].plugin=Some(PluginBinding{plugin:p.uuid.clone(),action:p.actions[0].uuid.clone(),settings:json!({}),state:0});}
        manager.sync(&config);
        let until=Instant::now()+Duration::from_secs(20);let mut registered=0;
        while registered<2 && Instant::now()<until {if let Ok(Reply::Message(_,v))=manager.rx.recv_timeout(Duration::from_millis(200)){if v["event"]=="registerPlugin"{registered+=1;}}}
        assert_eq!(registered,2);
        let (tx,rx)=mpsc::channel();drop(rx);manager.tx.send(Request::Pause(first.uuid.clone(),tx,std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)))).unwrap();
        let (barrier,wait)=mpsc::channel();manager.tx.send(Request::Resume(first.uuid.clone(),barrier)).unwrap();
        assert!(wait.recv_timeout(Duration::from_secs(10)).unwrap().is_err(),"abandoned pause must not persist");
        for p in [&first,&second] {let v=manager.visible().values().find(|v|v.plugin==p.uuid).unwrap().clone();manager.tx.send(Request::Virtual(v)).unwrap();}
        let until=Instant::now()+Duration::from_secs(6);let mut seen=HashSet::new();
        while Instant::now()<until && seen.len()<2 {if let Ok(Reply::Message(id,value))=manager.rx.recv_timeout(Duration::from_millis(200)){if value["event"]=="setTitle"{seen.insert(id);}}}
        assert!(seen.contains(&first.uuid) && seen.contains(&second.uuid));
    }
    #[test]fn applications_launch_and_terminate_once(){
        let watch=vec!["OBS64.exe".to_string(),"Spotify.exe".to_string()];
        let (now,events)=application_events(&watch,&["obs64.exe".into(),"explorer.exe".into()],&Default::default());
        assert_eq!(events,vec![json!({"event":"applicationDidLaunch","payload":{"application":"OBS64.exe"}})]);
        let (same,none)=application_events(&watch,&["obs64.exe".into()],&now);assert!(none.is_empty());
        let (_,gone)=application_events(&watch,&[],&same);
        assert_eq!(gone,vec![json!({"event":"applicationDidTerminate","payload":{"application":"OBS64.exe"}})]);
    }
    #[test]fn plugin_imported_at_runtime_starts_without_restart(){
        let folder=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sdplugin");
        let plugin=crate::plugins::parse(&folder,&std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
        let mut manager=Manager::new(Vec::new());
        let mut config=Config::default();
        config.keys_mut()[0].plugin=Some(PluginBinding{plugin:plugin.uuid.clone(),action:plugin.actions[0].uuid.clone(),settings:json!({}),state:0});
        manager.sync(&config);
        assert!(matches!(manager.rx.recv_timeout(Duration::from_secs(5)).unwrap(),Reply::Error(_,ref e) if e.contains("introuvable")));
        manager.set_catalog(vec![plugin.clone()]);
        assert_eq!(manager.catalog.len(),1);
        let deadline=Instant::now()+Duration::from_secs(20);
        loop {
            match manager.rx.recv_timeout(deadline.saturating_duration_since(Instant::now())).expect("imported plugin did not register") {
                Reply::Message(id,value) if value["event"]=="registerPlugin" => {assert_eq!(id,plugin.uuid);break;}
                Reply::Error(id,e)=>panic!("{id}: {e}"),
                _=>{}
            }
        }
    }
    #[test]fn page_diff(){let mut c=Config::default();c.keys_mut()[0].plugin=Some(PluginBinding{plugin:"fixture".into(),action:"fixture.a".into(),settings:json!({}),state:0});let old=visible(&c);c.profiles[c.active].pages.push(crate::model::Page::new("other".into(),"Other".into(),false));c.page="other".into();let next=visible(&c);let (gone,added)=diff(&old,&next);assert_eq!(gone.len(),1);assert!(added.is_empty());}
    #[test]fn settings_update_preserves_action_visibility(){
        let mut config=Config::default();
        config.keys_mut()[0].plugin=Some(PluginBinding{plugin:"fixture".into(),action:"fixture.action".into(),settings:json!({"count":0}),state:0});
        let before=visible(&config);
        config.keys_mut()[0].plugin.as_mut().unwrap().settings=json!({"count":1});
        let after=visible(&config);
        let (gone,added)=diff(&before,&after);
        assert!(gone.is_empty(),"updating settings must not emit willDisappear");
        assert!(added.is_empty(),"updating settings must not emit willAppear");
    }
    #[test]fn settings_update_targets_only_adjacent_socket(){
        let mut config=Config::default();
        config.keys_mut()[0].plugin=Some(PluginBinding{plugin:"fixture".into(),action:"fixture.action".into(),settings:json!({"count":0}),state:0});
        let before=visible(&config);
        let context=before.keys().next().unwrap().clone();
        let inspectors=HashMap::from([("pi-a".to_owned(),before[&context].clone())]);
        config.keys_mut()[0].plugin.as_mut().unwrap().settings=json!({"count":1});
        let after=visible(&config);
        let device=Device::virtual_device();
        let from_plugin=settings_notifications(&before,&after,&inspectors,&device,Some((&context,false)));
        assert_eq!(from_plugin.len(),1);
        assert_eq!(from_plugin[0].0,"fixture");
        assert_eq!(from_plugin[0].1["__pi"],"pi-a");
        assert_eq!(from_plugin[0].1["event"],"didReceiveSettings");
        assert_eq!(from_plugin[0].1["payload"]["settings"],json!({"count":1}));
        let from_pi=settings_notifications(&before,&after,&inspectors,&device,Some((&context,true)));
        assert_eq!(from_pi.len(),1);
        assert_eq!(from_pi[0].0,"fixture");
        assert!(from_pi[0].1.get("__pi").is_none());
        assert!(settings_notifications(&after,&after,&inspectors,&device,None).is_empty());
    }
    #[test]fn sdk_device_and_payload_contract(){
        let d=Device::from_grid("Elgato Stream Deck XL".into(),8,4);
        let v=Visible{context:"x".into(),plugin:"p".into(),action:"a".into(),index:17,settings:json!({"n":1}),state:1};
        let connect=d.event("deviceDidConnect");
        assert_eq!(connect["deviceInfo"]["size"],json!({"columns":8,"rows":4}));
        assert_eq!(connect["device"],d.id);
        for name in ["willAppear","willDisappear","didReceiveSettings","keyDown","keyUp"] {
            let msg=event(name,&v,&d);assert_eq!(msg["device"],connect["device"]);
            assert_eq!(msg["payload"]["coordinates"],json!({"column":1,"row":2}));
            assert_eq!(msg["payload"]["settings"],json!({"n":1}));
            assert_eq!(msg["payload"]["resources"],json!({}));
            assert_eq!(msg["payload"]["isInMultiAction"],false);
            assert_eq!(msg["payload"]["state"],1);
        }
        for name in ["willAppear","willDisappear","didReceiveSettings","keyDown","keyUp"] { assert_eq!(event(name,&v,&d)["payload"]["controller"],"Keypad"); }
        assert_eq!(Device::virtual_device().info()["size"]["columns"],5);
    }
    #[test]fn configured_title_event(){
        let v=Visible{context:"ctx".into(),plugin:"p".into(),action:"a".into(),index:7,settings:json!({}),state:1};
        let msg=title_event(&v,&Device::virtual_device(),"Titre configuré");
        assert_eq!(msg["event"],"titleParametersDidChange");
        assert_eq!(msg["payload"]["title"],"Titre configuré");
        assert_eq!(msg["payload"]["coordinates"],json!({"column":2,"row":1}));
        assert_eq!(msg["payload"]["controller"],"Keypad");
        assert_eq!(msg["payload"]["settings"],json!({}));
        assert_eq!(msg["payload"]["resources"],json!({}));
        assert_eq!(msg["payload"]["state"],1);
        assert!(msg["payload"]["titleParameters"].is_object());
    }
    #[test]fn event_coordinates(){let v=Visible{context:"ctx".into(),plugin:"p".into(),action:"a".into(),index:7,settings:json!({"x":1}),state:2};let message=event("keyDown",&v,&Device::virtual_device());assert_eq!(message["payload"]["coordinates"],json!({"column":2,"row":1}));assert_eq!(message["context"],"ctx");}
    #[test] fn configured_state_resets_on_binding_changes_and_rejects_other_owner() {
        let mut defaults=HashMap::new();
        let base=Visible{context:"ctx".into(),plugin:"p".into(),action:"a".into(),index:0,settings:json!({"x":0}),state:0};
        let mut initial=base.clone();reconcile_state(&mut initial,None,&mut defaults);
        let mut reported=initial.clone();reported.state=1;
        let mut same=base.clone();reconcile_state(&mut same,Some(&reported),&mut defaults);assert_eq!(same.state,1);
        for changed in [Visible{state:2,..base.clone()},Visible{settings:json!({"x":1}),..base.clone()},Visible{action:"b".into(),..base.clone()},Visible{plugin:"other".into(),..base.clone()}] {
            let mut config=HashMap::from([("ctx".into(),base.clone())]);
            let mut next=changed.clone();reconcile_state(&mut next,Some(&reported),&mut config);
            assert_eq!(next.state,changed.state);
        }
        let current=HashMap::from([("ctx".into(),base)]);
        assert!(state_owner(&current,"p",&json!({"context":"ctx"})).is_some());
        for (id,message) in [("other",json!({"context":"ctx"})),("p",json!({"context":"missing"})),("p",json!({"context":"ctx","action":"b"}))] {
            assert!(state_owner(&current,id,&message).is_none());
        }
    }
    #[test] fn reported_state_survives_sync_and_virtual_until_unknown() {
        let folder=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sdplugin");
        let mut plugin=crate::plugins::parse(&folder,&std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
        let temp=tempfile::tempdir().unwrap();
        let script=temp.path().join("plugin.js");
        std::fs::write(&script,r#"
const args=process.argv.slice(2), value=k=>args[args.indexOf(k)+1];
const ws=new WebSocket(`ws://127.0.0.1:${value('-port')}`);
let count=0;
ws.addEventListener('open',()=>ws.send(JSON.stringify({event:value('-registerEvent'),uuid:value('-pluginUUID')})));
ws.addEventListener('message',m=>{const msg=JSON.parse(m.data);if(msg.event!=='keyDown')return;
 count++;const context=msg.context;
 ws.send(JSON.stringify({event:'setTitle',context,payload:{title:`state-${count}-${msg.payload.state}`}}));
 if(count===1)ws.send(JSON.stringify({event:'setState',context,action:msg.action,payload:{state:1,novydeckStateSource:'local'}}));
 if(count===2){ws.send(JSON.stringify({event:'novydeckStateUnknown',context:'foreign-context'}));ws.send(JSON.stringify({event:'novydeckStateUnknown',context,action:'foreign-action'}));ws.send(JSON.stringify({event:'novydeckStateUnknown',context}));}
});
"#).unwrap();
        plugin.folder=temp.path().into();plugin.runtime=crate::plugins::Runtime::Node(script);
        let mut manager=Manager::new(vec![plugin.clone()]);
        let mut config=Config::default();config.keys_mut()[0].plugin=Some(PluginBinding{plugin:plugin.uuid.clone(),action:plugin.actions[0].uuid.clone(),settings:json!({}),state:0});
        manager.sync(&config);
        let context=manager.visible().values().next().unwrap().context.clone();
        let await_message=|manager:&Manager,name:&str|->Value{
            let deadline=Instant::now()+Duration::from_secs(15);
            loop {match manager.rx.recv_timeout(deadline.saturating_duration_since(Instant::now())).expect(name){
                Reply::Message(_,v) if v["event"]==name=>return v,
                Reply::Error(_,error)=>panic!("unexpected reply: {error}"),
                _=>{}
            }}
        };
        manager.key(0,true);
        assert_eq!(await_message(&manager,"setTitle")["payload"]["title"],"state-1-0");
        assert_eq!(await_message(&manager,"setState")["payload"]["state"],1);
        manager.sync(&config);
        let v=manager.visible()[&context].clone();manager.tx.send(Request::Virtual(v)).unwrap();
        assert_eq!(await_message(&manager,"setTitle")["payload"]["title"],"state-2-1");
        let unknown={
            let deadline=Instant::now()+Duration::from_secs(15);let mut rejected=0;
            loop {match manager.rx.recv_timeout(deadline.saturating_duration_since(Instant::now())).expect("unknown") {
                Reply::Error(id,_) if id==plugin.uuid=>rejected+=1,
                Reply::Message(_,v) if v["event"]=="novydeckStateUnknown"=>{assert_eq!(rejected,2,"foreign context and action rejected");break v},
                _=>{}
            }}
        };
        assert_eq!(unknown["context"],context);
        manager.key(0,true);
        assert_eq!(await_message(&manager,"setTitle")["payload"]["title"],"state-3-0");
    }
}
