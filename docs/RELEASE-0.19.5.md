# NovyDeck 0.19.5 — Reliable autosave, persistent logs and no key flashing

## English

- **Autosave no longer stalls**: identical settings re-sent by a plugin inspector are ignored instead of restarting the 800 ms autosave delay. The Workshop could previously stay on “Autosaving…”, and plugin triggers and physical key presses were refused as “editing in progress”. If an invalid configuration prevents autosave, an error is now shown.
- **No flashing on audio output switch**: changing the default audio device no longer re-uploads the whole profile to the deck; only keys whose audio face changes are redrawn, so other plugin keys keep their image.
- **Persistent journal**: the journal is also written to `logs/novydeck.log` next to `config.json`, with rotation and masked secrets. Plugin error output is kept in bounded per-plugin logs instead of being discarded. Unsaved-edit sources and skipped autosaves are logged. The Journal panel has an “Open log folder” button.

Validated by Windows builds and targeted tests (plugins, toggle, workshop, engine, i18n, autosave, inspector, log, audio). Not yet tested on a physical deck.

## Français

- **Sauvegarde automatique fiable** : les réglages identiques renvoyés par l'inspecteur d'un plugin sont ignorés au lieu de relancer le délai de 800 ms. L'Atelier pouvait rester sur « Sauvegarde automatique… » et refuser les déclencheurs et appuis physiques pour « édition en cours ». Une configuration invalide qui empêche la sauvegarde affiche désormais une erreur.
- **Plus de clignotement au changement de sortie audio** : changer de périphérique audio ne renvoie plus tout le profil au deck ; seules les touches dont l'image audio change sont redessinées.
- **Journal persistant** : le Journal est aussi écrit dans `logs/novydeck.log` à côté de `config.json`, avec rotation et secrets masqués. La sortie d'erreur des plugins est conservée dans des journaux bornés par plugin. La source des modifications non enregistrées et les sauvegardes sautées sont journalisées. Le panneau Journal propose « Ouvrir le dossier des journaux ».

Validé par compilation et tests Windows ciblés. Pas encore testé sur un deck physique.
