# NovyDeck 0.19.4 — Automatic state and ON/OFF blocks

## English

- **Plugin actions in blocks**: actions on the active page receive a stable virtual context and `willAppear` when loaded or synchronized, including nested actions. Leaving the page or removing the block sends `willDisappear`; changing settings does not restart the action.
- **Sensor-controlled ON/OFF**: Home Assistant Automatic state can trigger a key with an ON/OFF block. An `active` edge runs action 1 (ON), an `inactive` edge runs action 2 (OFF), and repeated identical edges run the same action again. The sensor action is not manually pressed by its own edge. Legacy triggers without an edge and physical key presses retain their previous behavior. A configured ON/OFF key face follows the applied sensor state.
- **Workshop ON/OFF block**: “Toggle between two actions” is available in the block library with two empty branches. The key editor also supports converting a direct plugin action into a two-state key.
- **Readable ON/OFF cards**: plugin actions show their manifest title and, when available, their mode instead of a generic plugin-action label.

Validated by targeted Windows builds and tests. Not tested on a physical deck or with a real Home Assistant instance.

## Français

- **Actions de plugin en blocs** : les actions de la page active reçoivent un contexte virtuel stable et `willAppear` au chargement ou à la synchronisation, y compris dans les blocs imbriqués. Quitter la page ou supprimer le bloc envoie `willDisappear` ; modifier les réglages ne redémarre pas l'action.
- **ON/OFF piloté par le capteur** : Automatic state de Home Assistant peut déclencher une touche contenant un bloc ON/OFF. Un événement `active` lance l'action 1 (ON), `inactive` l'action 2 (OFF) ; répéter le même événement relance la même action. L'action capteur ne reçoit pas d'appui manuel à cause de son propre événement. Les déclencheurs sans `edge` et les appuis physiques conservent leur comportement. L'image ON/OFF configurée suit l'état appliqué.
- **Bloc ON/OFF de l'Atelier** : « Basculer entre deux actions » figure dans la bibliothèque avec deux branches vides. L'éditeur de touche permet aussi de convertir une action directe de plugin en touche à deux états.
- **Cartes ON/OFF lisibles** : les actions de plugin affichent le titre du manifeste et, si disponible, leur mode au lieu d'un libellé générique.

Validé par compilation et tests Windows ciblés. Non testé sur un deck physique ni avec une instance Home Assistant réelle.
