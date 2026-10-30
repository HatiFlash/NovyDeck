# NovyDeck 0.18.1 · film animé Remotion v2

Deux compositions de 55 s à 30 i/s : `NovyDeckMotion` (1920 × 1080) et `NovyDeckMotionVertical` (1080 × 1920). Interface fidèle au thème bleu nuit de NovyDeck, redessinée en React, sans capture d'écran dans ces compositions. Portrait : scènes recomposées en grands panneaux autonomes, deck à quinze touches visible, marges haut/bas protégées. Les anciennes compositions `MotionPreview*` et `NovyDeckDemo` restent historiques.

| Temps | Scène | Accroche |
|---|---|---|
| 0–3 s | Logo en tuiles, deck 3D et titre claqué | Ton deck. Ton univers. |
| 3–7 s | Deck FIFINE D6, quinze touches LCD | Chaque touche compte. |
| 7–14 s | Inspecteur, couleur, icône, titre | À ton image. |
| 14–21 s | Bibliothèque, carte flottante et dépôt sur touche | Glisse et crée. |
| 21–29 s | Atelier, macros et bibliothèque de blocs | Compose tes macros. |
| 29–35 s | Deux cartes d'état et bascule locale | Deux états. |
| 35–42 s | Magasin, import, actions de plugins | Des plugins partout. |
| 42–49 s | Assistant, conversation et proposition à relire | Décris, puis choisis. |
| 49–55 s | Logo final, version, dépôt GitHub | NovyDeck 0.18.1 |

Les fonctions et réserves (état de plugin inconnu, proposition IA à relire) suivent `README.fr.md` et `docs/RELEASE-0.18.1.md`. Démonstration graphique, pas une capture de matériel en fonctionnement ; aucun plugin ou appareil n'est actionné. Animation déterministe uniquement selon le numéro d'image (`useCurrentFrame`, `interpolate`, `spring`, `Easing`), sans animation CSS ou horloge temps réel. Versions Remotion 4.0.537 / Three 0.175.0 inchangées.

## Reproduction

Depuis `scripts/media/remotion`, après `npm ci` :

    python3 make-motion-sound.py
    npm run typecheck
    npx remotion still src/index.ts NovyDeckMotion out/scene.png --frame=800 --scale=0.5 --gl=swangle
    npx remotion render src/index.ts NovyDeckMotion out/m169.mp4 --gl=swangle --crf=18 --concurrency=16
    npx remotion render src/index.ts NovyDeckMotionVertical out/m916.mp4 --gl=swangle --crf=18 --concurrency=16

La bande-son stéréo 48 kHz de 55 s est synthétisée avec Python standard ; clics, glissés, dépôts, bascules et frappe sont synchronisés aux nouvelles coupes. Contrôler l'export final avec ffprobe, volumedetect, blackdetect et inspection visuelle. Inter (SIL OFL) : `public/fonts/Inter-LICENSE.txt`. Logo et textures LCD : ressources NovyDeck.
