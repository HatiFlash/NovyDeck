# NovyDeck 0.18.1 · film animé en composants Remotion

Deux compositions de 55 s, 30 i/s : `NovyDeckMotion` (1920 × 1080) et `NovyDeckMotionVertical` (1080 × 1920). Le portrait est recomposé : fenêtre et panneaux empilés, deck cadré pour montrer les quinze touches, zone sûre supérieure/inférieure d'environ 10 %. L'interface est redessinée en composants React, sans aucune capture d'écran utilisée dans ces deux compositions. Les anciennes compositions `MotionPreview*` et `NovyDeckDemo` restent accessibles à titre historique (elles peuvent employer des captures).

## Scènes

| Temps | Séquence | Titre |
|---|---|---|
| 0–5 s | Logo en tuiles et deck 3D | Ton deck. Ton univers. |
| 5–11 s | Quinze touches LCD, survol et appui | Chaque touche compte. |
| 11–18 s | Apparence, couleur, icône et texte | À ton image. |
| 18–25 s | Carte d'action flottante, touche cible et dépôt | Glisse et crée. |
| 25–32 s | Atelier de blocs, réorganisation, exécution lumineuse | Compose tes macros. |
| 32–37 s | Bascule entre les deux faces | Deux états. |
| 37–43 s | Magasin et bibliothèque d'actions | Des plugins partout. |
| 43–49 s | Assistant, proposition à relire puis appliquer | Décris, puis choisis. |
| 49–55 s | Tuiles qui reforment le logo, version et dépôt GitHub | NovyDeck 0.18.1 |

Les scènes sont des démonstrations graphiques des fonctions décrites dans `README.fr.md` et `docs/RELEASE-0.18.1.md`, et non une capture d'exécution du logiciel ou du matériel. L'Assistant IA ne modifie rien automatiquement : la proposition doit être relue et appliquée. Aucun plugin ni appareil n'est actionné. Texte d'interface en français, palette nuit avec accent #3478f6. Le deck Three.js et les composants s'animent uniquement en fonction du numéro d'image (`useCurrentFrame`, `spring`, `interpolate`), sans animation CSS ni `useFrame`.

## Reproduction

Depuis `scripts/media/remotion` après `npm ci` (versions verrouillées dans le lockfile) :

    python3 make-motion-sound.py
    npm run typecheck
    npx remotion still src/index.ts NovyDeckMotion out/scene.png --frame=780 --scale=0.5 --gl=swangle
    npx remotion render src/index.ts NovyDeckMotion out/NovyDeck-0.18.1-motion-16x9.mp4 --gl=swangle --crf=18 --concurrency=16
    npx remotion render src/index.ts NovyDeckMotionVertical out/NovyDeck-0.18.1-motion-9x16.mp4 --gl=swangle --crf=18 --concurrency=16

`make-motion-sound.py` synthétise, avec Python standard uniquement, une piste stéréo 48 kHz déterministe de 55 s : musique, clics, glissés, dépôts, bascules et frappe synchronisés. La piste est limitée à −3 dBFS avant encodage AAC. Vérifier les exports réels avec ffprobe, `volumedetect` et inspection visuelle avant toute publication.

Inter (SIL OFL) : `public/fonts/Inter-LICENSE.txt`. Logo et textures LCD : ressources NovyDeck. Versions : Remotion 4.0.537, Three 0.175.0, sans modification des dépendances.
