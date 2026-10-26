mod intent_parser;
#[cfg(test)]
mod plugin_request_corpus;
mod proposal;
mod structured;
mod templates;
mod validator;

#[allow(unused_imports)] // API préparée pour le chat de la phase suivante.
pub use proposal::{Missing, Proposal, Status, Suggestion};
#[allow(unused_imports)] // API destinée au futur chat.
pub use templates::common as common_templates;
#[allow(unused_imports)] // API destinée au futur chat.
pub use validator::{diagnose, explain};

use crate::model::{Config, Step};
use intent_parser as parser;

// Ordre : expressions spécialisées avant les intentions génériques.
const INTENTS: &[(&str, &[&str])] = &[
    (
        "obs_scene",
        &["passe sur", "scene obs", "scene ", "active la scene"],
    ),
    (
        "obs_record",
        &["enregistrement", "enregistrer obs", "record obs"],
    ),
    ("obs_stream", &["stream", "diffusion obs", "diffuse obs"]),
    ("obs_mic", &["micro obs", "micro dans obs"]),
    (
        "app_volume",
        &[
            "volume de l application",
            "volume de l appli",
            "volume appli",
            "volume application active",
        ],
    ),
    (
        "volume",
        &["volume", "son general", "mets en muet", "coupe le son"],
    ),
    ("wait", &["attends", "attendre", "patiente", "pause de", "wait", "sleep", "delay"]),
    (
        "launch",
        &[
            "lance ",
            "lancer ",
            "ouvre l application",
            "ouvre l appli",
            "demarre ",
        ],
    ),
    (
        "url",
        &[
            "ouvre http",
            "ouvre https",
            "ouvrir http",
            "ouvrir https",
            "ouvre le site",
            "ouvre l url",
            "open http",
            "open https",
            "go to http",
            "go to https",
        ],
    ),
    (
        "media",
        &[
            "lecture",
            "pause musique",
            "musique suivante",
            "piste suivante",
            "piste precedente",
            "morceau suivant",
            "morceau precedent",
        ],
    ),
    ("text", &["tape ", "saisis ", "ecris ", "insere le texte", "type ", "write "]),
    ("notify", &["notification", "notifie", "affiche un message"]),
    ("repeat", &["repete ", "rejoue "]),
    ("page_next", &["page suivante", "prochaine page"]),
    ("page_previous", &["page precedente"]),
    ("back", &["retour au dossier"]),
    ("profile", &["profil ", "change de profil"]),
    ("brightness", &["luminosite"]),
    (
        "clipboard",
        &["presse papiers", "copie le texte", "copier le texte"],
    ),
    ("close", &["ferme la fenetre", "fermer la fenetre"]),
];

fn contains_phrase(haystack: &str, phrase: &str) -> bool {
    // Ne pas reconnaître « volume » dans un autre mot.
    haystack.match_indices(phrase).any(|(i, _)| {
        (i == 0 || !haystack.as_bytes()[i - 1].is_ascii_alphanumeric())
            && (phrase.ends_with(' ')
                || haystack
                    .as_bytes()
                    .get(i + phrase.len())
                    .is_none_or(|c| !c.is_ascii_alphanumeric()))
    })
}

