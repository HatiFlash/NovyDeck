# NovyDeck 0.18.0 — Visible, fluid animations and styled drag & drop

## English

- **Sliding tabs:** switching between Action / Appearance / Advanced, Short / Long press, the library tabs and the Settings tabs slides the content in from the side of the clicked tab; the active-tab indicator slides along.
- **Drag & drop with a card:** a grabbed action becomes a floating card (icon + name, soft shadow, slight tilt) that follows the pointer; the hovered key lights up and grows slightly. Dropping it makes the card dive into the key, followed by a "pop" and an accent ring. Dropping outside sends the card back.
- **Block Workshop:** the held block tilts, neighbouring blocks slide aside, moved blocks glide to their new place, and a drop ends with a pop.
- **Existing animations made visible:** Settings open with a fade and slight zoom, "Recent activity" really expands and collapses, deck pages slide in the navigation direction, and the key click bounce is noticeable.
- Everything can still be turned off with **Interface animations** and follows the Windows animation setting. Nothing is redrawn while idle.

Tested with the Windows CI suite and the off-screen UI harness on the test VM (571 tests passed); the GIFs come from real window frames rendered by the harness. Not yet tested on a connected deck.

## Français

- **Onglets qui glissent :** passer entre Action / Apparence / Avancé, Appui court / Appui long, les onglets de la bibliothèque et ceux des Réglages fait glisser le contenu depuis le côté de l'onglet cliqué ; l'indicateur de l'onglet actif glisse avec.
- **Glisser-déposer avec une carte :** une action attrapée devient une carte flottante (icône + nom, ombre douce, légère inclinaison) qui suit la souris ; la touche survolée s'illumine et grossit légèrement. En la lâchant, la carte plonge dans la touche, suivie d'un « pop » et d'un anneau lumineux. Lâchée à côté, la carte revient à sa place.
- **Atelier de blocs :** le bloc tenu s'incline, les blocs voisins s'écartent, les blocs déplacés glissent vers leur nouvelle place et le dépôt se termine par un pop.
- **Animations existantes rendues visibles :** les Réglages s'ouvrent en fondu avec un léger zoom, « Activité récente » s'ouvre et se ferme vraiment, les pages du deck glissent dans le sens de navigation et le rebond au clic se voit.
- Tout reste désactivable avec **Animations de l'interface** et suit le réglage d'animations de Windows. Rien n'est redessiné au repos.

Validé par la suite de tests Windows de la CI et par le harnais d'interface hors écran sur la VM de test (571 tests réussis) ; les GIF proviennent de vraies images de la fenêtre rendues par le harnais. Pas encore essayé sur un deck connecté.
