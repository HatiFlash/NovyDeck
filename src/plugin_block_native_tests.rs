//! Opt-in oracle: real isolated WebView2 windows, offline fixture only.
use super::*;
#[test]
#[ignore = "requires the Windows lab interactive desktop and capture helper"]
fn native_block_inspector_windows_route_independent_settings_without_keydown() {
    assert_eq!(std::env::var("NOVYDECK_LAB_NATIVE_PI").as_deref(),Ok("1"));
    let output=std::path::PathBuf::from(std::env::var_os("NOVYDECK_LAB_PI_OUTPUT").expect("lab output"));
    let capture=std::path::PathBuf::from(std::env::var_os("NOVYDECK_LAB_PI_CAPTURE").expect("capture helper"));
    std::fs::create_dir_all(&output).unwrap();
    let temp=tempfile::tempdir().unwrap();let folder=temp.path().join("native.sdPlugin");std::fs::create_dir_all(&folder).unwrap();
    let manifest=json!({"UUID":"org.novydeck.native.oracle","Name":"Native PI Oracle","Version":"1.0.0","CodePath":"plugin.js","Actions":[{"UUID":"org.novydeck.native.oracle.settings","Name":"Independent block","PropertyInspectorPath":"inspector.html","States":[{"Title":"Oracle"}]}]});
    std::fs::write(folder.join("manifest.json"),serde_json::to_vec(&manifest).unwrap()).unwrap();
    std::fs::write(folder.join("plugin.js"),r#"const a=process.argv.slice(2),v=k=>a[a.indexOf(k)+1];const ws=new WebSocket('ws://127.0.0.1:'+v('-port'));ws.onopen=()=>ws.send(JSON.stringify({event:v('-registerEvent'),uuid:v('-pluginUUID')}));ws.onmessage=e=>{const m=JSON.parse(e.data);if(m.event==='keyDown')ws.send(JSON.stringify({event:'logMessage',context:m.context,payload:{message:'UNEXPECTED_KEYDOWN'}}));};"#).unwrap();
    std::fs::write(folder.join("inspector.html"),r#"<!doctype html><meta charset="utf-8"><title>Native PI Oracle</title><style>body{font:16px system-ui;background:#172030;color:#eef4ff;padding:24px}input{font:24px system-ui;width:90%;padding:12px}small{display:block;margin-top:20px}</style><h1>Independent block settings</h1><p>Real isolated WebView2. Offline fixture.</p><input id="value" readonly><small id="context"></small><script>function connectElgatoStreamDeckSocket(port,uuid,event,info,actionInfo){const action=JSON.parse(actionInfo),n=action.payload.settings.n;document.querySelector('#value').value=n+1;document.querySelector('#context').textContent=action.context;const ws=new WebSocket('ws://127.0.0.1:'+port);ws.onopen=()=>{ws.send(JSON.stringify({event:event,uuid:uuid}));ws.send(JSON.stringify({event:'setSettings',context:uuid,payload:{n:n+1,native:true}}));};}</script>"#).unwrap();
    let plugin=crate::plugins::parse(&folder,&serde_json::to_vec(&manifest).unwrap()).unwrap();
    let manager=Manager::new(vec![plugin.clone()]);
    let contexts=["novydeck-inspector-block-native-one","novydeck-inspector-block-native-two"];
    for (index,context) in contexts.iter().enumerate(){manager.open_virtual_inspector(Visible{context:(*context).into(),plugin:plugin.uuid.clone(),action:plugin.actions[0].uuid.clone(),index,settings:json!({"n":if index==0{10}else{20}}),state:0}).unwrap();}
    let until=Instant::now()+Duration::from_secs(35);let mut received=HashMap::new();
    while Instant::now()<until && received.len()<2 {
        match manager.rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Reply::Message(owner,value)) if value["event"]=="setSettings"=>{
                assert_eq!(owner,plugin.uuid);assert_eq!(value["payload"]["native"],true);
                received.insert(value["context"].as_str().unwrap().to_owned(),value["payload"]["n"].as_u64().unwrap());
            },
            Ok(Reply::Message(_,value))=>assert_ne!(value["payload"]["message"],"UNEXPECTED_KEYDOWN"),
            Ok(Reply::Error(_,error))=>panic!("native inspector runtime error: {error}"),
            Err(_)=>{},
        }
    }
    assert_eq!(received.get(contexts[0]),Some(&11),"first real WebView callback absent");
    assert_eq!(received.get(contexts[1]),Some(&21),"second real WebView callback absent");
    let mut process=std::process::Command::new("powershell.exe").args(["-NoProfile","-ExecutionPolicy","Bypass","-File"]).arg(capture).arg("-ParentId").arg(std::process::id().to_string()).arg("-OutputDirectory").arg(&output).spawn().unwrap();
    let until=Instant::now()+Duration::from_secs(25);
    let status=loop {if let Some(status)=process.try_wait().unwrap(){break status;}if Instant::now()>=until {let _=process.kill();panic!("native window capture exceeded timeout");}thread::sleep(Duration::from_millis(100));};
    assert!(status.success(),"native window capture failed");
    for index in 0..2 {assert!(output.join(format!("native-block-{index}.png")).is_file());}
    manager.close_virtual_inspector(contexts[0]).unwrap();
    let (tx,rx)=mpsc::channel();manager.tx.send(Request::TestSessions(tx)).unwrap();
    let remaining=rx.recv_timeout(Duration::from_secs(3)).unwrap();assert_eq!(remaining.len(),1);assert_eq!(remaining[0].1,contexts[1]);
    manager.control().pause_plugin(&plugin.uuid).unwrap();
    println!("NATIVE_PI_WINDOWS=2 CALLBACKS=2 VALUES=11,21 CLOSED_TARGET_ONLY=1 NO_KEYDOWN=1");
}