fn has(n: &str, words: &[&str]) -> bool {
    words.iter().any(|w| contains_phrase(n, w))
}
fn window_op(op: &str) -> Step {
    Step::WindowOp { target: "foreground".into(), value: String::new(), op: op.into(), x: 0, y: 0, w: 800, h: 600, monitor: 2 }
}
/// Formulations naturelles (français et anglais) des actions Windows courantes, avant les
/// intentions par mots-clés. Retourne None si rien ne correspond.
fn natural(n: &str) -> Option<Step> {
    let windows = has(n, &["fenetre", "fenetres", "window", "windows", "applis", "applications", "programmes"]);
    let all = has(n, &["toutes", "toute", "tous", "tout", "all", "every", "autres", "other", "others"]);
    let except_active = has(n, &["sauf", "excepte", "hormis", "a part", "except", "but", "other", "others", "autres"])
        || has(n, &["celle sur laquelle", "celle ou je suis", "active", "actuelle", "courante", "en cours", "current", "focused"]);
    let close = has(n, &["ferme", "fermer", "fermez", "quitte", "quitter", "close", "kill"]);
    let minimize = has(n, &["reduis", "reduire", "reduit", "minimise", "minimiser", "minimize", "cache", "cacher", "hide"]);
    if windows && all && except_active && close { return Some(window_op("close_others")); }
    if windows && all && except_active && minimize { return Some(window_op("minimize_others")); }
    if (windows && all && minimize) || has(n, &["affiche le bureau", "afficher le bureau", "montre le bureau", "show desktop", "show the desktop"]) {
        return Some(Step::Hotkey { keys: "WIN+D".into() });
    }
    let this_window = windows && !all;
    if this_window && close { return Some(Step::CloseWindow); }
    if this_window && minimize { return Some(window_op("minimize")); }
    if this_window && has(n, &["agrandis", "agrandir", "maximise", "maximiser", "plein ecran", "maximize"]) { return Some(window_op("maximize")); }
    if this_window && has(n, &["restaure", "restaurer", "restore"]) { return Some(window_op("restore")); }
    if this_window && has(n, &["toujours devant", "toujours au premier plan", "always on top", "epingle", "pin"]) { return Some(window_op("toggle_always_on_top")); }
    if this_window && has(n, &["ecran", "moniteur", "screen", "monitor"]) && has(n, &["deplace", "deplacer", "envoie", "bouge", "move", "send"]) {
        let monitor = n.split_whitespace().find_map(|w| w.parse::<u32>().ok()).filter(|m| (1..=16).contains(m)).unwrap_or(2);
        return Some(Step::WindowOp { target: "foreground".into(), value: String::new(), op: "move_to_monitor".into(), x: 0, y: 0, w: 800, h: 600, monitor });
    }
    let shortcut = |keys: &str| Some(Step::Hotkey { keys: keys.into() });
    if has(n, &["capture d ecran", "capture ecran", "screenshot", "copie d ecran", "outil capture"]) { return shortcut("WIN+SHIFT+S"); }
    if has(n, &["verrouille", "verrouiller", "lock the pc", "lock pc", "lock my pc", "lock screen"]) { return shortcut("WIN+L"); }
    if has(n, &["gestionnaire des taches", "gestionnaire de taches", "task manager"]) { return shortcut("CTRL+SHIFT+ESC"); }
    if has(n, &["explorateur", "explorer", "mes fichiers", "file explorer"]) { return shortcut("WIN+E"); }
    if has(n, &["parametres windows", "reglages windows", "windows settings"]) { return shortcut("WIN+I"); }
    if has(n, &["bureau virtuel suivant", "next desktop", "next virtual desktop"]) { return shortcut("CTRL+WIN+RIGHT"); }
    if has(n, &["bureau virtuel precedent", "previous desktop", "previous virtual desktop"]) { return shortcut("CTRL+WIN+LEFT"); }
    if has(n, &["copier", "copie", "copy"]) && !has(n, &["texte", "text", "presse papiers", "clipboard"]) { return shortcut("CTRL+C"); }
    // Navigateur (s'applique à la fenêtre active).
    for (words, key) in [
        (&["rouvre l onglet", "rouvrir l onglet", "reopen the tab", "reopen closed tab"][..], "CTRL+SHIFT+T"),
        (&["navigation privee", "fenetre privee", "private window", "incognito"][..], "CTRL+SHIFT+N"),
        (&["nouvel onglet", "nouveau onglet", "new tab"][..], "CTRL+T"),
        (&["nouvelle fenetre", "new window"][..], "CTRL+N"),
        (&["ferme l onglet", "ferme cet onglet", "fermer l onglet", "close the tab", "close tab"][..], "CTRL+W"),
        (&["recharge la page", "actualise la page", "rafraichis la page", "recharge", "reload", "refresh the page"][..], "F5"),
    ] {
        if has(n, words) { return shortcut(key); }
    }
    for (words, key) in [
        (&["fleche du bas", "fleche bas", "down arrow", "arrow down", "touche bas"][..], "DOWN"),
        (&["fleche du haut", "fleche haut", "up arrow", "arrow up", "touche haut"][..], "UP"),
        (&["fleche de gauche", "fleche gauche", "left arrow", "arrow left"][..], "LEFT"),
        (&["fleche de droite", "fleche droite", "right arrow", "arrow right"][..], "RIGHT"),
        (&["touche tab", "appuie sur tab", "press tab"][..], "TAB"),
        (&["touche espace", "barre d espace", "press space", "spacebar"][..], "SPACE"),
    ] {
        if has(n, words) { return shortcut(key); }
    }
    if has(n, &["mets la musique en pause", "met la musique en pause", "mets en pause", "met en pause", "mets pause", "met pause", "pause la musique", "pause the music", "pause music", "play pause"]) {
        return Some(Step::Media { command: "play_pause".into() });
    }
    if has(n, &["appuie sur entree", "appuyer sur entree", "touche entree", "valide avec entree", "press enter", "hit enter", "press return"]) { return shortcut("ENTER"); }
    if has(n, &["appuie sur echap", "touche echap", "press escape", "press esc", "hit escape"]) { return shortcut("ESC"); }
    if has(n, &["coller", "colle", "paste"]) { return shortcut("CTRL+V"); }
    if has(n, &["annuler", "annule", "undo"]) { return shortcut("CTRL+Z"); }
    if has(n, &["tout selectionner", "selectionne tout", "select all"]) { return shortcut("CTRL+A"); }
    if has(n, &["enregistre le fichier", "sauvegarde le fichier", "save the file", "save file"]) { return shortcut("CTRL+S"); }
    if has(n, &["change de fenetre", "fenetre suivante", "switch window", "alt tab"]) { return shortcut("ALT+TAB"); }
    // Anglais pour les intentions existantes.
    if has(n, &["play pause", "pause the music", "play the music", "resume music"]) { return Some(Step::Media { command: "play_pause".into() }); }
    if has(n, &["next track", "next song", "skip song"]) { return Some(Step::Media { command: "next".into() }); }
    if has(n, &["previous track", "previous song"]) { return Some(Step::Media { command: "previous".into() }); }
    if has(n, &["next page"]) { return Some(Step::PageNext); }
    if has(n, &["previous page"]) { return Some(Step::PagePrevious); }
    None
}

/// Demandes qui correspondent à plusieurs blocs : on reprend le préréglage du catalogue.
fn multi_step(n: &str) -> Option<Vec<Step>> {
    let plain = has(n, &["sans mise en forme", "sans formatage", "sans format", "texte brut", "plain text", "without formatting", "unformatted"]);
    if plain && has(n, &["coller", "colle", "paste"]) {
        return crate::catalog::presets().into_iter().find(|p| p.key.label == "Coller sans mise en forme").map(|p| p.key.steps);
    }
    None
}

fn clarification(question: &str, choices: &[&str]) -> Status {
    Status::Ambigue {
        question: question.into(),
        choix: choices.iter().map(|s| (*s).into()).collect(),
        choix_autonomes: false,
    }
}

