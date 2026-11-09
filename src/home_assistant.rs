//! Home Assistant : client WebSocket dédié (jamais sur le thread de l'interface),
//! cache des états, jeton protégé par DPAPI et machine à états des déclencheurs.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fmt,
    net::{TcpStream, ToSocketAddrs},
    sync::{
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
        Arc, Mutex, OnceLock,
    },
    time::{Duration, Instant},
};
use tungstenite::{Message, WebSocket};

// ---------------------------------------------------------------- jeton

/// Jeton en clair, uniquement en mémoire. Jamais sérialisé, jamais affiché.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Secret(String);
impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into().trim().to_owned())
    }
    pub fn expose(&self) -> &str {
        &self.0
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

/// Jeton chiffré par DPAPI (compte Windows courant), encodé en base64 dans preferences.json.
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProtectedToken(String);
impl fmt::Debug for ProtectedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            f.write_str("ProtectedToken(vide)")
        } else {
            f.write_str("ProtectedToken(protégé)")
        }
    }
}
impl ProtectedToken {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn protect(secret: &Secret) -> Result<Self> {
        if secret.is_empty() {
            return Ok(Self::default());
        }
        use base64::Engine;
        Ok(Self(
            base64::engine::general_purpose::STANDARD
                .encode(dpapi::protect(secret.expose().as_bytes())?),
        ))
    }
    pub fn reveal(&self) -> Result<Secret> {
        if self.0.is_empty() {
            return Ok(Secret::default());
        }
        use base64::Engine;
        let blob = base64::engine::general_purpose::STANDARD
            .decode(&self.0)
            .context("Jeton Home Assistant illisible")?;
        let plain = dpapi::unprotect(&blob)?;
        Ok(Secret::new(
            String::from_utf8(plain).context("Jeton Home Assistant illisible")?,
        ))
    }
}

#[cfg(windows)]
mod dpapi {
    use anyhow::{bail, Result};
    use windows_sys::Win32::{Foundation::LocalFree, Security::Cryptography::*};
    const ENTROPY: &[u8] = b"NovyDeck.HomeAssistant.v1";
    fn blob(data: &[u8]) -> CRYPT_INTEGER_BLOB {
        CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        }
    }
    fn run(data: &[u8], protect: bool, extra: Option<&[u8]>) -> Result<Vec<u8>> {
        anyhow::ensure!(data.len() < 64 * 1024, "Jeton Home Assistant trop long");
        let input = blob(data);
        let entropy = extra.map(blob);
        let entropy_ptr = entropy.as_ref().map_or(std::ptr::null(), |e| e as *const _);
        let mut out = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };
        // SAFETY : tampons valides pendant l'appel ; la sortie est libérée par LocalFree.
        let ok = unsafe {
            if protect {
                CryptProtectData(
                    &input,
                    std::ptr::null(),
                    entropy_ptr,
                    std::ptr::null(),
                    std::ptr::null(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut out,
                )
            } else {
                CryptUnprotectData(
                    &input,
                    std::ptr::null_mut(),
                    entropy_ptr,
                    std::ptr::null(),
                    std::ptr::null(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut out,
                )
            }
        };
        if ok == 0 || out.pbData.is_null() {
            bail!("Protection Windows du jeton Home Assistant indisponible");
        }
        let bytes = unsafe { std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec() };
        unsafe {
            LocalFree(out.pbData as _);
        }
        Ok(bytes)
    }
    pub fn protect(data: &[u8]) -> Result<Vec<u8>> {
        run(data, true, Some(ENTROPY))
    }
    pub fn unprotect(data: &[u8]) -> Result<Vec<u8>> {
        run(data, false, Some(ENTROPY))
    }
    /// Jeton du plugin officiel : chiffré sans entropie supplémentaire.
    pub fn unprotect_plain(data: &[u8]) -> Result<Vec<u8>> {
        run(data, false, None)
    }
}
#[cfg(not(windows))]
mod dpapi {
    pub fn protect(_: &[u8]) -> anyhow::Result<Vec<u8>> {
        anyhow::bail!("DPAPI indisponible")
    }
    pub fn unprotect(_: &[u8]) -> anyhow::Result<Vec<u8>> {
        anyhow::bail!("DPAPI indisponible")
    }
    pub fn unprotect_plain(_: &[u8]) -> anyhow::Result<Vec<u8>> {
        anyhow::bail!("DPAPI indisponible")
    }
}

/// Réglages persistés (preferences.json). Le jeton n'y figure que chiffré.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Prefs {
    pub url: String,
    pub token: ProtectedToken,
    /// Pastille d'état superposée aux touches liées à un déclencheur Home Assistant.
    pub key_badge: bool,
}
impl Default for Prefs {
    fn default() -> Self {
        Self {
            url: String::new(),
            token: ProtectedToken::default(),
            key_badge: true,
        }
    }
}
impl Prefs {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
    pub fn configured(&self) -> bool {
        !self.url.trim().is_empty() && !self.token.is_empty()
    }
}

// ---------------------------------------------------------------- plugin officiel (lecture seule)

pub const PLUGIN_ID: &str = "com.novydeck.homeassistant";
/// Première adresse trouvée dans les réglages des touches du plugin officiel.
pub fn plugin_url(config: &crate::model::Config) -> Option<String> {
    config
        .profiles
        .iter()
        .flat_map(|p| std::iter::once(&p.keys).chain(p.pages.iter().map(|page| &page.keys)))
        .flatten()
        .filter_map(|k| k.plugin.as_ref())
        .filter(|b| b.plugin == PLUGIN_ID)
        .find_map(|b| {
            b.settings
                .get("url")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|u| !u.is_empty())
                .map(str::to_owned)
        })
}
/// Lit (sans le modifier) le jeton chiffré par le plugin officiel pour ce compte Windows.
pub fn plugin_token() -> Result<Secret> {
    let path =
        std::path::PathBuf::from(std::env::var_os("APPDATA").context("APPDATA indisponible")?)
            .join("NovyDeck")
            .join("plugins")
            .join(PLUGIN_ID)
            .join("secrets.bin");
    let blob =
        std::fs::read(&path).context("Aucun jeton enregistré par le plugin Home Assistant")?;
    anyhow::ensure!(blob.len() < 64 * 1024, "Jeton du plugin illisible");
    let plain = dpapi::unprotect_plain(&blob).context("Jeton du plugin illisible")?;
    let token = Secret::new(String::from_utf8(plain).context("Jeton du plugin illisible")?);
    anyhow::ensure!(!token.is_empty(), "Jeton du plugin vide");
    Ok(token)
}

// ---------------------------------------------------------------- adresse

/// Accepte `http(s)://hôte:8123`, `ws(s)://…` avec ou sans `/api/websocket`.
pub fn websocket_url(raw: &str) -> Result<String> {
    let mut url = url::Url::parse(raw.trim()).context("Adresse Home Assistant invalide")?;
    let scheme = match url.scheme() {
        "http" | "ws" => "ws",
        "https" | "wss" => "wss",
        _ => anyhow::bail!("Adresse Home Assistant : http(s):// ou ws(s):// attendu"),
    };
    anyhow::ensure!(
        url.host_str().is_some() && url.username().is_empty() && url.password().is_none(),
        "Adresse Home Assistant invalide"
    );
    url.set_scheme(scheme)
        .ok()
        .context("Adresse Home Assistant invalide")?;
    url.set_query(None);
    url.set_fragment(None);
    let path = url.path().trim_end_matches('/').to_owned();
    if !path.ends_with("/api/websocket") {
        url.set_path(&format!("{path}/api/websocket"));
    }
    Ok(url.into())
}

