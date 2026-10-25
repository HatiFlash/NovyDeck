# NovyDeck 0.11.0 — plugin management, block inspectors and state faces

## English

- **Plugin Store.** See installed versus available versions; confirm a verified, transactional update with backup/rollback and downgrade refusal. Confirm uninstall after reviewing references from keys and blocks. Profiles, images and configuration remain in place; actions referencing a removed plugin become unavailable until it is reinstalled or reassigned.
- **Workshop.** Installed plugins are compact and individually collapsible; the action picker is grouped and scrollable in compact and HiDPI windows. Clicking a plugin block or **Plugin settings** opens that action's real Property Inspector, if supplied by the plugin. New blocks have independently saved settings.
- **State faces.** Select a direct plugin action or macro block as the state source; configure separate ON/OFF images and labels. The app distinguishes a plugin-confirmed state from a local one and falls back to the base face on missing/invalid reports. Existing custom key images have priority. Google Home and Wallpaper Engine 1.0.5 supply state reports; existing 1.0.4 installations are not replaced automatically.
- **Testing.** **Test for real** handles a key with one direct plugin action, behind the existing confirmation. **Simulate** sends no plugin command.

Your profiles, key images, macros, plugin settings and connections are kept. Windows builds, automated UI/runtime tests and an isolated native Property Inspector fixture were used; the physical D6, a live Google account and a live Wallpaper Engine installation were **not** tested. Review the changes before updating plugins from the Store.

## Français

- **Magasin de plugins.** Compare les versions installée et disponible ; une mise à jour vérifiée et transactionnelle est confirmée, sauvegardée et annulable en cas d'échec. Une version inférieure est refusée. La désinstallation indique les usages sur les touches et les blocs et demande confirmation. Profils, images et réglages restent en place ; les actions liées au plugin deviennent indisponibles jusqu'à réinstallation ou réaffectation.
- **Atelier.** La bibliothèque de plugins installés est compacte et repliable plugin par plugin ; le sélecteur d'actions, classé et défilable, fonctionne en petite fenêtre et en HiDPI. Un clic sur un bloc plugin ou **Réglages du plugin** ouvre le véritable inspecteur de propriétés, si le plugin en fournit un. Chaque bloc neuf possède ses propres réglages persistants.
- **Faces d'état.** Choisissez une action plugin directe ou un bloc de macro comme source, puis des images et libellés ON/OFF séparés. L'application distingue un état confirmé par le plugin d'un état local et revient à la face de base sans rapport valide. Les images personnalisées restent prioritaires. Google Home et Wallpaper Engine 1.0.5 émettent les états ; les installations 1.0.4 ne sont pas remplacées automatiquement.
- **Tests.** **Tester pour de vrai** gère une touche composée d'une seule action plugin, avec la confirmation existante. **Simuler** n'envoie aucune commande plugin.

Vos profils, images de touches, macros, réglages de plugins et connexions sont conservés. La compilation Windows, les tests automatisés d'interface/runtime et une fixture native d'inspecteur isolée ont servi à la validation ; le D6 physique, un compte Google réel et une installation Wallpaper Engine réelle n'ont **pas** été testés. Examinez les changements avant de mettre à jour les plugins depuis le Magasin.