fn parse_one(text: &str) -> (Option<Step>, Option<Status>, Vec<String>) {
    let normalized = parser::normalize(text);
    let n = normalized.as_str();
    if let Some(keys) = parser::hotkey(text) {
        return (Some(Step::Hotkey { keys }), None, vec![]);
    }
    if let Some(step) = natural(n) {
        let notes = if matches!(&step, Step::WindowOp { op, .. } if op == "close_others") {
            vec!["Les applications reçoivent une demande de fermeture et peuvent proposer d'enregistrer ; NovyDeck et la fenêtre active restent ouverts.".into()]
        } else { vec![] };
        return (Some(step), None, notes);
    }
    let kind = INTENTS
        .iter()
        .find(|(_, aliases)| aliases.iter().any(|alias| contains_phrase(n, alias)))
        .map(|(kind, _)| *kind);
    let name = || {
        parser::quoted_or_after(
            text,
            &[
                " sur la scène ",
                " sur la scene ",
                " sur ",
                " scène ",
                " scene ",
            ],
        )
    };
    let step = match kind {
        Some("obs_scene") => Step::ObsScene {
            name: name().unwrap_or_default(),
        },
        Some("obs_record") => Step::ObsRecord { mode: obs_mode(n) },
        Some("obs_stream") => Step::ObsStream { mode: obs_mode(n) },
        Some("obs_mic") => Step::ObsMute {
            input: "Micro/Aux".into(),
            mode: if n.contains("active") || n.contains("retabl") {
                "unmute"
            } else if n.contains("coupe") || n.contains("muet") {
                "mute"
            } else {
                "toggle"
            }
            .into(),
        },
        Some("volume" | "app_volume") => {
            let explicit_app = kind == Some("app_volume")
                || n.contains("application active")
                || n.contains("appli active");
            let explicit_general = n.contains("general") || n.contains("systeme");
            if !explicit_app && !explicit_general {
                return (
                    None,
                    Some(clarification(
                        "Volume de quoi : général ou application active ?",
                        &["Général", "Application active"],
                    )),
                    vec![],
                );
            }
            let down = ["baisse", "diminue", "reduis"]
                .iter()
                .any(|w| n.contains(w));
            let up = ["augmente", "monte"].iter().any(|w| n.contains(w));
            let mute = n.contains("muet") || n.contains("coupe");
            let action = if mute {
                "mute"
            } else if down {
                "down"
            } else if up {
                "up"
            } else {
                "set"
            };
            let pct = parser::percent(text);
            if !mute && pct.is_none() {
                return (
                    None,
                    Some(clarification(
                        "De combien de pour cent faut-il régler le volume ?",
                        &["Indiquer un pourcentage entre 1 et 100"],
                    )),
                    vec![],
                );
            }
            let p = pct.unwrap_or(5);
            if explicit_app {
                Step::AppVolume {
                    target: "foreground".into(),
                    action: action.into(),
                    step: p,
                    level: p,
                    include_children: true,
                }
            } else {
                Step::DeviceVolume {
                    input: false,
                    device: "default".into(),
                    action: action.into(),
                    step: p,
                    level: p,
                }
            }
        }
        Some("wait") => match parser::duration(text) {
            Some(ms) => Step::Delay { ms },
            None => {
                return (
                    None,
                    Some(clarification(
                        "Combien de temps faut-il attendre (ms, secondes ou minutes) ?",
                        &["Indiquer une durée"],
                    )),
                    vec![],
                )
            }
        },
        Some("launch") => Step::Launch {
            program: String::new(),
            args: vec![],
            focus_if_running: false,
        },
        Some("url") => {
            let url = text
                .split_whitespace()
                .find(|s| s.starts_with("http://") || s.starts_with("https://"));
            match url {
                Some(url) => Step::Url {
                    url: url.trim_end_matches(['.', ',', '!']).into(),
                },
                None => {
                    return (
                        None,
                        Some(clarification(
                            "Quelle adresse HTTP ou HTTPS ouvrir ?",
                            &["Indiquer l'URL complète"],
                        )),
                        vec![],
                    )
                }
            }
        }
        Some("media") => {
            let command = if n.contains("suivant") {
                "next"
            } else if n.contains("precedent") {
                "previous"
            } else if n.contains("arrete") {
                "stop"
            } else {
                "play_pause"
            };
            Step::Media {
                command: command.into(),
            }
        }
        Some("text") => Step::Text {
            text: parser::quoted_or_after(
                text,
                &["tape ", "saisis ", "écris ", "ecris ", "texte ", "type ", "write "],
            )
            .unwrap_or_default(),
        },
        Some("notify") => Step::Notify {
            title: "NovyDeck".into(),
            message: parser::quoted_or_after(text, &["message ", "notification ", "notifie "])
                .unwrap_or_default(),
        },
        Some("repeat") => match parser::number(text) {
            Some(count) if (1..=1000).contains(&count) => Step::Repeat {
                count,
                steps: vec![],
            },
            _ => {
                return (
                    None,
                    Some(clarification(
                        "Combien de fois répéter (1 à 1000) ?",
                        &["Indiquer le nombre"],
                    )),
                    vec![],
                )
            }
        },
        Some("page_next") => Step::PageNext,
        Some("page_previous") => Step::PagePrevious,
        Some("back") => Step::Back,
        Some("profile") => Step::Profile {
            name: parser::quoted_or_after(text, &["profil "]).unwrap_or_default(),
        },
        Some("brightness") => {
            let mode = if n.contains("baisse") || n.contains("diminue") {
                "down"
            } else if n.contains("augmente") || n.contains("monte") {
                "up"
            } else {
                "set"
            };
            match parser::percent(text) {
                Some(value) => Step::D6Brightness {
                    mode: mode.into(),
                    value,
                },
                None => {
                    return (
                        None,
                        Some(clarification(
                            "Quelle variation de luminosité en pour cent ?",
                            &["Indiquer un pourcentage"],
                        )),
                        vec![],
                    )
                }
            }
        }
        Some("clipboard") => {
            if n.contains("efface") || n.contains("vide") {
                Step::Clipboard {
                    op: "clear".into(),
                    value: String::new(),
                    name: "texte".into(),
                }
            } else {
                Step::Clipboard {
                    op: "copy_text".into(),
                    value: parser::quoted_or_after(text, &["texte "]).unwrap_or_default(),
                    name: "texte".into(),
                }
            }
        }
        Some("close") => Step::CloseWindow,
        _ => return (None, Some(Status::Inconnue), vec![]),
    };
    let notes = if kind == Some("launch") {
        vec![format!(
            "Application « {} » : indique son chemin exact ; aucun .exe n'est inventé.",
            parser::quoted_or_after(
                text,
                &["lance ", "lancer ", "demarre ", "application ", "appli "]
            )
            .unwrap_or_default()
        )]
    } else {
        vec![]
    };
    (Some(step), None, notes)
}

fn obs_mode(n: &str) -> String {
    if n.contains("arrete") || n.contains("stop") {
        "stop"
    } else if n.contains("demarre") || n.contains("lance") || n.contains("commence") {
        "start"
    } else if n.contains("pause") {
        "pause"
    } else {
        "toggle"
    }
    .into()
}