// ---------------------------------------------------------------- protocole

#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    pub state: String,
    pub attributes: Value,
}
impl Entity {
    pub fn friendly_name(&self) -> Option<&str> {
        self.attributes.get("friendly_name").and_then(Value::as_str)
    }
    /// Valeur lue : l'état, ou un attribut si `attribute` n'est pas vide.
    pub fn value(&self, attribute: &str) -> Option<String> {
        if attribute.is_empty() {
            return Some(self.state.clone());
        }
        match self.attributes.get(attribute)? {
            Value::Null => None,
            Value::String(s) => Some(s.clone()),
            other => Some(other.to_string()),
        }
    }
}
fn entity_of(value: &Value) -> Option<(String, Entity)> {
    let id = value.get("entity_id")?.as_str()?.to_owned();
    let state = value
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    Some((
        id,
        Entity {
            state,
            attributes: value.get("attributes").cloned().unwrap_or(Value::Null),
        },
    ))
}

#[derive(Clone, Debug, PartialEq)]
pub enum Incoming {
    AuthRequired,
    AuthOk,
    AuthInvalid(String),
    Result {
        id: u64,
        success: bool,
        result: Value,
        error: String,
    },
    StateChanged {
        entity_id: String,
        new_state: Option<Entity>,
    },
    Pong,
    Other,
}
pub fn parse(text: &str) -> Result<Incoming> {
    let v: Value = serde_json::from_str(text).context("Message Home Assistant illisible")?;
    Ok(match v["type"].as_str().unwrap_or("") {
        "auth_required" => Incoming::AuthRequired,
        "auth_ok" => Incoming::AuthOk,
        "auth_invalid" => Incoming::AuthInvalid(
            v["message"]
                .as_str()
                .unwrap_or("")
                .chars()
                .take(200)
                .collect(),
        ),
        "result" => Incoming::Result {
            id: v["id"].as_u64().unwrap_or(0),
            success: v["success"].as_bool().unwrap_or(false),
            result: v["result"].clone(),
            error: v["error"]["message"]
                .as_str()
                .unwrap_or("")
                .chars()
                .take(200)
                .collect(),
        },
        "pong" => Incoming::Pong,
        "event" if v["event"]["event_type"] == "state_changed" => {
            let data = &v["event"]["data"];
            let Some(entity_id) = data["entity_id"].as_str() else {
                return Ok(Incoming::Other);
            };
            Incoming::StateChanged {
                entity_id: entity_id.to_owned(),
                new_state: entity_of(&data["new_state"]).map(|(_, e)| e),
            }
        }
        _ => Incoming::Other,
    })
}
pub fn states_of(result: &Value) -> HashMap<String, Entity> {
    result
        .as_array()
        .map(|xs| xs.iter().filter_map(entity_of).collect())
        .unwrap_or_default()
}

// ---------------------------------------------------------------- connexion

type Socket = WebSocket<TcpStream>;
const SUBSCRIBE_ID: u64 = 1;
const STATES_ID: u64 = 2;

#[derive(Debug)]
struct AuthRejected(String);
impl fmt::Display for AuthRejected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            f.write_str("Jeton Home Assistant refusé")
        } else {
            write!(f, "Jeton Home Assistant refusé : {}", self.0)
        }
    }
}
impl std::error::Error for AuthRejected {}

fn set_timeout(ws: &mut Socket, timeout: Duration) -> Result<()> {
    ws.get_mut().set_read_timeout(Some(timeout))?;
    Ok(())
}
fn read_text(ws: &mut Socket) -> Result<Option<String>> {
    match ws.read() {
        Ok(Message::Text(text)) => Ok(Some(text.to_string())),
        Ok(Message::Close(_)) => anyhow::bail!("Home Assistant a fermé la connexion"),
        Ok(_) => Ok(None),
        Err(tungstenite::Error::Io(e))
            if matches!(
                e.kind(),
                std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
            ) =>
        {
            Ok(None)
        }
        Err(e) => Err(e).context("Connexion Home Assistant interrompue"),
    }
}
fn send(ws: &mut Socket, value: Value) -> Result<()> {
    ws.send(Message::Text(value.to_string().into()))
        .context("Envoi vers Home Assistant impossible")
}
/// Ouvre la connexion et s'authentifie. Les erreurs ne contiennent jamais le jeton.
fn open(url: &str, token: &Secret, timeout: Duration) -> Result<Socket> {
    let target = websocket_url(url)?;
    let parsed = url::Url::parse(&target)?;
    // TLS non embarqué (aucune pile TLS dans les dépendances) : adresse locale http:// uniquement.
    anyhow::ensure!(
        parsed.scheme() == "ws",
        "Connexion chiffrée (https) non prise en charge : utilise l'adresse locale http://IP:8123"
    );
    let host = parsed
        .host_str()
        .context("Adresse Home Assistant invalide")?
        .to_owned();
    let port = parsed
        .port_or_known_default()
        .context("Port Home Assistant invalide")?;
    let mut last = None;
    let mut stream = None;
    for addr in (host.as_str(), port)
        .to_socket_addrs()
        .context("Serveur Home Assistant introuvable")?
    {
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(e) => last = Some(e),
        }
    }
    let stream = match stream {
        Some(s) => s,
        None => {
            return Err(last
                .map(anyhow::Error::from)
                .unwrap_or_else(|| anyhow::anyhow!("aucune adresse")))
            .context("Serveur Home Assistant injoignable")
        }
    };
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let _ = stream.set_nodelay(true);
    let (mut ws, _) = tungstenite::client(target.as_str(), stream).map_err(|e| match e {
        tungstenite::HandshakeError::Failure(e) => {
            anyhow::Error::from(e).context("Connexion WebSocket Home Assistant impossible")
        }
        tungstenite::HandshakeError::Interrupted(_) => {
            anyhow::anyhow!("Connexion WebSocket Home Assistant interrompue")
        }
    })?;
    let deadline = Instant::now() + timeout;
    let mut sent = false;
    loop {
        anyhow::ensure!(
            Instant::now() < deadline,
            "Home Assistant ne répond pas à l'authentification"
        );
        let Some(text) = read_text(&mut ws)? else {
            continue;
        };
        match parse(&text)? {
            Incoming::AuthRequired if !sent => {
                send(
                    &mut ws,
                    json!({"type": "auth", "access_token": token.expose()}),
                )?;
                sent = true;
            }
            Incoming::AuthOk => return Ok(ws),
            Incoming::AuthInvalid(message) => return Err(AuthRejected(message).into()),
            _ => {}
        }
    }
}
/// « Tester la connexion » : bloquant, à appeler depuis un thread dédié. Renvoie le nombre d'entités.
pub fn test_connection(url: &str, token: &Secret, timeout: Duration) -> Result<usize> {
    let mut ws = open(url, token, timeout)?;
    send(&mut ws, json!({"id": STATES_ID, "type": "get_states"}))?;
    let deadline = Instant::now() + timeout;
    loop {
        anyhow::ensure!(
            Instant::now() < deadline,
            "Home Assistant ne renvoie pas ses états"
        );
        let Some(text) = read_text(&mut ws)? else {
            continue;
        };
        if let Incoming::Result {
            id: STATES_ID,
            success,
            result,
            error,
        } = parse(&text)?
        {
            anyhow::ensure!(
                success,
                "Home Assistant refuse la lecture des états : {error}"
            );
            let _ = ws.close(None);
            return Ok(states_of(&result).len());
        }
    }
}

