//! Source picker for the existing two-face editor. Include as `mod ui_plugin_state;` inside ui.rs.
use eframe::egui;
use crate::{i18n::tr, model::{Key, PluginStateTarget}, plugin_state_display::{self, Observation}, plugins::Plugin};

fn action_name(catalog: Option<&[Plugin]>, plugin: &str, action: &str) -> String {
    if let Some((owner, entry)) = catalog.and_then(|items| items.iter().find(|p| p.uuid == plugin)
        .and_then(|p| p.actions.iter().find(|a| a.uuid == action).map(|a| (p, a)))) {
        return format!("{} · {}", owner.display_name(), if entry.name.is_empty() { action } else { &entry.name });
    }
    format!("{plugin} · {action}")
}

/// Call before borrowing `key.state_display`. Only changes the opt-in target;
/// face images remain intact when switching or disabling the source.
pub(super) fn source_ui(ui: &mut egui::Ui, key: &mut Key, observation: Observation, catalog: Option<&[Plugin]>) -> bool {
    let mut choices = Vec::new();
    if let Some(binding) = &key.plugin {
        if plugin_state_display::target_valid(key, &PluginStateTarget::Direct) {
            choices.push((PluginStateTarget::Direct, format!("{} · {}", tr("Touche"), action_name(catalog, &binding.plugin, &binding.action))));
        }
    }
    for (path, plugin, action) in plugin_state_display::block_sources(key) {
        let index = path.iter().map(|i| (i + 1).to_string()).collect::<Vec<_>>().join(".");
        choices.push((PluginStateTarget::Block { path }, format!("{} {index} · {}", tr("Bloc"), action_name(catalog, &plugin, &action))));
    }
    let selected = key.plugin_state.as_ref().and_then(|target| choices.iter().find(|(candidate, _)| candidate == target).map(|(_, name)| name.as_str()))
        .unwrap_or_else(|| if key.plugin_state.is_some() { tr("Source introuvable") } else { tr("Aucune source plugin") });
    let mut changed = false;
    ui.label(tr("Source de l'état plugin"));
    let max_width = ui.available_width().max(100.0);
    egui::ComboBox::from_id_salt("plugin_state_source").width(max_width.min(380.0))
        .selected_text(egui::RichText::new(selected).size(11.0)).truncate().show_ui(ui, |ui| {
        ui.set_max_width(max_width.min(380.0));
        changed |= ui.selectable_value(&mut key.plugin_state, None, tr("Aucune source plugin")).changed();
        for (target, name) in choices { changed |= ui.selectable_value(&mut key.plugin_state, Some(target), name).changed(); }
    });
    if key.plugin_state.is_some() {
        let status = match observation {
            Observation::Confirmed(_) => tr("État confirmé"),
            Observation::Local(_) => tr("État local"),
            Observation::Unknown => tr("État inconnu"),
            Observation::SourceReported(_) => tr("Signalé par plugin (non vérifié)"),
        };
        ui.label(status);
    }
    changed
}