/// Proposition issue de l'IA locale : mêmes contrôles que les règles (champs manquants, profils…).
pub fn from_model(steps: Vec<Step>, config: &Config) -> Proposal {
    let status = if steps.is_empty() { Status::Inconnue } else { Status::Reconnue };
    validator::validate(
        Proposal {
            steps,
            notes: vec!["Proposé par l'IA locale : vérifie chaque bloc avant d'ajouter.".into()],
            missing: vec![],
            status,
        },
        config,
    )
}

/// Analyse pure : aucune exécution ni mutation de la configuration.
pub fn propose(text: &str, config: &Config) -> Proposal {
    propose_with(text, config, &crate::local_llm::Knowledge::builtin())
}

const OPEN_VERBS: &[&str] = &["ouvre", "ouvrir", "lance", "lancer", "demarre", "demarrer", "open", "launch", "start"];

/// La clause ne contient que « ouvre » + le nom de l'application (« démarre l'enregistrement OBS » n'en est pas une).
fn only_opens(n: &str, app: &crate::local_llm::KnownApp) -> bool {
    let names: Vec<String> = app.noms.iter().map(|name| parser::normalize(name)).chain(std::iter::once(parser::normalize(&app.exe))).collect();
    let mut rest = format!(" {n} ");
    for name in names.iter().filter(|n| !n.is_empty()) {
        rest = rest.replace(&format!(" {name} "), " ");
    }
    rest.split_whitespace().all(|w| OPEN_VERBS.contains(&w) || ["le", "la", "l", "les", "un", "une", "moi", "stp", "please", "the", "app", "application", "appli", "logiciel", "programme", "navigateur", "exe"].contains(&w))
}

/// Comme [`propose`], avec les applications connues du fichier `ia-locale.json`.
/// Une demande conditionnelle ou répétée est soit comprise en entier, soit laissée à l'IA
/// locale (statut inconnu) : jamais de proposition partielle.
pub fn propose_with(text: &str, config: &Config, knowledge: &crate::local_llm::Knowledge) -> Proposal {
    let n = parser::normalize(text);
    let rules = |t: &str| {
        let p = propose_plain(t, config, knowledge);
        matches!(p.status, Status::Reconnue).then_some(p.steps)
    };
    let structured = if structured::is_conditional(&n) {
        Some(structured::condition(text, knowledge, &rules))
    } else if structured::is_repetition(&n) {
        // « répète 3 fois » seul : les règles ordinaires créent la répétition et demandent quoi répéter.
        structured::repetition(text, &rules).map(|step| Some((step, "Répétition : vérifie le nombre de tours avant d'ajouter.".to_owned())))
    } else {
        None
    };
    match structured {
        Some(Some((step, note))) => validator::validate(
            Proposal { steps: vec![step], notes: vec![note], missing: vec![], status: Status::Reconnue },
            config,
        ),
        Some(None) => Proposal {
            steps: vec![],
            notes: vec!["Demande avec condition ou répétition non comprise par les règles : l'IA locale peut la traduire (ou reformule : « si Brave est ouvert, … sinon … », « appuie 3 fois sur … »).".into()],
            missing: vec![],
            status: Status::Inconnue,
        },
        None => propose_plain(text, config, knowledge),
    }
}

/// Choix complets, jamais une action implicite : partagés entre règles et IA locale.
fn plugin_choices(items: &[(&crate::plugins::Plugin, &crate::plugins::Action)], unsafe_target: bool) -> Vec<String> {
    items.iter().filter(|(p, a)| !unsafe_target && p.inspector.is_none() && a.inspector.is_none()).take(6).map(|(p, a)| {
        if crate::i18n::current() == "en" {
            format!("Launch {} from {}", a.name, p.display_name())
        } else {
            format!("Lance {} de {}", a.name, p.display_name())
        }
    }).collect()
}

/// Un refus par le garde-fou annule toute la macro, y compris les blocs acceptés.
pub fn from_model_guarded(steps: Vec<Step>, rejected: Vec<(String, String)>, config: &Config, knowledge: &crate::local_llm::Knowledge) -> Proposal {
    if rejected.is_empty() { return from_model(steps, config); }
    let available = |p: &crate::plugins::Plugin| p.prerequisite.is_none()
        && matches!(p.runtime, crate::plugins::Runtime::Exe(_) | crate::plugins::Runtime::Node(_) | crate::plugins::Runtime::Html(_));
    let unknown = rejected.iter().any(|(id, _)| !knowledge.plugins.iter().any(|p| p.uuid == *id && available(p)));
    let (question, choix) = if unknown {
        (if crate::i18n::current() == "en" { "The proposed plugin is not installed or available; no blocks were added." } else { "Le plugin proposé n'est pas installé ou disponible ; aucun bloc n'a été ajouté." }.to_owned(), vec![])
    } else {
        let (id, action) = &rejected[0];
        let plugin = knowledge.plugins.iter().find(|p| p.uuid == *id).expect("plugin vérifié");
        let mut actions: Vec<_> = plugin.actions.iter().collect();
        actions.sort_by_key(|a| if a.uuid == *action { 0 } else { 1 });
        let items: Vec<_> = actions.iter().map(|a| (plugin, *a)).collect();
        (if crate::i18n::current() == "en" {
            format!("Which action from '{}' did you mean? The model's plugin action was rejected (name or settings); no blocks were added.", plugin.display_name())
        } else {
            format!("Quelle action de « {} » voulais-tu ? L'action proposée par le modèle a été refusée (nom ou réglages) ; aucun bloc n'a été ajouté.", plugin.display_name())
        }, plugin_choices(&items, false))
    };
    Proposal { steps: vec![], notes: vec![], missing: vec![], status: Status::Ambigue { question, choix, choix_autonomes: true } }
}

