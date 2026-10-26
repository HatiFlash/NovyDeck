//! Corpus déterministe du chemin règles, sans réseau ni modèle. Les questions ci-dessous
//! sont délibérées : elles ne doivent jamais inventer une action, un plugin ou un réglage.
use super::{propose_with, Status};
use crate::local_llm::Knowledge;
use crate::model::{Config, Step};

type Choice = (&'static str, &'static str);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expected<'a> {
    Pass(Choice, bool), // bool : suivi d'une attente de 2 s
    Question(&'a [Choice]),
    Unknown,
}
use Expected::{Pass, Question, Unknown};

#[test]
fn plugin_request_corpus() {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/counter.sdPlugin");
    let mut pad = crate::plugins::parse(&folder, &std::fs::read(folder.join("manifest.json")).unwrap()).unwrap();
    pad.name = "SoundPad".into();
    pad.actions[0].name = "Toggle Mute".into();
    pad.actions[1].name = "Record Clip".into();
    let mut light = pad.clone();
    light.uuid = "org.example.lightbar".into();
    light.name = "LightBar".into();
    light.actions[0].uuid = "org.example.lightbar.mute".into();
    light.actions[0].name = "Toggle Mute".into();
    light.actions[1].uuid = "org.example.lightbar.brightness".into();
    light.actions[1].name = "Brightness".into();
    let mut obs = pad.clone();
    obs.uuid = "org.example.obsfixture".into();
    obs.name = "OBS-like".into();
    obs.actions[0].uuid = "org.example.obsfixture.start".into();
    obs.actions[0].name = "Start Recording".into();
    obs.actions[1].uuid = "org.example.obsfixture.stop".into();
    obs.actions[1].name = "Stop Recording".into();
    let mut scene = obs.actions[0].clone();
    scene.uuid = "org.example.obsfixture.scene".into();
    scene.name = "Switch Scene".into();
    obs.actions.push(scene);
    let mut knowledge = Knowledge::builtin();
    knowledge.plugins = vec![pad, light, obs];
    let tm = ("SoundPad", "Toggle Mute");
    let lm = ("LightBar", "Toggle Mute");
    let rc = ("SoundPad", "Record Clip");
    let br = ("LightBar", "Brightness");
    let start = ("OBS-like", "Start Recording");
    let stop = ("OBS-like", "Stop Recording");
    let scene = ("OBS-like", "Switch Scene");
    let cases: &[(&str, Expected)] = &[
        ("lance Toggle Mute de SoundPad", Pass(tm, false)),
        ("active Toggle Mute de LightBar", Pass(lm, false)),
        ("déclenche Toggle Mute avec SoundPad", Pass(tm, false)),
        ("appuie sur Toggle Mute de SoundPad", Pass(tm, false)),
        ("utilise Toggle Mute sur SoundPad", Pass(tm, false)),
        ("run Toggle Mute from SoundPad", Pass(tm, false)),
        ("trigger Toggle Mute with LightBar", Pass(lm, false)),
        ("toggle Toggle Mute from SoundPad", Pass(tm, false)),
        ("start Toggle Mute with SoundPad", Pass(tm, false)),
        ("LANCE TOGGLE-MUTE DE SOUNDPAD", Pass(tm, false)),
        ("SoundPad : Toggle Mute", Pass(tm, false)),
        ("Mute Toggle de SoundPad", Pass(tm, false)),
        ("SoundPad Record Clip", Pass(rc, false)),
        ("Record Clip", Pass(rc, false)),
        ("record-clip avec soundpad", Pass(rc, false)),
        ("Brightness de LightBar", Pass(br, false)),
        ("LIGHTBAR BRIGHTNESS", Pass(br, false)),
        ("lance Start Recording de OBS-like", Pass(start, false)),
        ("Start Recording from OBS-like", Pass(start, false)),
        ("run Start Recording with OBS-like", Pass(start, false)),
        ("arrête Stop Recording de OBS-like", Pass(stop, false)),
        ("Switch Scene sur OBS-like", Pass(scene, false)),
        ("scene switch from OBS-like", Pass(scene, false)),
        ("lance Toggle Mute de SoundPad, puis attends 2 s", Pass(tm, true)),
        ("trigger Start Recording from OBS-like then wait 2 seconds", Pass(start, true)),
        ("active Toggle Mute", Question(&[tm, lm])),
        // Le nom partiel ne justifie pas l'insertion : choix complets uniquement.
        ("mute de SoundPad", Question(&[tm])),
        ("Toggle de LightBar", Question(&[lm])),
        ("utilise SoundPad", Question(&[tm, rc])),
        ("run LightBar", Question(&[lm, br])),
        ("OBS-like", Question(&[start, stop, scene])),
        ("fais clignoter LightBar", Question(&[lm, br])),
        // Un plugin inexistant ne peut pas être substitué par l'homonyme installé.
        ("Toggle Mute de GhostPad", Question(&[tm, lm])),
        ("Toggle Mute from GhostPad", Question(&[tm, lm])),
        ("Record Clip de GhostPad", Question(&[rc])),
        // Cible/paramètres absents du manifeste : pas de bouton à compléter implicitement.
        ("Toggle Mute de SoundPad sur mon micro", Question(&[])),
        ("Toggle Mute from SoundPad on my mic", Question(&[])),
        ("Record Clip de SoundPad sur ma piste", Question(&[])),
        ("SoundPad lance une action inconnue", Question(&[tm, rc])),
        ("trigger Fade Out with OBS-like", Question(&[start, stop, scene])),
        ("dessine une licorne", Unknown),
        ("draw a unicorn", Unknown),
        ("launch plugin GhostPad", Unknown),
        ("start recording tomorrow", Unknown),
        ("démarre Start Recording de OBS-like demain", Unknown),
        ("Switch Scene from OBS-like next week", Unknown),
    ];
    let mut counts = [0usize; 3];
    let mut gaps = Vec::new();
    for &(request, expected) in cases {
        let p = propose_with(request, &Config::default(), &knowledge);
        // Aucun résultat, même inattendu, ne peut ajouter une action absente de la
        // requête ou un plugin non installé. L'attente explicitement demandée est seule admise.
        let mentioned = super::parser::normalize(request);
        let inserted: Vec<_> = p.steps.iter().filter_map(|s| match s {
            Step::PluginAction { plugin, action, .. } => Some((plugin, action)),
            _ => None,
        }).collect();
        for (id, action_id) in &inserted {
            let valid = knowledge.plugins.iter().any(|plugin| plugin.uuid == **id
                && plugin.actions.iter().any(|action| action.uuid == **action_id
                    && super::parser::normalize(&action.name).split_whitespace()
                        .all(|word| super::contains_phrase(&mentioned, word))
                    && (super::contains_phrase(&mentioned, &super::parser::normalize(plugin.display_name()))
                        || knowledge.plugins.iter().filter(|p| p.actions.iter().any(|a| a.name == action.name)).count() == 1)));
            if !valid { gaps.push(format!("FAUX POSITIF {request:?}: {p:?}")); }
        }
        let actual = match &p.status {
            Status::Reconnue if p.insertable() => {
                counts[0] += 1;
                if let [Step::PluginAction { plugin, action, .. }, rest @ ..] = &p.steps[..] {
                    let found = knowledge.plugins.iter().find(|item| item.uuid == *plugin).and_then(|item|
                        item.actions.iter().find(|item_action| item_action.uuid == *action).map(|a| (item.display_name(), a.name.as_str())));
                    let delay = if rest.is_empty() { Some(false) }
                        else if matches!(rest, [Step::Delay { ms: 2000 }]) { Some(true) }
                        else { None };
                    match (found, delay, expected) {
                        (Some((name, action_name)), Some(actual_delay), Pass((wanted_name, wanted_action), wanted_delay))
                            if name == wanted_name && action_name == wanted_action && actual_delay == wanted_delay => expected,
                        _ => Unknown,
                    }
                } else { Unknown }
            }
            Status::Ambigue { question, choix, choix_autonomes: true } if !question.is_empty() && p.steps.is_empty() && !p.insertable() => {
                counts[1] += 1;
                let mut found = Vec::new();
                for choice in choix {
                    let resolved = propose_with(choice, &Config::default(), &knowledge);
                    if let [Step::PluginAction { plugin, action, .. }] = &resolved.steps[..] {
                        if resolved.insertable() {
                            if let Some(pair) = knowledge.plugins.iter().find(|item| item.uuid == *plugin).and_then(|item|
                                item.actions.iter().find(|a| a.uuid == *action).map(|a| (item.display_name(), a.name.as_str()))) {
                                found.push(pair);
                            }
                        }
                    }
                }
                if found.len() != choix.len() { gaps.push(format!("CHOIX NON AUTONOME {request:?}: {p:?}")); }
                // On compare les choix proposés ET leur ordre, pour déceler les omissions.
                let expected_choices = match expected { Question(items) => items, _ => &[] };
                if found != expected_choices { gaps.push(format!("CHOIX {request:?}: attendu {expected_choices:?}, obtenu {found:?}")); }
                Question(&[])
            }
            _ => { counts[2] += 1; Unknown }
        };
        let same = match (actual, expected) {
            (Question(_), Question(_)) => true,
            (Pass(a, x), Pass(b, y)) => a == b && x == y,
            (Unknown, Unknown) => p.steps.is_empty() && !p.insertable(),
            _ => false,
        };
        if !same { gaps.push(format!("ÉCART {request:?}: attendu {expected:?}, obtenu {p:?}")); }
    }
    eprintln!("CORPUS plugins: pass={} question={} inconnue={} écarts={}", counts[0], counts[1], counts[2], gaps.len());
    for gap in &gaps { eprintln!("  {gap}"); }
    assert!(gaps.is_empty(), "{} écarts dans le corpus plugins: {}", gaps.len(), gaps.join(" | "));

    // Question et choix restent anglais, y compris les refus du modèle.
    crate::i18n::set_language("en");
    for (request, prefix, choices) in [
        ("Toggle Mute from GhostPad", "Which installed plugin", 2),
        ("Toggle Mute", "Which action from which plugin", 2),
        ("SoundPad", "Which action from", 2),
        ("Toggle Mute from SoundPad on my mic", "Which target or device", 0),
    ] {
        let p = propose_with(request, &Config::default(), &knowledge);
        assert!(matches!(&p.status, Status::Ambigue { question, choix, .. }
            if question.starts_with(prefix) && choix.len() == choices
                && choix.iter().all(|item| item.starts_with("Launch "))), "{request}: {p:?}");
    }
    let unknown = super::from_model_guarded(vec![], vec![("org.example.ghost".into(), "missing".into())], &Config::default(), &knowledge);
    assert!(matches!(&unknown.status, Status::Ambigue { question, choix, .. }
        if question.starts_with("The proposed plugin is not installed") && choix.is_empty()), "{unknown:?}");
    let rejected = super::from_model_guarded(vec![], vec![(knowledge.plugins[0].uuid.clone(), knowledge.plugins[0].actions[0].uuid.clone())], &Config::default(), &knowledge);
    assert!(matches!(&rejected.status, Status::Ambigue { question, choix, .. }
        if question.starts_with("Which action from") && question.contains("settings")
            && choix.iter().all(|item| item.starts_with("Launch "))), "{rejected:?}");
    crate::i18n::set_language("fr");
}