// ---------------------------------------------------------------- moniteur

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Disabled,
    Connecting,
    Connected,
    Reconnecting {
        attempt: u32,
        error: String,
    },
    AuthFailed(String),
}
impl Status {
    pub fn connected(&self) -> bool {
        matches!(self, Self::Connected)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub url: String,
    pub token: Secret,
}

#[derive(Default)]
pub struct Shared {
    pub status: Status,
    pub entities: HashMap<String, Entity>,
    pub generation: u64,
}

pub struct Monitor {
    tx: Sender<Option<Settings>>,
    shared: Arc<Mutex<Shared>>,
    timing: Timing,
}
#[derive(Clone, Copy)]
struct Timing {
    connect: Duration,
    keepalive: Duration,
    silence: Duration,
    poll: Duration,
    scale_ms: u64,
}
const REAL: Timing = Timing {
    connect: Duration::from_secs(8),
    keepalive: Duration::from_secs(30),
    silence: Duration::from_secs(75),
    poll: Duration::from_millis(250),
    scale_ms: 1000,
};

/// Délai croissant plafonné ; plus long après un jeton refusé pour ne pas marteler le serveur.
pub fn backoff(attempt: u32, auth_failed: bool) -> Duration {
    let secs = [1u64, 2, 5, 10, 30, 60][attempt.min(5) as usize];
    Duration::from_secs(if auth_failed { secs.max(60) * 5 } else { secs })
}

impl Monitor {
    pub fn spawn() -> Self {
        Self::spawn_with(REAL)
    }
    fn spawn_with(timing: Timing) -> Self {
        let (tx, rx) = mpsc::channel();
        let shared = Arc::new(Mutex::new(Shared::default()));
        let worker = shared.clone();
        std::thread::Builder::new()
            .name("home-assistant".into())
            .spawn(move || run(rx, worker, timing))
            .expect("thread Home Assistant");
        Self { tx, shared, timing }
    }
    /// Remplace les réglages (None = désactivé). N'effectue aucun accès réseau sur l'appelant.
    pub fn configure(&self, settings: Option<Settings>) {
        let _ = self.tx.send(settings);
    }
    pub fn read<R>(&self, f: impl FnOnce(&Shared) -> R) -> R {
        f(&self.shared.lock().unwrap_or_else(|e| e.into_inner()))
    }
    pub fn status(&self) -> Status {
        self.read(|s| s.status.clone())
    }
    pub fn generation(&self) -> u64 {
        self.read(|s| s.generation)
    }
    /// Entités connues triées : (entity_id, nom lisible).
    pub fn entity_list(&self) -> Vec<(String, String)> {
        let mut list = self.read(|s| {
            s.entities
                .iter()
                .map(|(id, e)| (id.clone(), e.friendly_name().unwrap_or("").to_owned()))
                .collect::<Vec<_>>()
        });
        list.sort();
        list
    }
    #[cfg(test)]
    fn timing(&self) -> Timing {
        self.timing
    }
}

static GLOBAL: OnceLock<Monitor> = OnceLock::new();
pub fn global() -> &'static Monitor {
    GLOBAL.get_or_init(Monitor::spawn)
}
/// Lecture du cache uniquement, sans réseau : None si Home Assistant n'est pas connecté.
pub fn cached_value(entity: &str, attribute: &str) -> Option<Option<String>> {
    let monitor = GLOBAL.get()?;
    monitor.read(|s| {
        s.status
            .connected()
            .then(|| s.entities.get(entity).and_then(|e| e.value(attribute)))
    })
}

fn bump(shared: &Mutex<Shared>, f: impl FnOnce(&mut Shared)) {
    let mut s = shared.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut s);
    s.generation = s.generation.wrapping_add(1);
}

enum Exit {
    Settings(Option<Settings>),
    Stop,
}

