# NovyDeck 0.16.0 — Hardware ON/OFF key fixes and Block Workshop redesign

## English

- **Physical ON/OFF key fixes (FIFINE D6):**
  - **Persistent red dot removed:** the device-only state badge overlay ("1/2", "2/2") that rendered as an unwanted red circle on physical deck screens has been removed for all toggle and ON/OFF keys.
  - **Eliminated transient default-logo flash:** pressing an ON/OFF toggle key now updates only the targeted key face immediately without triggering a full-deck re-upload, avoiding the brief flash of the default NovyDeck logo.
- **Block Workshop complete redesign:**
  - **Fluid drag-and-drop:** text selection is disabled during dragging (`selectable_labels = false`), preventing unwanted blue highlights. Features a dedicated ghost card following the cursor, an esthetic 2×3 solid round-dot grab handle, and a distinct dashed "Drop here" placeholder that dynamically parts cards.
  - **Compact, categorized cards:** cards now display a colored category icon pill, a category-tinted left border, semibold titles, and a clean single-line summary.
  - **Visual container branches:** Toggle ("STATE 1 · ON", "STATE 2 · OFF"), If/Else ("THEN", "ELSE"), and Repeat branches feature colored chips, a vertical connector line linking children to their container, and folding chevrons with block counters.
  - **Block library & plugin integration:** instant accent-insensitive search bar with a clear button (×), pinned **Favorites** section stored in user preferences, and grouped plugins with manifest icons, direct drag into sequences, and an in-card "Settings" button.

## Français

- **Corrections matérielles des touches ON/OFF (FIFINE D6) :**
  - **Suppression du rond rouge persistant :** l'overlay de badge d'état ("1/2", "2/2") incrusté uniquement côté périphérique, qui apparaissait sous la forme d'un point rouge permanent sur l'écran du stream deck, a été retiré pour toutes les touches à bascule et ON/OFF.
  - **Suppression du flash transitoire du logo par défaut :** l'appui physique sur une touche ON/OFF met désormais à jour atomiquement la seule touche concernée sans forcer de réémission globale de tout le deck, éliminant le bref clignotement bleu du logo NovyDeck.
- **Refonte complète de l'Atelier de blocs :**
  - **Glisser-déposer fluide et précis :** la sélection de texte parasite en bleu lors du maintien est neutralisée. L'Atelier intègre une carte fantôme qui suit le curseur, une poignée de saisie soignée en grille de points ronds pleins (2×3), et un repère d'insertion « Déposer ici » en pointillés qui écarte clairement les cartes.
  - **Cartes compactes et colorées par catégorie :** pastille avec icône de catégorie, liseré gauche teinté, titres en demi-gras et résumé d'étape clair sur une seule ligne.
  - **Branches de conteneurs plus visuelles :** branches Basculer (« ÉTAT 1 · ON », « ÉTAT 2 · OFF »), Si/Sinon (« ALORS », « SINON ») et Répéter avec des étiquettes colorées, ligne de liaison verticale reliant les enfants au conteneur, et chevrons de repli avec compteur de blocs.
  - **Bibliothèque de blocs et plugins :** champ de recherche instantané insensible aux accents avec bouton d'effacement (×), section **Favoris** épinglables par étoile et persistés dans les préférences, et plugins regroupés avec leurs icônes, glisser direct vers la séquence et bouton direct « Réglages » sur la carte.

## Qualification / Validation

These changes were validated with 149 automated egui tests in the Windows off-screen test harness (including mouse drag-and-drop sequences, hardware toggle packet verification, and UI captures). They do not constitute a live test on a physical FIFINE D6; hardware behavior should be confirmed on the actual device.

Ces modifications ont été validées par 149 tests automatisés dans le harnais Windows hors écran (séquences de glisser-déposer egui, vérification des trames envoyées au deck, et captures d'interface). Elles ne constituent pas un essai sur un FIFINE D6 physique branché : le comportement matériel final devra être confirmé sur le périphérique réel.
