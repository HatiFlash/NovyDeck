# NovyDeck 0.18.1 · film Remotion

1600 images, 53⅓ s, 1920 × 1080, 30 i/s. Onze scènes, dix transitions de 12 images et une bande-son originale stéréo 48 kHz synchronisée (`python make-sound.py`, Python stdlib). La scène 3D Three.js montre les 15 touches LCD du FIFINE D6. Aucun matériel ni plugin n'est déclenché au rendu. Les scènes d'interface emploient uniquement de vraies captures du projet : aucun écran recomposé ou simulé.

## Découpage (début en images)

| Image | Scène | Message |
|---:|---|---|
| 0 | Ouverture | Ton deck. Ton univers. / NovyDeck |
| 138 | Deck 3D | Chaque touche, une idée. |
| 276 | Interface | Tout commence ici. |
| 414 | Personnalisation | Couleurs. Icônes. Ta signature. |
| 562 | Glisser-déposer | Attrape. Glisse. C’est posé. |
| 710 | Atelier de blocs | Des macros qui bougent. |
| 853 | Deux états | Un appui. Deux visages. |
| 996 | Plugins et magasin | Plus loin, ensemble. |
| 1139 | Assistant IA | Décris. Relis. Crée. |
| 1272 | Fluidité 0.18 | Tout coule de source. |
| 1405 | Signature | NovyDeck 0.18.1 / github.com/HatiFlash/NovyDeck |

Les captures `public/shots/v*.png` proviennent de `docs/images/` : `v017-deck-white-fr`, `v017-deck-color-free-fr`, `v017-activity-collapsed-fr`, `v016-workshop-cards-fr`, `v016-workshop-library-fr`, `v015-assistant-fr`, `v015-appearance-fr`, `v0161-onoff-cards-fr`, `v0161-plugins-tab-fr`, `v014-update-fr`. Les photogrammes `ui018r-drag-00…13`, `ui018r-atelier-00…13`, `ui018r-tabs-00…12` sont des images réelles 0.18 du harnais Windows (environ 33 ms entre images), jouées à une image Remotion par photogramme, puis figées pour laisser le temps de lire. Les cadres, textes, perspective, zooms et transitions sont du motion design autour de ces captures. Les écrans LCD du deck 3D utilisent les textures déjà présentes dans `public/keys/` ; valider leur pertinence visuelle avant diffusion.

## Sur Windows (PowerShell, dans `scripts/media/remotion`)

Avec Node.js et les dépendances du `package-lock.json` (Remotion 4.0.537 / Three.js 0.175.0) :

    npm.cmd ci
    npm.cmd run assets
    npm.cmd run typecheck
    npm.cmd run still -- 90 out/s0090.png
    npm.cmd run still -- 600 out/s0600.png
    npm.cmd run still -- 1510 out/s1510.png
    npm.cmd run studio

Plage valide pour `still` : 0…1599. L'export final, uniquement après validation :

    npm.cmd run render -- out/novydeck-0.18.1.mp4

Le rendu H.264 utilise `--gl=angle` ; en cas de problème GPU, diagnostiquer le pilote et essayer `--gl=swangle`. Ne pas annoncer un MP4 comme validé sans rendu ni visionnage. Animation déterministe par image (`useCurrentFrame`, `interpolate`, `spring`), sans chronomètre ni animation CSS.

## Origine et licences

Inter (SIL OFL) : `public/fonts/Inter-LICENSE.txt`. Synthèse sonore sous MIT : `licenses/sfx-MIT-LICENSE.txt`. Logo, captures et textures LCD sont des ressources NovyDeck. Les captures sont celles des versions indiquées par leurs noms : elles illustrent des fonctions toujours documentées en 0.18.1, sans prétendre montrer une nouvelle capture de l'application 0.18.1. Les séquences animées 0.18 viennent du harnais hors écran ; elles ne prouvent pas une interaction physique sur un D6. La version 0.18.1 et le repli de l'état inconnu sont décrits dans `docs/RELEASE-0.18.1.md`.