fn session(
    settings: &Settings,
    shared: &Mutex<Shared>,
    rx: &Receiver<Option<Settings>>,
    timing: Timing,
) -> Result<Exit> {
    let mut ws = open(&settings.url, &settings.token, timing.connect)?;
    // Abonnement avant la lecture complète : aucun changement ne peut passer entre les deux.
    send(
        &mut ws,
        json!({"id": SUBSCRIBE_ID, "type": "subscribe_events", "event_type": "state_changed"}),
    )?;
    send(&mut ws, json!({"id": STATES_ID, "type": "get_states"}))?;
    set_timeout(&mut ws, timing.poll)?;
    let mut next_id = STATES_ID + 1;
    let mut last_rx = Instant::now();
    let mut last_ping = Instant::now();
    loop {
        match rx.try_recv() {
            Ok(other) => {
                if other.as_ref() != Some(settings) {
                    let _ = ws.close(None);
                    return Ok(Exit::Settings(other));
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                let _ = ws.close(None);
                return Ok(Exit::Stop);
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
        if let Some(text) = read_text(&mut ws)? {
            last_rx = Instant::now();
            match parse(&text)? {
                Incoming::Result {
                    id: STATES_ID,
                    success,
                    result,
                    error,
                } => {
                    anyhow::ensure!(
                        success,
                        "Home Assistant refuse la lecture des états : {error}"
                    );
                    let entities = states_of(&result);
                    bump(shared, |s| {
                        s.entities = entities;
                        s.status = Status::Connected;
                    });
                }
                Incoming::Result {
                    id: SUBSCRIBE_ID,
                    success: false,
                    error,
                    ..
                } => anyhow::bail!("Abonnement Home Assistant refusé : {error}"),
                Incoming::StateChanged {
                    entity_id,
                    new_state,
                } => bump(shared, |s| match new_state {
                    Some(e) => {
                        s.entities.insert(entity_id, e);
                    }
                    None => {
                        s.entities.remove(&entity_id);
                    }
                }),
                _ => {}
            }
        }
        if last_rx.elapsed() >= timing.silence {
            anyhow::bail!("Home Assistant ne répond plus");
        }
        if last_ping.elapsed() >= timing.keepalive {
            last_ping = Instant::now();
            send(&mut ws, json!({"id": next_id, "type": "ping"}))?;
            next_id += 1;
        }
    }
}

fn run(rx: Receiver<Option<Settings>>, shared: Arc<Mutex<Shared>>, timing: Timing) {
    let mut current: Option<Settings> = None;
    let mut attempt = 0u32;
    let mut due = Instant::now();
    let adopt = |next: Option<Settings>,
                 current: &mut Option<Settings>,
                 attempt: &mut u32,
                 due: &mut Instant| {
        if next != *current {
            *current = next;
            *attempt = 0;
            *due = Instant::now();
            let status = if current.is_some() {
                Status::Connecting
            } else {
                Status::Disabled
            };
            bump(&shared, |s| {
                s.status = status;
                if current.is_none() {
                    s.entities.clear();
                }
            });
        }
    };
    loop {
        let wait = due
            .saturating_duration_since(Instant::now())
            .min(timing.poll);
        match rx.recv_timeout(wait) {
            Ok(next) => adopt(next, &mut current, &mut attempt, &mut due),
            Err(RecvTimeoutError::Disconnected) => return,
            Err(RecvTimeoutError::Timeout) => {}
        }
        if Instant::now() < due {
            continue;
        }
        let Some(settings) = current.clone() else {
            due = Instant::now() + Duration::from_secs(3600);
            continue;
        };
        match session(&settings, &shared, &rx, timing) {
            Ok(Exit::Stop) => return,
            Ok(Exit::Settings(next)) => adopt(next, &mut current, &mut attempt, &mut due),
            Err(error) => {
                let auth = error.downcast_ref::<AuthRejected>().is_some();
                let message = format!("{error:#}").chars().take(240).collect::<String>();
                let status = if auth {
                    Status::AuthFailed(message)
                } else {
                    Status::Reconnecting {
                        attempt: attempt + 1,
                        error: message,
                    }
                };
                bump(&shared, |s| s.status = status);
                let delay = backoff(attempt, auth);
                due = Instant::now()
                    + Duration::from_millis(delay.as_millis() as u64 * timing.scale_ms / 1000);
                attempt = attempt.saturating_add(1);
            }
        }
    }
}

// ---------------------------------------------------------------- déclencheur

/// Règle d'un déclencheur `ha_state` (stockée dans `Trigger::ha`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Rule {
    pub entity: String,
    /// Vide = état principal de l'entité.
    pub attribute: String,
    /// `equals`, `not_equals`, `above`, `below`.
    pub op: String,
    pub value: String,
    pub hold_seconds: u32,
    /// Touche exécutée quand la condition redevient fausse après un déclenchement.
    pub return_key: Option<usize>,
    /// Autorise le déclenchement si la condition est déjà vraie au démarrage.
    pub fire_on_start: bool,
    /// L'image de la touche suit le mode : 2ᵉ état (« actif ») tant que le mode est actif, 1er état sinon.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub show_mode: bool,
    /// Touche qui affiche le mode (None = la touche cible du déclencheur).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_key: Option<usize>,
}
impl Default for Rule {
    fn default() -> Self {
        Self {
            entity: String::new(),
            attribute: String::new(),
            op: "equals".into(),
            value: String::new(),
            hold_seconds: 0,
            return_key: None,
            fire_on_start: false,
            show_mode: false,
            display_key: None,
        }
    }
}
pub const OPS: [&str; 4] = ["equals", "not_equals", "above", "below"];
pub fn valid_entity_id(id: &str) -> bool {
    let mut parts = id.split('.');
    let ok = |s: &str| {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    };
    id.len() <= 255
        && matches!((parts.next(), parts.next(), parts.next()), (Some(a), Some(b), None) if ok(a) && ok(b))
}
pub fn validate_comparison(entity: &str, attribute: &str, op: &str, value: &str) -> Result<()> {
    anyhow::ensure!(valid_entity_id(entity), "Entité Home Assistant invalide");
    anyhow::ensure!(
        attribute.len() <= 128 && !attribute.chars().any(char::is_control),
        "Attribut Home Assistant invalide"
    );
    anyhow::ensure!(OPS.contains(&op), "Comparaison Home Assistant inconnue");
    anyhow::ensure!(
        value.len() <= 256 && !value.chars().any(char::is_control),
        "Valeur Home Assistant invalide"
    );
    if matches!(op, "above" | "below") {
        anyhow::ensure!(
            value.trim().parse::<f64>().is_ok_and(f64::is_finite),
            "Seuil numérique attendu"
        );
    }
    Ok(())
}
impl Rule {
    pub fn validate(&self, key: usize) -> Result<()> {
        validate_comparison(&self.entity, &self.attribute, &self.op, &self.value)?;
        anyhow::ensure!(self.hold_seconds <= 86_400, "Durée : 24 h au maximum");
        anyhow::ensure!(
            self.return_key.is_none_or(|k| k < 15 && k != key),
            "Touche de retour invalide"
        );
        anyhow::ensure!(
            self.display_key.is_none_or(|k| k < 15),
            "Touche d'affichage invalide"
        );
        Ok(())
    }
    pub fn matches(&self, actual: Option<&str>) -> bool {
        compare(actual, &self.op, &self.value)
    }
}
/// `None` (entité ou attribut absent) ne vérifie aucune comparaison, sauf « différent ».
pub fn compare(actual: Option<&str>, op: &str, value: &str) -> bool {
    let number = |s: &str| s.trim().parse::<f64>().ok().filter(|v| v.is_finite());
    match (op, actual) {
        ("equals", Some(a)) => a.trim().eq_ignore_ascii_case(value.trim()),
        ("not_equals", a) => !a.is_some_and(|a| a.trim().eq_ignore_ascii_case(value.trim())),
        ("above", Some(a)) => matches!((number(a), number(value)), (Some(a), Some(b)) if a > b),
        ("below", Some(a)) => matches!((number(a), number(value)), (Some(a), Some(b)) if a < b),
        _ => false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    Disconnected,
    Idle,
    Countdown(u32),
    Fired,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Fire,
    Return,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Unprimed,
    Idle,
    Pending(Instant),
    Latched,
    Fired,
}

pub struct Sample<'a> {
    pub now: Instant,
    pub connected: bool,
    /// Valeur lue (état ou attribut), None si absente.
    pub actual: Option<&'a str>,
    /// Pause globale, édition en cours… : les transitions ont lieu sans action.
    pub blocked: bool,
}

/// Front montant, durée « depuis X s », annulation et retour. Pure : testable sans réseau.
#[derive(Clone, Debug, PartialEq)]
pub struct Machine {
    phase: Phase,
    actual: Option<String>,
}
impl Default for Machine {
    fn default() -> Self {
        Self {
            phase: Phase::Unprimed,
            actual: None,
        }
    }
}
impl Machine {
    pub fn step(&mut self, rule: &Rule, s: Sample) -> Option<Action> {
        if !s.connected {
            // Déconnexion : un compte à rebours en cours est annulé, le reste est conservé.
            if matches!(self.phase, Phase::Pending(_)) {
                self.phase = Phase::Idle;
            }
            self.actual = None;
            return None;
        }
        let changed = self.actual.as_deref() != s.actual;
        self.actual = s.actual.map(str::to_owned);
        let hold = Duration::from_secs(u64::from(rule.hold_seconds));
        let elapse = |since: Instant, phase: &mut Phase| -> Option<Action> {
            if s.now.saturating_duration_since(since) < hold {
                *phase = Phase::Pending(since);
                return None;
            }
            if s.blocked {
                *phase = Phase::Latched;
                None
            } else {
                *phase = Phase::Fired;
                Some(Action::Fire)
            }
        };
        match (self.phase, rule.matches(s.actual)) {
            (Phase::Unprimed, true) if !rule.fire_on_start => {
                self.phase = Phase::Latched;
                None
            }
            (Phase::Unprimed | Phase::Idle, true) => elapse(s.now, &mut self.phase),
            (Phase::Pending(_), true) if changed => elapse(s.now, &mut self.phase),
            (Phase::Pending(since), true) => elapse(since, &mut self.phase),
            (Phase::Latched | Phase::Fired, true) => None,
            (Phase::Fired, false) => {
                self.phase = Phase::Idle;
                (!s.blocked && rule.return_key.is_some()).then_some(Action::Return)
            }
            (_, false) => {
                self.phase = Phase::Idle;
                None
            }
        }
    }
    /// Mode actif : le déclencheur s'est déclenché (ou la condition était déjà vraie au démarrage / pendant
    /// une pause) et la condition n'est pas encore redevenue fausse. Conservé pendant une déconnexion.
    pub fn active(&self) -> bool {
        matches!(self.phase, Phase::Fired | Phase::Latched)
    }
    pub fn display(&self, rule: &Rule, now: Instant, connected: bool) -> Display {
        if !connected {
            return Display::Disconnected;
        }
        match self.phase {
            Phase::Pending(since) => {
                let left = Duration::from_secs(u64::from(rule.hold_seconds))
                    .saturating_sub(now.saturating_duration_since(since));
                Display::Countdown(left.as_secs() as u32 + u32::from(left.subsec_nanos() > 0))
            }
            Phase::Fired => Display::Fired,
            _ => Display::Idle,
        }
    }
}

// ---------------------------------------------------------------- exécution côté application

/// Touche ciblée : (profil, page, index).
pub type Target = (String, String, usize);

/// Machines des déclencheurs `ha_state`, conservées tant que le déclencheur ne change pas.
#[derive(Default)]
pub struct Runtime {
    machines: Vec<(crate::desktop::Trigger, Machine)>,
    pub displays: HashMap<Target, Display>,
    /// Mode actif/inactif des touches d'affichage (option « Image de la touche selon le mode »).
    pub modes: HashMap<Target, bool>,
}
impl Runtime {
    /// Avance toutes les machines. `read` lit le cache (aucun réseau). Renvoie les touches à exécuter
    /// et indique si l'affichage d'une touche a changé (au plus une fois par seconde pour un compte à rebours).
    pub fn tick(
        &mut self,
        triggers: &[crate::desktop::Trigger],
        now: Instant,
        connected: bool,
        blocked: bool,
        read: impl Fn(&str, &str) -> Option<String>,
    ) -> (Vec<Target>, bool) {
        let mut previous = std::mem::take(&mut self.machines);
        let mut actions = vec![];
        for trigger in triggers.iter().filter(|t| t.kind == "ha_state") {
            let Some(rule) = &trigger.ha else { continue };
            let mut machine = previous
                .iter()
                .position(|(t, _)| t == trigger)
                .map(|i| previous.swap_remove(i).1)
                .unwrap_or_default();
            let actual = if connected {
                read(&rule.entity, &rule.attribute)
            } else {
                None
            };
            match machine.step(
                rule,
                Sample {
                    now,
                    connected,
                    actual: actual.as_deref(),
                    blocked,
                },
            ) {
                Some(Action::Fire) => {
                    actions.push((trigger.profile.clone(), trigger.page.clone(), trigger.key))
                }
                Some(Action::Return) => {
                    if let Some(k) = rule.return_key {
                        actions.push((trigger.profile.clone(), trigger.page.clone(), k))
                    }
                }
                None => {}
            }
            self.machines.push((trigger.clone(), machine));
        }
        let mut displays = HashMap::new();
        for (trigger, machine) in &self.machines {
            let Some(rule) = &trigger.ha else { continue };
            let shown = machine.display(rule, now, connected);
            // Plusieurs scénarios sur une même touche : on garde l'état le plus parlant.
            let rank = |d: &Display| match d {
                Display::Idle => 0,
                Display::Disconnected => 1,
                Display::Countdown(_) => 2,
                Display::Fired => 3,
            };
            displays
                .entry((trigger.profile.clone(), trigger.page.clone(), trigger.key))
                .and_modify(|d: &mut Display| {
                    if rank(&shown) > rank(d) {
                        *d = shown
                    }
                })
                .or_insert(shown);
        }
        let mut modes: HashMap<Target, bool> = HashMap::new();
        for (trigger, machine) in &self.machines {
            let Some(rule) = trigger.ha.as_ref().filter(|r| r.show_mode) else { continue };
            let target = (
                trigger.profile.clone(),
                trigger.page.clone(),
                rule.display_key.unwrap_or(trigger.key),
            );
            *modes.entry(target).or_insert(false) |= machine.active();
        }
        let modes_changed = modes != self.modes;
        self.modes = modes;
        let changed = displays != self.displays || modes_changed;
        self.displays = displays;
        (actions, changed)
    }
    pub fn display(&self, target: &Target) -> Option<Display> {
        self.displays.get(target).copied()
    }
    pub fn mode(&self, target: &Target) -> Option<bool> {
        self.modes.get(target).copied()
    }
}

/// Texte de la pastille superposée (l'image de l'utilisateur reste intacte).
pub fn badge(display: Display) -> Option<String> {
    match display {
        Display::Idle => None,
        Display::Disconnected => Some("HA ×".into()),
        Display::Countdown(n) => Some(format!("{n} s")),
        Display::Fired => Some("HA OK".into()),
    }
}
/// Couleur de la pastille pour les badges Home Assistant ; None pour les autres badges.
pub fn badge_color(badge: &str) -> Option<[u8; 3]> {
    if badge == "HA ×" {
        Some([110, 118, 129])
    } else if badge == "HA OK" {
        Some([46, 160, 67])
    } else if badge.ends_with(" s") && badge.trim_end_matches(" s").parse::<u32>().is_ok() {
        Some([214, 140, 20])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rule(hold: u32) -> Rule {
        Rule {
            entity: "binary_sensor.test_presence".into(),
            op: "equals".into(),
            value: "off".into(),
            hold_seconds: hold,
            return_key: Some(2),
            ..Default::default()
        }
    }
    struct Run {
        m: Machine,
        r: Rule,
        t0: Instant,
    }
    impl Run {
        fn new(r: Rule) -> Self {
            Self {
                m: Machine::default(),
                r,
                t0: Instant::now(),
            }
        }
        fn at(&mut self, secs: u64, actual: &str) -> Option<Action> {
            self.go(secs, Some(actual), true, false)
        }
        fn go(
            &mut self,
            secs: u64,
            actual: Option<&str>,
            connected: bool,
            blocked: bool,
        ) -> Option<Action> {
            self.m.step(
                &self.r,
                Sample {
                    now: self.t0 + Duration::from_secs(secs),
                    connected,
                    actual,
                    blocked,
                },
            )
        }
        fn shows(&self, secs: u64) -> Display {
            self.m
                .display(&self.r, self.t0 + Duration::from_secs(secs), true)
        }
    }

    #[test]
    fn hold_then_single_rising_edge_then_return() {
        let mut r = Run::new(rule(30));
        assert_eq!(r.at(0, "on"), None);
        assert_eq!(r.at(1, "off"), None);
        assert_eq!(r.shows(8), Display::Countdown(23));
        assert_eq!(r.at(30, "off"), None);
        assert_eq!(r.at(31, "off"), Some(Action::Fire));
        assert_eq!(r.shows(31), Display::Fired);
        for t in 32..40 {
            assert_eq!(r.at(t, "off"), None, "pas de boucle");
        }
        assert_eq!(r.at(41, "on"), Some(Action::Return));
        assert_eq!(r.shows(41), Display::Idle);
        assert_eq!(r.at(42, "on"), None);
    }
    #[test]
    fn cancelled_when_condition_drops_before_the_end() {
        let mut r = Run::new(rule(30));
        r.at(0, "on");
        r.at(1, "off");
        assert_eq!(r.at(20, "on"), None, "annulé, pas d'action de retour");
        assert_eq!(r.shows(20), Display::Idle);
        assert_eq!(r.at(25, "off"), None);
        assert_eq!(r.at(54, "off"), None);
        assert_eq!(
            r.at(55, "off"),
            Some(Action::Fire),
            "nouveau compte complet depuis 25 s"
        );
    }
    #[test]
    fn countdown_restarts_when_the_value_changes() {
        let mut r = Run::new(Rule {
            op: "above".into(),
            value: "20".into(),
            hold_seconds: 10,
            ..rule(10)
        });
        r.at(0, "18");
        r.at(1, "21");
        assert_eq!(r.at(8, "22"), None, "changement : le compte repart");
        assert_eq!(r.at(11, "22"), None);
        assert_eq!(r.at(18, "22"), Some(Action::Fire));
    }
    #[test]
    fn nothing_fires_at_startup_unless_explicit() {
        let mut r = Run::new(rule(0));
        assert_eq!(r.at(0, "off"), None);
        assert_eq!(r.at(5, "off"), None);
        assert_eq!(r.at(6, "on"), None, "aucun retour sans déclenchement");
        assert_eq!(r.at(7, "off"), Some(Action::Fire));
        let mut r = Run::new(Rule {
            fire_on_start: true,
            ..rule(3)
        });
        assert_eq!(r.at(0, "off"), None);
        assert_eq!(r.at(3, "off"), Some(Action::Fire));
    }
    #[test]
    fn pause_blocks_fire_and_return_without_replay() {
        let mut r = Run::new(rule(5));
        r.at(0, "on");
        r.at(1, "off");
        assert_eq!(
            r.go(6, Some("off"), true, true),
            None,
            "bloqué pendant la pause"
        );
        assert_eq!(r.at(7, "off"), None, "pas de rattrapage après la pause");
        assert_eq!(r.at(8, "on"), None);
        r.at(9, "off");
        assert_eq!(r.at(14, "off"), Some(Action::Fire));
        assert_eq!(
            r.go(15, Some("on"), true, true),
            None,
            "retour bloqué aussi"
        );
    }
    #[test]
    fn disconnection_cancels_the_countdown_and_shows_it() {
        let mut r = Run::new(rule(10));
        r.at(0, "on");
        r.at(1, "off");
        assert_eq!(r.go(5, None, false, false), None);
        assert_eq!(r.m.display(&r.r, r.t0, false), Display::Disconnected);
        assert_eq!(r.at(6, "off"), None, "reprise : nouveau compte");
        assert_eq!(r.at(15, "off"), None);
        assert_eq!(r.at(16, "off"), Some(Action::Fire));
    }
    #[test]
    fn return_needs_a_return_key() {
        let mut r = Run::new(Rule {
            return_key: None,
            ..rule(0)
        });
        r.at(0, "on");
        assert_eq!(r.at(1, "off"), Some(Action::Fire));
        assert_eq!(r.at(2, "on"), None);
    }
    #[test]
    fn comparisons() {
        assert!(compare(Some("Off "), "equals", "off"));
        assert!(compare(None, "not_equals", "on") && !compare(Some("on"), "not_equals", "on"));
        assert!(
            compare(Some("21.5"), "above", "21") && !compare(Some("unavailable"), "above", "21")
        );
        assert!(compare(Some("-3"), "below", "0") && !compare(None, "below", "0"));
        let e = Entity {
            state: "on".into(),
            attributes: json!({"brightness": 128, "friendly_name": "Lampe"}),
        };
        assert_eq!(e.value("brightness").as_deref(), Some("128"));
        assert_eq!(e.value("absent"), None);
        assert_eq!(e.friendly_name(), Some("Lampe"));
    }
    #[test]
    fn rule_validation_and_old_json() {
        assert!(rule(30).validate(0).is_ok());
        assert!(Rule {
            entity: "Bad Id".into(),
            ..rule(0)
        }
        .validate(0)
        .is_err());
        assert!(Rule {
            op: "above".into(),
            value: "x".into(),
            ..rule(0)
        }
        .validate(0)
        .is_err());
        assert!(rule(0).validate(2).is_err(), "retour = même touche");
        let r: Rule = serde_json::from_str(r#"{"entity":"sensor.t"}"#).unwrap();
        assert_eq!(r.op, "equals");
    }
    #[test]
    fn urls() {
        assert_eq!(
            websocket_url("http://ha.local:8123").unwrap(),
            "ws://ha.local:8123/api/websocket"
        );
        assert_eq!(
            websocket_url("https://ha.example/").unwrap(),
            "wss://ha.example/api/websocket"
        );
        assert_eq!(
            websocket_url("wss://ha.example/api/websocket").unwrap(),
            "wss://ha.example/api/websocket"
        );
        assert!(websocket_url("ftp://x").is_err() && websocket_url("http://u:p@x").is_err());
    }
    #[test]
    fn messages() {
        assert_eq!(
            parse(r#"{"type":"auth_required","ha_version":"2026.10"}"#).unwrap(),
            Incoming::AuthRequired
        );
        assert_eq!(
            parse(r#"{"type":"auth_invalid","message":"Invalid access"}"#).unwrap(),
            Incoming::AuthInvalid("Invalid access".into())
        );
        let ev = r#"{"id":1,"type":"event","event":{"event_type":"state_changed","data":{"entity_id":"binary_sensor.test_presence","old_state":null,"new_state":{"entity_id":"binary_sensor.test_presence","state":"off","attributes":{}}}}}"#;
        assert!(
            matches!(parse(ev).unwrap(), Incoming::StateChanged { ref entity_id, new_state: Some(ref e) } if entity_id == "binary_sensor.test_presence" && e.state == "off")
        );
        let removed = r#"{"id":1,"type":"event","event":{"event_type":"state_changed","data":{"entity_id":"light.x","new_state":null}}}"#;
        assert!(matches!(
            parse(removed).unwrap(),
            Incoming::StateChanged {
                new_state: None,
                ..
            }
        ));
        let Incoming::Result { id: 2, success: true, result, .. } = parse(r#"{"id":2,"type":"result","success":true,"result":[{"entity_id":"light.a","state":"on","attributes":{"friendly_name":"A"}}]}"#).unwrap() else { panic!() };
        assert_eq!(states_of(&result)["light.a"].state, "on");
        assert!(parse("pas du json").is_err());
    }
    #[test]
    fn token_never_leaks() {
        let secret = Secret::new("eyJTOPSECRET");
        assert!(!format!("{secret:?}").contains("TOPSECRET"));
        let prefs = Prefs {
            url: "http://127.0.0.1:1".into(),
            token: ProtectedToken::protect(&secret).unwrap(),
            key_badge: true,
        };
        let saved = serde_json::to_string(&prefs).unwrap();
        assert!(!saved.contains("TOPSECRET") && !format!("{prefs:?}").contains("TOPSECRET"));
        let back: Prefs = serde_json::from_str(&saved).unwrap();
        assert_eq!(back.token.reveal().unwrap(), secret, "aller-retour DPAPI");
        assert!(ProtectedToken("AAAA".into()).reveal().is_err());
        let settings = Settings {
            url: prefs.url.clone(),
            token: secret,
        };
        assert!(!format!("{settings:?}").contains("TOPSECRET"));
    }

    fn mode_trigger(display_key: Option<usize>) -> crate::desktop::Trigger {
        crate::desktop::Trigger {
            profile: "P".into(),
            page: String::new(),
            key: 0,
            keys: String::new(),
            kind: "ha_state".into(),
            value: String::new(),
            days: vec![],
            minutes: 0,
            input: false,
            ha: Some(Rule {
                show_mode: true,
                display_key,
                ..rule(2)
            }),
        }
    }
    fn mode_of(rt: &Runtime, key: usize) -> Option<bool> {
        rt.mode(&("P".into(), String::new(), key))
    }
    #[test]
    fn mode_follows_inactive_countdown_active_return() {
        let triggers = vec![mode_trigger(Some(7))];
        let mut rt = Runtime::default();
        let t0 = Instant::now();
        let at = |s: u64| t0 + Duration::from_secs(s);
        let state = std::cell::RefCell::new("on");
        let read = |_: &str, _: &str| Some(state.borrow().to_string());
        // Démarrage : condition fausse -> inactif sur la touche d'affichage, pas sur la cible.
        rt.tick(&triggers, at(0), true, false, read);
        assert_eq!(mode_of(&rt, 7), Some(false));
        assert_eq!(mode_of(&rt, 0), None);
        // Compte à rebours : image inactive.
        *state.borrow_mut() = "off";
        rt.tick(&triggers, at(1), true, false, read);
        assert_eq!(rt.display(&("P".into(), String::new(), 0)), Some(Display::Countdown(2)));
        assert_eq!(mode_of(&rt, 7), Some(false));
        // Déclenché : actif.
        let (actions, changed) = rt.tick(&triggers, at(3), true, false, read);
        assert_eq!(actions.len(), 1);
        assert!(changed);
        assert_eq!(mode_of(&rt, 7), Some(true));
        // Retour : inactif + action de retour.
        *state.borrow_mut() = "on";
        let (actions, _) = rt.tick(&triggers, at(4), true, false, read);
        assert_eq!(actions, vec![("P".into(), String::new(), 2)]);
        assert_eq!(mode_of(&rt, 7), Some(false));
    }
    #[test]
    fn mode_defaults_to_target_key_and_cancel_stays_inactive() {
        let triggers = vec![mode_trigger(None)];
        let mut rt = Runtime::default();
        let t0 = Instant::now();
        let at = |s: u64| t0 + Duration::from_secs(s);
        let state = std::cell::RefCell::new("on");
        let read = |_: &str, _: &str| Some(state.borrow().to_string());
        rt.tick(&triggers, at(0), true, false, read);
        *state.borrow_mut() = "off";
        rt.tick(&triggers, at(1), true, false, read);
        *state.borrow_mut() = "on";
        let (actions, _) = rt.tick(&triggers, at(2), true, false, read);
        assert!(actions.is_empty(), "annulé : aucune action");
        assert_eq!(mode_of(&rt, 0), Some(false));
    }
    #[test]
    fn mode_is_active_at_startup_without_action_and_survives_disconnection() {
        let triggers = vec![mode_trigger(None)];
        let mut rt = Runtime::default();
        let t0 = Instant::now();
        let at = |s: u64| t0 + Duration::from_secs(s);
        let read = |_: &str, _: &str| Some("off".to_string());
        // Condition déjà vraie depuis le démarrage : image active, aucune action.
        let (actions, _) = rt.tick(&triggers, at(0), true, false, read);
        assert!(actions.is_empty());
        assert_eq!(mode_of(&rt, 0), Some(true));
        // Déconnecté : dernière image connue conservée, pastille « HA × ».
        let (actions, _) = rt.tick(&triggers, at(5), false, false, read);
        assert!(actions.is_empty());
        assert_eq!(mode_of(&rt, 0), Some(true));
        assert_eq!(rt.display(&("P".into(), String::new(), 0)), Some(Display::Disconnected));
        // Reconnexion, condition toujours vraie : toujours actif, aucune action.
        let (actions, _) = rt.tick(&triggers, at(6), true, false, read);
        assert!(actions.is_empty());
        assert_eq!(mode_of(&rt, 0), Some(true));
    }
    #[test]
    fn mode_pause_keeps_following_the_real_state_without_actions() {
        let triggers = vec![mode_trigger(None)];
        let mut rt = Runtime::default();
        let t0 = Instant::now();
        let at = |s: u64| t0 + Duration::from_secs(s);
        let state = std::cell::RefCell::new("on");
        let read = |_: &str, _: &str| Some(state.borrow().to_string());
        rt.tick(&triggers, at(0), true, true, read);
        *state.borrow_mut() = "off";
        rt.tick(&triggers, at(1), true, true, read);
        let (actions, _) = rt.tick(&triggers, at(4), true, true, read);
        assert!(actions.is_empty(), "pause : action bloquée");
        assert_eq!(mode_of(&rt, 0), Some(true), "l'image suit l'état réel");
        *state.borrow_mut() = "on";
        let (actions, _) = rt.tick(&triggers, at(5), true, true, read);
        assert!(actions.is_empty(), "pause : pas d'action de retour");
        assert_eq!(mode_of(&rt, 0), Some(false));
    }
    #[test]
    fn mode_option_is_off_by_default_and_validated() {
        let r: Rule = serde_json::from_str(r#"{"entity":"sensor.t"}"#).unwrap();
        assert!(!r.show_mode && r.display_key.is_none());
        assert!(!serde_json::to_string(&r).unwrap().contains("show_mode"));
        let mut rt = Runtime::default();
        let plain = crate::desktop::Trigger { ha: Some(rule(0)), ..mode_trigger(None) };
        rt.tick(&[plain], Instant::now(), true, false, |_: &str, _: &str| Some("off".to_string()));
        assert!(rt.modes.is_empty());
        assert!(Rule { display_key: Some(15), ..rule(0) }.validate(0).is_err());
        assert!(Rule { display_key: Some(14), ..rule(0) }.validate(0).is_ok());
    }
    #[test]
    fn runtime_executes_targets_and_reports_display_changes() {
        let trigger = crate::desktop::Trigger {
            profile: "P".into(),
            page: String::new(),
            key: 0,
            keys: String::new(),
            kind: "ha_state".into(),
            value: String::new(),
            days: vec![],
            minutes: 0,
            input: false,
            ha: Some(rule(2)),
        };
        let triggers = vec![trigger];
        let mut rt = Runtime::default();
        let t0 = Instant::now();
        let at = |s: u64| t0 + Duration::from_secs(s);
        let state = std::cell::RefCell::new("on");
        let read = |_: &str, _: &str| Some(state.borrow().to_string());
        assert_eq!(rt.tick(&triggers, at(0), true, false, read), (vec![], true));
        *state.borrow_mut() = "off";
        assert_eq!(rt.tick(&triggers, at(1), true, false, read), (vec![], true));
        assert_eq!(
            rt.display(&("P".into(), String::new(), 0)),
            Some(Display::Countdown(2))
        );
        let (_, changed) = rt.tick(
            &triggers,
            at(1) + Duration::from_millis(300),
            true,
            false,
            read,
        );
        assert!(!changed, "même seconde affichée : pas de nouvelle image");
        assert_eq!(
            rt.tick(&triggers, at(3), true, false, read),
            (vec![("P".into(), String::new(), 0)], true)
        );
        *state.borrow_mut() = "on";
        assert_eq!(
            rt.tick(&triggers, at(4), true, false, read).0,
            vec![("P".into(), String::new(), 2)],
            "action de retour"
        );
        assert_eq!(
            rt.tick(&triggers, at(5), false, false, read),
            (vec![], true)
        );
        assert_eq!(
            badge(rt.display(&("P".into(), String::new(), 0)).unwrap()).as_deref(),
            Some("HA ×")
        );
        assert_eq!(badge_color("12 s"), Some([214, 140, 20]));
        assert_eq!(badge_color("REC"), None);
    }
    #[test]
    fn backoff_grows_and_is_capped() {
        let d: Vec<_> = (0..8).map(|a| backoff(a, false).as_secs()).collect();
        assert_eq!(d, [1, 2, 5, 10, 30, 60, 60, 60]);
        assert!(backoff(0, true) >= Duration::from_secs(300));
    }

    // ------------------------------------------------ faux serveur Home Assistant
    use std::net::TcpListener;
    struct Fake {
        port: u16,
        states: Arc<Mutex<Vec<Value>>>,
        events: Sender<Value>,
        kick: Arc<std::sync::atomic::AtomicBool>,
        sessions: Arc<std::sync::atomic::AtomicUsize>,
    }
    fn state(id: &str, s: &str) -> Value {
        json!({"entity_id": id, "state": s, "attributes": {"friendly_name": id.to_uppercase()}})
    }
    fn fake(token: &'static str) -> Fake {
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let states = Arc::new(Mutex::new(vec![state("binary_sensor.test_presence", "on")]));
        let (events, rx) = mpsc::channel::<Value>();
        let rx = Arc::new(Mutex::new(rx));
        let kick = Arc::new(AtomicBool::new(false));
        let sessions = Arc::new(AtomicUsize::new(0));
        let (st, k, n) = (states.clone(), kick.clone(), sessions.clone());
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                let (st, k, n, rx) = (st.clone(), k.clone(), n.clone(), rx.clone());
                std::thread::spawn(move || {
                    let mut ws = tungstenite::accept(stream).unwrap();
                    ws.send(Message::Text(
                        json!({"type":"auth_required","ha_version":"test"})
                            .to_string()
                            .into(),
                    ))
                    .unwrap();
                    let auth: Value =
                        serde_json::from_str(&ws.read().unwrap().into_text().unwrap()).unwrap();
                    if auth["access_token"] != token {
                        let _ = ws.send(Message::Text(json!({"type":"auth_invalid","message":"Invalid access token or password"}).to_string().into()));
                        return;
                    }
                    ws.send(Message::Text(json!({"type":"auth_ok"}).to_string().into()))
                        .unwrap();
                    n.fetch_add(1, Ordering::SeqCst);
                    ws.get_mut()
                        .set_read_timeout(Some(Duration::from_millis(30)))
                        .unwrap();
                    loop {
                        if k.swap(false, Ordering::SeqCst) {
                            return;
                        } // coupure brutale
                        match ws.read() {
                            Ok(Message::Text(t)) => {
                                let m: Value = serde_json::from_str(&t).unwrap();
                                let reply = match m["type"].as_str().unwrap() {
                                    "get_states" => {
                                        json!({"id": m["id"], "type":"result","success":true,"result": st.lock().unwrap().clone()})
                                    }
                                    "ping" => json!({"id": m["id"], "type":"pong"}),
                                    _ => {
                                        json!({"id": m["id"], "type":"result","success":true,"result":null})
                                    }
                                };
                                if ws.send(Message::Text(reply.to_string().into())).is_err() {
                                    return;
                                }
                            }
                            Err(tungstenite::Error::Io(ref e))
                                if matches!(
                                    e.kind(),
                                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                                ) => {}
                            Err(_) => return,
                            _ => {}
                        }
                        while let Ok(new_state) = rx.lock().unwrap().try_recv() {
                            let id = new_state["entity_id"].clone();
                            let ev = json!({"id":1,"type":"event","event":{"event_type":"state_changed","data":{"entity_id": id, "new_state": new_state}}});
                            if ws.send(Message::Text(ev.to_string().into())).is_err() {
                                return;
                            }
                        }
                    }
                });
            }
        });
        Fake {
            port,
            states,
            events,
            kick,
            sessions,
        }
    }
    fn wait(what: &str, mut f: impl FnMut() -> bool) {
        let end = Instant::now() + Duration::from_secs(10);
        while !f() {
            assert!(Instant::now() < end, "délai dépassé : {what}");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    const FAST: Timing = Timing {
        connect: Duration::from_secs(3),
        keepalive: Duration::from_millis(300),
        silence: Duration::from_secs(3),
        poll: Duration::from_millis(20),
        scale_ms: 20,
    };

    #[test]
    fn fake_server_connection_test_and_auth() {
        let f = fake("good-token");
        let url = format!("http://127.0.0.1:{}", f.port);
        assert_eq!(
            test_connection(&url, &Secret::new("good-token"), Duration::from_secs(3)).unwrap(),
            1
        );
        let err = test_connection(&url, &Secret::new("bad-token-XYZ"), Duration::from_secs(3))
            .unwrap_err();
        let text = format!("{err:#}");
        assert!(text.contains("refusé") && !text.contains("XYZ"), "{text}");
    }
    #[test]
    fn fake_server_live_events_reconnect_and_reload() {
        use std::sync::atomic::Ordering;
        let f = fake("good-token");
        let monitor = Monitor::spawn_with(FAST);
        assert_eq!(monitor.timing().scale_ms, 20);
        monitor.configure(Some(Settings {
            url: format!("http://127.0.0.1:{}", f.port),
            token: Secret::new("good-token"),
        }));
        wait("connexion", || monitor.status().connected());
        assert_eq!(
            monitor.entity_list(),
            vec![(
                "binary_sensor.test_presence".to_owned(),
                "BINARY_SENSOR.TEST_PRESENCE".to_owned()
            )]
        );
        f.events
            .send(state("binary_sensor.test_presence", "off"))
            .unwrap();
        wait("événement poussé", || {
            monitor.read(|s| s.entities["binary_sensor.test_presence"].state == "off")
        });
        // Coupure : un état change pendant l'absence, la relecture complète le rattrape.
        *f.states.lock().unwrap() = vec![
            state("binary_sensor.test_presence", "on"),
            state("sensor.test_temp", "21.5"),
        ];
        f.kick.store(true, Ordering::SeqCst);
        wait("détection de la coupure", || !monitor.status().connected());
        wait("reconnexion", || {
            monitor.status().connected() && f.sessions.load(Ordering::SeqCst) >= 2
        });
        wait("relecture", || {
            monitor.read(|s| {
                s.entities.len() == 2 && s.entities["binary_sensor.test_presence"].state == "on"
            })
        });
        monitor.configure(None);
        wait("désactivation", || {
            monitor.status() == Status::Disabled && monitor.entity_list().is_empty()
        });
    }
    #[test]
    fn fake_server_rejected_token_status() {
        let f = fake("good-token");
        let monitor = Monitor::spawn_with(FAST);
        monitor.configure(Some(Settings {
            url: format!("http://127.0.0.1:{}", f.port),
            token: Secret::new("nope"),
        }));
        wait("refus", || {
            matches!(monitor.status(), Status::AuthFailed(_))
        });
        assert!(!format!("{:?}", monitor.status()).contains("nope"));
    }
    #[test]
    fn unreachable_server_reports_reconnecting() {
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let monitor = Monitor::spawn_with(FAST);
        monitor.configure(Some(Settings {
            url: format!("http://127.0.0.1:{port}"),
            token: Secret::new("t"),
        }));
        wait(
            "tentatives",
            || matches!(monitor.status(), Status::Reconnecting { attempt, .. } if attempt >= 2),
        );
    }
    #[test]
    fn secret_and_settings_debug_never_show_the_token() {
        let secret = Secret::new("jeton-super-secret");
        let settings = Settings {
            url: "http://ha:8123".into(),
            token: secret.clone(),
        };
        for text in [
            format!("{secret:?}"),
            format!("{settings:?}"),
            format!("{:#?}", Some(settings)),
        ] {
            assert!(!text.contains("jeton-super-secret"), "{text}");
        }
        let prefs = Prefs {
            url: "http://ha:8123".into(),
            ..Prefs::default()
        };
        assert!(!serde_json::to_string(&prefs)
            .unwrap()
            .contains("jeton-super-secret"));
    }
}