fn plugin_intent(clause: &str, plugins: &[crate::plugins::Plugin]) -> Option<Result<Step, Status>> {
    let n = parser::normalize(clause);
    let named = |name: &str| !name.trim().is_empty() && contains_phrase(&n, &parser::normalize(name));
    let available: Vec<_> = plugins.iter().filter(|p| p.prerequisite.is_none() && matches!(p.runtime, crate::plugins::Runtime::Exe(_) | crate::plugins::Runtime::Node(_) | crate::plugins::Runtime::Html(_))).collect();
    let mentioned: Vec<_> = available.iter().copied().filter(|p| named(p.display_name()) || named(&p.uuid)).collect();
    let pool = if mentioned.is_empty() { &available } else { &mentioned };
    let exact: Vec<_> = pool.iter().flat_map(|p| p.actions.iter().filter(|a| named(&a.name) || named(&a.uuid)).map(move |a| (*p, a))).collect();
    // Une partie d'un nom d'action ne suffit jamais à insérer. Seul le nom complet
    // (éventuellement avec les mots dans un autre ordre) est une preuve suffisante.
    let partial: Vec<_> = if exact.is_empty() && !mentioned.is_empty() {
        pool.iter().flat_map(|p| p.actions.iter().filter(|a| {
            let action = parser::normalize(&a.name);
            action.split_whitespace().any(|word| word.len() > 2 && contains_phrase(&n, word))
        }).map(move |a| (*p, a))).collect()
    } else { vec![] };
    let complete: Vec<_> = partial.iter().copied().filter(|(_, a)| {
        parser::normalize(&a.name).split_whitespace().filter(|w| w.len() > 2).all(|w| contains_phrase(&n, w))
    }).collect();
    let candidates = if !exact.is_empty() { &exact } else { &partial };
    // Une action nommée ne suffit pas à autoriser une planification différée :
    // aucune étape de calendrier n'est déduite implicitement d'un adverbe temporel.
    if !candidates.is_empty() && has(&n, &["tomorrow", "demain", "next week", "la semaine prochaine", "tonight", "ce soir"]) {
        return Some(Err(Status::Inconnue));
    }
    let unsafe_target = has(&n, &["sur mon", "sur ma", "sur le", "sur la", "sur l app", "sur un", "sur une", "for my", "on my"]);
    let question = |fr: String, en: String, items: &[(&crate::plugins::Plugin, &crate::plugins::Action)]| Status::Ambigue {
        question: if crate::i18n::current() == "en" { en } else { fr },
        choix: plugin_choices(items, unsafe_target),
        choix_autonomes: true,
    };
    // Une qualification de plugin non installé, même après un nom d'action unique,
    // interdit de choisir silencieusement son homonyme installé.
    let unknown_qualifier = mentioned.is_empty() && exact.iter().any(|(_, a)| {
        [a.name.as_str(), a.uuid.as_str()].iter().any(|name| {
            let token = parser::normalize(name);
            n.split_once(&token).is_some_and(|(_, rest)| ["de ", "from ", "avec ", "with ", "sur "]
                .iter().any(|prefix| rest.trim_start().starts_with(prefix)))
        })
    });
    if unknown_qualifier {
        return Some(Err(question("Quel plugin installé veux-tu utiliser ? Précise son nom exact et son action.".into(), "Which installed plugin do you want to use? Specify its exact name and action.".into(), &exact)));
    }
    match candidates.as_slice() {
        [(p, a)] if !exact.is_empty() || complete.len() == 1 && partial.len() == 1 => {
            // Les champs de l'inspecteur tiers ne figurent pas dans le manifeste :
            // ni appareil ni réglages implicites ne peuvent être déduits ici.
            if a.inspector.is_some() || p.inspector.is_some() {
                return Some(Err(question(format!("Quels réglages pour « {} » de « {} » ? Configure-les dans l'inspecteur du plugin ; je ne connais pas ses champs internes.", a.name, p.display_name()), format!("What settings for '{}' from '{}'? Configure them in the plugin inspector; I don't know its internal fields.", a.name, p.display_name()), &[])));
            }
            if unsafe_target {
                return Some(Err(question(format!("Quelle cible ou quel appareil pour « {} » de « {} » ? Le manifeste ne décrit pas ce réglage.", a.name, p.display_name()), format!("Which target or device for '{}' from '{}'? The manifest does not describe this setting.", a.name, p.display_name()), &[])));
            }
            Some(Ok(Step::PluginAction { plugin: p.uuid.clone(), action: a.uuid.clone(), settings: serde_json::json!({}), label: a.name.clone() }))
        }
        [] if !mentioned.is_empty() => {
            let actions: Vec<_> = mentioned.iter().flat_map(|p| p.actions.iter().map(move |a| (*p, a))).collect();
            Some(Err(question(format!("Quelle action de « {} » veux-tu utiliser ? Précise son nom parmi les actions installées.", mentioned.iter().map(|p| p.display_name()).collect::<Vec<_>>().join(", ")), format!("Which action from '{}' do you want to use? Specify its name from the installed actions.", mentioned.iter().map(|p| p.display_name()).collect::<Vec<_>>().join(", ")), &actions)))
        }
        [] => None,
        _ => Some(Err(question(format!("Quelle action de quel plugin ? Plusieurs correspondent : {}.", candidates.iter().map(|(p, a)| format!("{} — {}", p.display_name(), a.name)).collect::<Vec<_>>().join(" ; ")), format!("Which action from which plugin? Several match: {}.", candidates.iter().map(|(p, a)| format!("{} — {}", p.display_name(), a.name)).collect::<Vec<_>>().join(" ; ")), candidates))),
    }
}

