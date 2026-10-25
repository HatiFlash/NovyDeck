# NovyDeck — expérience de lancement

Refonte indépendante du site vitrine, développée dans `website/launch/`. Elle **ne remplace pas** `website/index.html` ; aucune publication ni mise à jour du logiciel Rust n'est impliquée.

## Démarrer

Node.js 22 ou plus récent et npm sont requis. Depuis `website/launch/` :

```sh
npm ci
npm run dev
# http://127.0.0.1:3000/
```

Pour tester l'export statique qui sera distribué :

```sh
npm run typecheck
npm run build
python3 -m http.server 8766 --directory out --bind 127.0.0.1
# http://127.0.0.1:8766/
```

Dans un autre terminal : `npm run test:e2e` (Chromium installé via Playwright), puis `node tests/perf.mjs`. Les captures et `performance.json` sont sous `test-results/`, ignoré par Git. Le script de performance suppose que le serveur statique répond sur `127.0.0.1:8766` ; ses chiffres sont locaux et ne constituent pas un score Lighthouse.

## Architecture et comportement

- Next.js, TypeScript, Tailwind CSS 4 ; export statique `out/`. Le site doit être servi à la **racine** de l'hôte. Les chemins des polices et de l'icône sont absolus ; pour un sous-chemin, ajuster `basePath`, `assetPrefix`, les URLs des médias et les métadonnées puis retester l'export.
- Onglets du showcase générés par la CLI shadcn (`src/components/ui/tabs.tsx`, primitif Radix). Motion anime uniquement le passage entre captures sombre/clair. GSAP/ScrollTrigger règle la révélation des chapitres, les cinq étapes illustratives de l'Atelier et la rotation de la scène en fonction du scroll. Lenis est réservé aux écrans de 900 px ou plus sans préférence de réduction des animations ; les changements de préférence sont pris en compte.
- La scène Three.js / React Three Fiber est chargée **à l'approche** de sa section et seulement sur grand écran sans réduction des animations. Sur mobile, ou avec `prefers-reduced-motion`, un deck CSS statique utilise les mêmes textures de touches. WebGL fonctionne en rendu à la demande. La vidéo 1080p n'est pas préchargée ; affichage du poster, lecture automatique seulement sur grand écran visible sans réduction des animations ni économie de données, bouton lecture/pause explicite.
- Navigation clavier, lien « aller au contenu », onglets Radix, thème illustré à boutons avec `aria-pressed`, FR/EN et menu mobile. Les images de captures comportent des textes alternatifs ; le montage 3D est annoncé comme illustratif.

## Médias et véracité

- `public/assets/main-hi.webp`, `dark.webp` : capture réelle dérivée de `docs/images/novydeck-main.png` ; `light.webp` : `docs/images/redesign-light-fr.png` ; `atelier.webp` : `docs/images/novydeck-block-editor.png` ; `store.webp` : `docs/images/v0111-store-fr.png` ; `assistant.webp` : `docs/images/novydeck-assistant.png` ; `states.webp` : `docs/images/v011-plugin-state-on-fr.png` ; `crop.webp` : `docs/images/novydeck-media-crop.png`. Les autres copies de captures sous `public/assets/` viennent de ces mêmes sources.
- Les quinze `key-00.webp`…`key-14.webp` proviennent des textures utilisées par `scripts/media/remotion/public/keys/`. La forme de la scène 3D reprend le modèle `scripts/media/remotion/src/Deck3D.tsx` : **illustration matérielle**, pas photographie du D6. Le schéma séquencé dans la section Atelier est également une **illustration**, accompagnée d'une véritable capture du logiciel.
- `novydeck-motion.mp4` est un extrait de 11 s issu de `docs/media/novydeck-demo.mp4` encodé depuis la source Full HD ; `video-poster.webp` provient du poster du projet. Le logo vient de `res/branding/novydeck-logo.png`. La police InterDisplay est fournie avec sa licence dans `public/fonts/`.
- Les fonctionnalités et limites correspondent à `README.fr.md` et `docs/PLUGINS-COMPATIBILITE.md` : actions et suggestions locales présentes, modèle IA local **optionnel et désactivé par défaut**, perspectives futures signalées comme telles. Les événements `keyDown` / `keyUp` sont reçus selon les plugins compatibles ; aucune compatibilité universelle n'est promise. Un état ON/OFF inconnu ne devient pas un état confirmé. Les intégrations peuvent dépendre de services tiers.
- Le téléchargement pointe vers l'installateur GitHub `v0.11.1` et la page de cette version ; les deux ont été vérifiés via l'API GitHub pendant la création du site. Mettre à jour **ensemble** version, liens, captures et mentions de compatibilité à la prochaine sortie.

## Vérifications réalisées localement

`npm run typecheck`, `npm run build` et les cinq scénarios Playwright sont passés sur l'export statique : bureau, téléphone 390 px, petit téléphone 320 px, formats intermédiaires 768/1024 px, réduction des animations, onglets, thème, traduction, navigation, images, erreurs JS, requêtes en échec, débordement horizontal et audit axe des erreurs d'accessibilité graves. Le contrôle Chrome DevTools Protocol `tests/perf.mjs` a mesuré FCP et LCP autour de 404 ms et CLS ~0,0003 sur serveur local, CPU simulé ×4 avec réseau local rapide ; **ces valeurs ne prédisent pas la performance réelle sur Internet**. Le chargement différé de WebGL a réduit les octets transférés à l'ouverture d'environ 2,28 Mo à 1,39 Mo dans ce protocole de test.

Le MCP 21st n'était pas connecté à l'environnement. Son CLI a refusé une recherche hors connexion (« Not signed in ») : aucune ressource 21st n'a été importée, aucun identifiant n'a été demandé ni inventé. La CLI shadcn, Playwright et Chrome DevTools Protocol ont bien été utilisés. Une évaluation visuelle automatisée externe des captures a été tentée mais bloquée par quota ; les contrôles géométriques et OCR ne remplacent pas une revue artistique humaine.

## Publication

**Non publiée.** Garder l'ancienne page `website/index.html` disponible jusqu'à validation humaine de la refonte. Avant la mise en ligne : fixer le domaine, vérifier les métadonnées sociales et le chemin de base réel, exécuter les tests sur l'hébergement cible et obtenir l'accord pour remplacer la page publique.
