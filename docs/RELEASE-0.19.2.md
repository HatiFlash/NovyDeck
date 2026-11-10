# NovyDeck 0.19.2 — Hardening: plugins, secrets, downloads

## English

- **Plugin messages (audit F-01)**: `setSettings` updates the configuration in memory and is written to disk at most once every 500 ms (atomic write with a `.bak` copy, no timestamped backup), and always on exit. Each plugin connection is limited to 50 messages per second; extra messages are dropped and logged at most once every 5 s.
- **Settings during editing (F-04)**: a `setSettings` received while a key edit is pending is queued (up to 32) and replayed once the edit is validated or cancelled, instead of being silently ignored.
- **Secrets (S-07)**: the OBS password is stored encrypted with Windows DPAPI (user scope); a plain-text password from an older configuration is read and rewritten encrypted at the next save, never in clear text again. MCP tool results replace plugin settings and any field whose name contains password, token or secret with `<masqué>`. Profile export has an "Export without secrets" option, enabled by default, which removes the OBS password and plugin settings; importing such a file says which settings must be filled in again.
- **Plugin import (S-13)**: size limits are enforced on the bytes actually extracted, not on the sizes declared in the archive; an archive that goes over the limit is refused and nothing is left in place.
- **Update installer (S-14)**: the SHA-256 of the downloaded installer is computed again right before it is launched; a modified file is refused.
- **Store badges (S-08)**: "Scanned" becomes "Antivirus: nothing detected", and the plugin page warns when a plugin ships an unsigned native executable that runs with your rights.
- **Build and process (S-09, S-10, P-01)**: GitHub actions pinned by commit SHA, Inno Setup 6.7.4 downloaded from its official release and checked by SHA-256, `cargo audit` in CI (known vulnerabilities fail the build), `SECURITY.md` and GitHub private vulnerability reporting.

Validated by targeted Windows tests (plugin host and blocks, plugin state and triggers, preferences, store, import, update, MCP, DPAPI, translations) and CI. Not changed in this release: the remaining audit items (signed updates and catalogue, Authenticode, profile import review, plugin isolation).

## Français

- **Messages des plugins (audit F-01)** : `setSettings` met à jour la configuration en mémoire et n'est écrit sur le disque qu'au plus une fois toutes les 500 ms (écriture atomique avec copie `.bak`, sans sauvegarde horodatée), et toujours à la fermeture. Chaque connexion de plugin est limitée à 50 messages par seconde ; les messages en trop sont ignorés et signalés au plus une fois toutes les 5 s.
- **Réglages pendant une modification (F-04)** : un `setSettings` reçu pendant qu'une modification de touche est en attente est mis en file (32 au maximum) puis rejoué après validation ou annulation, au lieu d'être ignoré en silence.
- **Secrets (S-07)** : le mot de passe OBS est stocké chiffré par Windows (DPAPI, portée utilisateur) ; un mot de passe en clair d'une ancienne configuration est lu puis réécrit chiffré à la sauvegarde suivante, plus jamais en clair. Les résultats des outils MCP remplacent les réglages des plugins et tout champ dont le nom contient password, token ou secret par `<masqué>`. L'export de profil a une option « Exporter sans secrets », cochée par défaut, qui retire le mot de passe OBS et les réglages des plugins ; l'import d'un tel fichier indique les réglages à refaire.
- **Import de plugin (S-13)** : les limites de taille portent sur les octets réellement extraits, et non sur les tailles annoncées par l'archive ; une archive qui dépasse la limite est refusée et rien n'est laissé en place.
- **Installeur de mise à jour (S-14)** : l'empreinte SHA-256 de l'installeur téléchargé est recalculée juste avant son lancement ; un fichier modifié est refusé.
- **Badges du magasin (S-08)** : « Analysé » devient « Antivirus : rien détecté », et la fiche avertit quand un plugin contient un exécutable natif non signé qui s'exécute avec tes droits.
- **Compilation et processus (S-09, S-10, P-01)** : actions GitHub épinglées par SHA, Inno Setup 6.7.4 téléchargé depuis sa release officielle et vérifié par SHA-256, `cargo audit` en CI (une faille connue fait échouer la compilation), `SECURITY.md` et signalement privé des failles sur GitHub.

Validé par des tests Windows ciblés (hôte et blocs des plugins, état et déclencheurs de plugin, préférences, magasin, import, mise à jour, MCP, DPAPI, traductions) et la CI. Pas encore traité dans cette version : les autres points de l'audit (mises à jour et catalogue signés, Authenticode, revue à l'import de profil, isolation des plugins).