fn propose_plain(text: &str, config: &Config, knowledge: &crate::local_llm::Knowledge) -> Proposal {
    let mut result = Proposal {
        steps: vec![],
        notes: vec![],
        missing: vec![],
        status: Status::Inconnue,
    };
    for clause in parser::clauses(text) {
        let cn = parser::normalize(&clause);
        if let Some(intent) = plugin_intent(&clause, &knowledge.plugins) {
            match intent {
                Ok(step) => { result.steps.push(step); continue; }
                Err(status) => {
                    result.steps.clear();
                    result.status = status;
                    return result;
                }
            }
        }
        if let Some(steps) = multi_step(&cn) {
            result.steps.extend(steps);
            continue;
        }
        // « mets Edge au premier plan » : fenêtre de l'application connue.
        if has(&cn, &["premier plan", "en avant", "bascule sur", "bascule vers", "passe sur", "switch to", "bring", "focus"]) && !has(&cn, &["toujours", "always"]) {
            if let Some(app) = knowledge.find_app(&clause) {
                result.steps.push(Step::WindowOp { target: "exe".into(), value: app.exe.clone(), op: "activate".into(), x: 0, y: 0, w: 800, h: 600, monitor: 2 });
                continue;
            }
        }
        // « ouvre Brave » : programme tiré des applications connues, jamais inventé.
        if has(&cn, OPEN_VERBS) {
            if let Some(app) = knowledge.find_app(&clause).filter(|app| only_opens(&cn, app)) {
                result.steps.push(Step::Launch { program: app.exe.clone(), args: vec![], focus_if_running: false });
                result.notes.push(format!("Programme : {} (applications connues de ia-locale.json).", app.exe));
                continue;
            }
        }
        let (step, status, notes) = parse_one(&clause);
        if let Some(status) = status {
            result.status = status;
            result.notes.push(format!("Clause non résolue : {clause}"));
            return result;
        }
        if let Some(step) = step {
            result.steps.push(step);
            result.notes.extend(notes);
        }
    }
    if !result.steps.is_empty() {
        result.status = Status::Reconnue;
    }
    validator::validate(result, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn first(text: &str) -> Proposal {
        propose(text, &Config::default())
    }
    #[test]
    fn plugin_action_named_installed_only() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
        let mut plugin = crate::plugins::parse(&folder, &std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
        plugin.name = "SoundPad".into();
        plugin.actions[0].name = "Toggle Mute".into();
        let action = plugin.actions[0].uuid.clone();
        let id = plugin.uuid.clone();
        let mut knowledge = crate::local_llm::Knowledge::builtin();
        knowledge.plugins = vec![plugin];
        let proposal = propose_with("lance l'action Toggle Mute de SoundPad", &Config::default(), &knowledge);
        assert!(matches!(&proposal.steps[..], [Step::PluginAction { plugin, action, .. }] if plugin == &id && action == &knowledge.plugins[0].actions[0].uuid), "{proposal:?}");
        assert!(!proposal.steps.iter().any(|s| matches!(s, Step::Delay { .. })));
        let unknown = propose_with("lance l'action Toggle Mute de SonInconnu", &Config::default(), &knowledge);
        assert!(!unknown.steps.iter().any(|s| matches!(s, Step::PluginAction { .. })), "{unknown:?}");
        assert!(!action.is_empty());
    }
    #[test]
    fn installed_plugin_phrase_corpus_requires_unambiguous_manifest_action() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
        let mut pad = crate::plugins::parse(&folder, &std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
        pad.name = "SoundPad".into();
        pad.actions[0].name = "Toggle Mute".into();
        pad.actions[1].name = "Record Clip".into();
        let mut light = pad.clone();
        light.uuid = "org.example.light".into();
        light.name = "LightBar".into();
        light.actions[0].uuid = "org.example.light.toggle".into();
        light.actions[0].name = "Toggle Mute".into();
        light.actions[1].uuid = "org.example.light.bright".into();
        light.actions[1].name = "Brightness".into();
        let mut knowledge = crate::local_llm::Knowledge::builtin();
        knowledge.plugins = vec![pad, light];
        // Résultat attendu du chemin règles, sans prétendre mesurer la compréhension du modèle.
        let cases = [
            ("lance Toggle Mute de SoundPad", "pass"),
            ("active Record Clip de SoundPad", "pass"),
            ("lance Brightness de LightBar", "pass"),
            ("active Record Clip", "pass"),
            ("active Toggle Mute", "question"),
            ("active mute de SoundPad", "question"),
            ("lance toggle-mute de soundpad", "pass"),
            ("utilise SoundPad", "question"),
            ("lance une action SoundPad", "question"),
            ("active Toggle Mute de SonInconnu", "question"),
            ("lance Toggle Mute de SoundPad sur mon micro", "question"),
            ("active Toggle Mute de SoundPad sur l'app", "question"),
            ("lance Toggle Mute de SoundPad, attends 2 s", "pass"),
            ("dessine une licorne", "error"),
        ];
        let mut counts = [0; 3];
        for (request, expected) in cases {
            let proposal = propose_with(request, &Config::default(), &knowledge);
            let actual = match &proposal.status {
                Status::Reconnue if proposal.insertable() => { counts[0] += 1; "pass" }
                Status::Ambigue { question, choix, .. } if !question.is_empty() && !proposal.insertable() => {
                    assert!(proposal.steps.is_empty(), "{request}: {proposal:?}");
                    let expected_choices: &[(&str, &str)] = match request {
                        "utilise SoundPad" => &[("SoundPad", "Toggle Mute"), ("SoundPad", "Record Clip")],
                        "active Toggle Mute" => &[("SoundPad", "Toggle Mute"), ("LightBar", "Toggle Mute")],
                        "active mute de SoundPad" => &[("SoundPad", "Toggle Mute")],
                        "active Toggle Mute de SonInconnu" => &[("SoundPad", "Toggle Mute"), ("LightBar", "Toggle Mute")],
                        "lance Toggle Mute de SoundPad sur mon micro" | "active Toggle Mute de SoundPad sur l'app" => {
                            assert!(choix.is_empty(), "{request}: {proposal:?}");
                            &[]
                        }
                        _ => &[],
                    };
                    if !expected_choices.is_empty() {
                        assert_eq!(choix.len(), expected_choices.len(), "{request}: {proposal:?}");
                        for (choice, (plugin_name, action_name)) in choix.iter().zip(expected_choices) {
                            assert!(choice.contains(plugin_name) && choice.contains(action_name), "{request}: {choice}");
                            let resolved = propose_with(choice, &Config::default(), &knowledge);
                            let p = knowledge.plugins.iter().find(|p| p.display_name() == *plugin_name).unwrap();
                            let a = p.actions.iter().find(|a| a.name == *action_name).unwrap();
                            assert!(resolved.insertable() && matches!(&resolved.steps[..], [Step::PluginAction { plugin, action, .. }] if plugin == &p.uuid && action == &a.uuid), "{request}: {choice}: {resolved:?}");
                        }
                    }
                    counts[1] += 1; "question"
                }
                _ => { counts[2] += 1; "error" }
            };
            assert_eq!(actual, expected, "{request}: {proposal:?}");
        }
        assert_eq!(counts, [6, 7, 1]);
    }
    #[test]
    fn plugin_inspector_fields_are_not_guessed() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
        let mut plugin = crate::plugins::parse(&folder, &std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
        plugin.name = "SoundPad".into();
        plugin.actions[0].name = "Toggle Mute".into();
        plugin.actions[0].inspector = Some("inspector.html".into());
        let mut knowledge = crate::local_llm::Knowledge::builtin();
        knowledge.plugins = vec![plugin];
        let p = propose_with("active Toggle Mute de SoundPad", &Config::default(), &knowledge);
        assert!(matches!(&p.status, Status::Ambigue { question, choix, .. } if question.contains("réglages") && choix.is_empty()), "{p:?}");
        assert!(p.steps.is_empty() && !p.insertable());
    }
    #[test]
    fn plugin_choices_follow_interface_language_and_partial_names_stay_ambiguous() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
        let mut pad = crate::plugins::parse(&folder, &std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
        pad.name = "SoundPad".into();
        pad.actions[0].name = "Toggle Mute".into();
        pad.actions[1].name = "Mute Channel".into();
        let mut knowledge = crate::local_llm::Knowledge::builtin();
        knowledge.plugins = vec![pad];
        crate::i18n::set_language("en");
        let p = propose_with("mute de SoundPad", &Config::default(), &knowledge);
        let Status::Ambigue { choix, .. } = &p.status else { panic!("{p:?}") };
        assert_eq!(choix.len(), 2);
        assert!(p.steps.is_empty());
        for (choice, action) in choix.iter().zip(&knowledge.plugins[0].actions) {
            assert!(choice.starts_with("Launch ") && choice.contains(" from SoundPad"));
            let resolved = propose_with(choice, &Config::default(), &knowledge);
            assert!(resolved.insertable() && matches!(&resolved.steps[..], [Step::PluginAction { action: id, .. }] if id == &action.uuid), "{choice}: {resolved:?}");
        }
        crate::i18n::set_language("fr");
    }
    fn recognized(text: &str) -> Step {
        let p = first(text);
        assert_eq!(p.status, Status::Reconnue, "{text}: {:?}", p.notes);
        p.steps[0].clone()
    }
    #[test]
    fn natural_french_and_english_window_requests() {
        for text in [
            "fermer toute les fenetres sauf celle sur laquelle je suis",
            "Ferme toutes les fenêtres sauf l'active",
            "ferme les autres fenêtres",
            "close all windows except the current one",
            "close all other windows",
        ] {
            assert!(matches!(recognized(text), Step::WindowOp { op, target, .. } if op == "close_others" && target == "foreground"), "{text}");
        }
        assert!(matches!(recognized("réduis toutes les fenêtres sauf celle-ci active"), Step::WindowOp { op, .. } if op == "minimize_others"));
        assert!(matches!(recognized("minimise toutes les fenêtres"), Step::Hotkey { keys } if keys == "WIN+D"));
        assert!(matches!(recognized("affiche le bureau"), Step::Hotkey { keys } if keys == "WIN+D"));
        assert!(matches!(recognized("agrandis la fenêtre"), Step::WindowOp { op, .. } if op == "maximize"));
        assert!(matches!(recognized("déplace la fenêtre sur l'écran 3"), Step::WindowOp { op, monitor: 3, .. } if op == "move_to_monitor"));
        assert!(matches!(recognized("ferme la fenêtre"), Step::CloseWindow));
        assert!(matches!(recognized("fais une capture d'écran"), Step::Hotkey { keys } if keys == "WIN+SHIFT+S"));
        assert!(matches!(recognized("ouvre le gestionnaire des tâches"), Step::Hotkey { keys } if keys == "CTRL+SHIFT+ESC"));
        let p = first("wait 2 seconds then open https://example.com and press play pause");
        assert_eq!(p.status, Status::Reconnue, "{:?}", p.notes);
        assert!(matches!(&p.steps[..], [Step::Delay { ms: 2000 }, Step::Url { url }, Step::Media { command }] if url == "https://example.com" && command == "play_pause"), "{:?}", p.steps);
        // Les validations existantes restent : le volume ambigu demande toujours une précision.
        assert!(matches!(first("baisse le volume de 10 %").status, Status::Ambigue { .. }));
    }
    #[test]
    fn plain_paste_half_second_and_enter() {
        let p = first("colle sans mise en forme");
        assert_eq!(p.status, Status::Reconnue);
        assert!(matches!(&p.steps[..], [Step::Clipboard { op, .. }, Step::Clipboard { .. }, Step::Hotkey { keys }] if op == "read_to_variable" && keys == "CTRL+V"), "{:?}", p.steps);
        let p = first("wait half a second then press enter");
        assert_eq!(p.status, Status::Reconnue, "{:?}", p.notes);
        assert!(matches!(&p.steps[..], [Step::Delay { ms: 500 }, Step::Hotkey { keys }] if keys == "ENTER"), "{:?}", p.steps);
        assert!(matches!(recognized("attends une demi-seconde"), Step::Delay { ms: 500 }));
        assert!(matches!(recognized("attends deux secondes"), Step::Delay { ms: 2000 }));
        assert!(matches!(recognized("wait one second"), Step::Delay { ms: 1000 }));
        assert!(matches!(recognized("colle"), Step::Hotkey { keys } if keys == "CTRL+V"));
    }
    #[test]
    fn example_sequence() {
        let p = first("Lance OBS, attends 2 s, passe sur Gaming");
        assert_eq!(p.status, Status::Reconnue);
        assert!(
            matches!(&p.steps[..], [Step::Launch { program, .. }, Step::Delay { ms: 2000 }, Step::ObsScene { name }] if program == "obs64.exe" && name == "Gaming")
        );
        // OBS est une application connue : programme rempli, plus rien à compléter.
        assert!(p.missing.is_empty() && p.insertable());
    }
    #[test]
    fn volume_ambigu() {
        let p = first("baisse le volume de 10 %");
        assert!(matches!(p.status, Status::Ambigue { .. }));
        assert!(!p.insertable());
    }
    #[test]
    fn volume_explicit() {
        assert!(matches!(
            recognized("baisse le volume général de 10 %"),
            Step::DeviceVolume { step: 10, .. }
        ));
        assert!(matches!(
            recognized("baisse le volume de l'application active de 10 %"),
            Step::AppVolume { step: 10, .. }
        ));
    }
    #[test]
    fn hotkeys() {
        assert!(
            matches!(recognized("Ctrl+Shift+M"), Step::Hotkey { keys } if keys == "CTRL+SHIFT+M")
        );
        assert!(matches!(recognized("alt f4"), Step::Hotkey { .. }));
        assert!(matches!(recognized("windows+d"), Step::Hotkey { keys } if keys == "WIN+D"));
    }
    #[test]
    fn delays() {
        assert!(matches!(
            recognized("attends 500 ms"),
            Step::Delay { ms: 500 }
        ));
        assert!(matches!(
            recognized("attends 1,5 seconde"),
            Step::Delay { ms: 1500 }
        ));
        assert!(matches!(
            recognized("attends 2 minutes"),
            Step::Delay { ms: 120000 }
        ));
    }
    #[test]
    fn launch_requires_path() {
        let p = first("ouvre l'application MonOutilMaison");
        assert!(matches!(p.steps[0], Step::Launch { .. }));
        assert_eq!(p.missing[0].field, "program");
    }
    #[test]
    fn obs() {
        assert!(
            matches!(recognized("passe sur Gaming"), Step::ObsScene { name } if name == "Gaming")
        );
        assert!(matches!(
            recognized("démarre l'enregistrement OBS"),
            Step::ObsRecord { .. }
        ));
        assert!(matches!(
            recognized("démarre le stream OBS"),
            Step::ObsStream { .. }
        ));
        assert!(matches!(
            recognized("coupe le micro OBS"),
            Step::ObsMute { .. }
        ));
    }
    #[test]
    fn media() {
        assert!(matches!(recognized("lecture pause"), Step::Media { .. }));
        assert!(
            matches!(recognized("piste suivante"), Step::Media { command } if command == "next")
        );
        assert!(
            matches!(recognized("piste précédente"), Step::Media { command } if command == "previous")
        );
    }
    #[test]
    fn text_url_notify() {
        assert!(matches!(recognized("tape \"Bonjour\""), Step::Text { text } if text == "Bonjour"));
        assert!(matches!(
            recognized("ouvre https://example.org"),
            Step::Url { .. }
        ));
        assert!(
            matches!(recognized("notification \"Salut\""), Step::Notify { message, .. } if message == "Salut")
        );
    }
    #[test]
    fn repeat_and_navigation() {
        let p = first("répète 3 fois");
        assert!(matches!(p.steps[0], Step::Repeat { count: 3, .. }));
        assert_eq!(p.missing[0].field, "steps");
        assert!(matches!(recognized("page suivante"), Step::PageNext));
        assert!(matches!(recognized("page précédente"), Step::PagePrevious));
        assert!(matches!(recognized("retour au dossier"), Step::Back));
    }
    #[test]
    fn brightness_clipboard_close() {
        assert!(matches!(
            recognized("baisse la luminosité de 20 %"),
            Step::D6Brightness { value: 20, .. }
        ));
        assert!(
            matches!(recognized("efface le presse-papiers"), Step::Clipboard { op, .. } if op == "clear")
        );
        assert!(matches!(recognized("ferme la fenêtre"), Step::CloseWindow));
    }
    #[test]
    fn unknown_and_invalid() {
        assert_eq!(first("dessine une licorne").status, Status::Inconnue);
        assert!(matches!(
            first("attends 99 heures").status,
            Status::Ambigue { .. }
        ));
        assert!(matches!(first("passe sur").status, Status::Reconnue));
    }
    #[test]
    fn diagnosis_and_explanation() {
        let steps = vec![
            Step::Delay { ms: 100 },
            Step::Delay { ms: 200 },
            Step::Delay { ms: 0 },
            Step::Disabled {
                step: Box::new(Step::CloseWindow),
            },
        ];
        let suggestions = diagnose(&steps);
        assert!(suggestions
            .iter()
            .any(|s| matches!(s.steps.first(), Some(Step::Delay { ms: 300 }))));
        assert!(suggestions.iter().any(|s| s.reason.contains("désactivé")));
        assert!(explain(&Step::Launch {
            program: String::new(),
            args: vec![],
            focus_if_running: false
        })
        .1
        .iter()
        .any(|m| m.field == "program"));
    }
    #[test]
    fn profile_validation() {
        assert!(
            matches!(recognized("change de profil Bureau"), Step::Profile { name } if name == "Bureau")
        );
        assert!(matches!(
            first("change de profil Inexistant").status,
            Status::Ambigue { .. }
        ));
    }
    #[test]
    fn invalid_pct_not_insertable() {
        let p = first("baisse le volume général de 0 %");
        assert!(!p.insertable());
        assert!(matches!(p.status, Status::Ambigue { .. }));
    }
    #[test]
    fn clauses_keep_decimal_and_quotes() {
        let p = first("attends 1,5 seconde, tape \"Bonjour, et merci\"");
        assert_eq!(p.steps.len(), 2);
        assert!(matches!(&p.steps[1], Step::Text { text } if text == "Bonjour, et merci"));
    }
    #[test]
    fn clipboard_copy() {
        assert!(
            matches!(recognized("copie le texte \"salut\" dans le presse-papiers"), Step::Clipboard { value, .. } if value == "salut")
        );
    }
    #[test]
    fn navigation_mixed_is_not_insertable() {
        let p = first("page suivante, attends 2 s");
        assert!(!p.insertable());
        assert!(matches!(p.status, Status::Ambigue { .. }));
    }
    #[test]
    fn templates_reuse_catalog() {
        assert!(!common_templates().is_empty());
    }
    #[test]
    fn no_silent_partial() {
        let p = first("attends 2 s, dessine une licorne");
        assert_eq!(p.status, Status::Inconnue);
        assert!(!p.insertable());
    }
}
