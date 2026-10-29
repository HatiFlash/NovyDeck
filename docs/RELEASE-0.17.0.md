# NovyDeck 0.17.0 — Deck colours, larger preview and subtle animations

## English

- **Deck colour:** choose the colour of the deck shown in the preview among eight shades (original black, graphite, white, silver, midnight blue, red, green, pink) or any custom colour with the colour picker, in **Settings › General**. Existing configurations keep the original black.
- **Larger preview:** the deck preview takes more room; the key inspector below it keeps its controls visible.
- **Collapsible "Recent activity":** the panel can be folded into a single line showing the event count and the latest event.
- **Subtle interface animations:** deck keys brighten slightly on hover and bounce briefly on click; Settings, the activity panel and deck colour changes fade in. A new **Interface animations** setting turns them off; they are also disabled when Windows animations are turned off. Nothing is redrawn while the interface is idle.
- **Fix found during testing:** clicking a key in the preview could freeze the interface in a pre-release build; fixed before publication.

Tested with the Windows CI suite and the off-screen UI harness on the test VM (565 tests passed). The animations were checked by tests, not watched on a real screen. Not yet tested on a connected deck.

## Français

- **Couleur du deck :** choisissez la couleur du deck affiché dans l'aperçu parmi huit teintes (noir d'origine, graphite, blanc, argent, bleu nuit, rouge, vert, rose) ou une couleur libre avec la pipette, dans **Réglages › Général**. Les configurations existantes gardent le noir d'origine.
- **Aperçu plus grand :** l'aperçu du deck occupe plus de place ; l'inspecteur de touche en dessous garde ses contrôles visibles.
- **« Activité récente » repliable :** le panneau peut se replier sur une seule ligne avec le nombre d'événements et le dernier événement.
- **Animations sobres de l'interface :** les touches du deck s'éclairent légèrement au survol et rebondissent brièvement au clic ; les Réglages, le panneau d'activité et les changements de couleur du deck apparaissent en fondu. Le nouveau réglage **Animations de l'interface** permet de les couper ; elles sont aussi coupées quand les animations de Windows sont désactivées. Rien n'est redessiné quand l'interface est au repos.
- **Correctif trouvé pendant les tests :** un clic sur une touche de l'aperçu pouvait figer l'interface dans une version de travail ; corrigé avant publication.

Validé par la suite de tests Windows de la CI et par le harnais d'interface hors écran sur la VM de test (565 tests réussis). Les animations ont été vérifiées par des tests, pas regardées sur un vrai écran. Pas encore essayé sur un deck connecté.
