# NovyDeck 0.18.1 — Two-state keys work when the plugin state is unknown

## English

- **Two-state display with a plugin state source** (e.g. RivaTuner OSD): when the plugin cannot report its state, the key no longer stays stuck on the base icon with a red "Unknown state" badge. It now shows your configured **Off** face; the "Unknown state" hint only appears as text in the inspector.
- When the plugin reports nothing, each press toggles the key between Off and On locally; your choice survives repeated "unknown" reports. As soon as the plugin reports its real state, that state wins.
- Changing the icon of the On / Off face immediately updates the preview and the deck.
- An "unknown" report only affects the key it targets; other keys of the same plugin keep their state.

Validated by the Windows CI suite; installer and upgrade from 0.18.0 checked on the test VM. Not yet tested on a connected deck with RivaTuner.

## Français

- **Affichage à deux états avec un état de plugin** (ex. RivaTuner OSD) : quand le plugin ne peut pas indiquer son état, la touche ne reste plus bloquée sur l'icône de base avec la pastille rouge « État inconnu ». Elle affiche ta face **Désactivé** ; l'indication « État inconnu » n'apparaît plus qu'en texte dans l'inspecteur.
- Quand le plugin ne dit rien, chaque appui fait basculer la touche entre Désactivé et Activé ; ton choix reste affiché même si le plugin répète « inconnu ». Dès que le plugin indique son vrai état, c'est lui qui décide.
- Changer l'icône de la face Activé / Désactivé met à jour l'aperçu et le deck immédiatement.
- Un état « inconnu » ne touche que la touche concernée ; les autres touches du même plugin gardent leur état.

Validé par la suite de tests Windows de la CI ; installateur et mise à jour depuis la 0.18.0 vérifiés sur la VM de test. Pas encore essayé sur un deck connecté avec RivaTuner.
