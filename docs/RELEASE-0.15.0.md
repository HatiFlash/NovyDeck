# NovyDeck 0.15.0 — Clearer controls and assistant proposals

## English

- **Deck header and library:** the main screen labels key execution as running or paused, and the library names its Templates and Saved sections.
- **Key inspector:** the layout keeps initial action steps visible at compact and HiDPI sizes; Appearance controls remain accessible by scrolling.
- **Assistant entry and conversation:** an empty key offers Ask the assistant, opening Workshop → Assistant. Requests and replies appear in conversation order. Recognized proposals show a block count and structured preview, including nested branches, before the user chooses Add or Apply.
- **Deterministic rules and optional local AI:** built-in deterministic rules work without an AI model. An optional local model can handle requests beyond those rules; it does not imply universal plugin comprehension. Plugin actions/settings are validated, not assumed.

## Français

- **En-tête du deck et bibliothèque :** l'écran principal indique si l'exécution des touches est active ou en pause ; la bibliothèque nomme ses sections Modèles et Enregistrés.
- **Inspecteur de touche :** la disposition garde les premières étapes d'action visibles en petite fenêtre et en HiDPI ; les réglages Apparence restent accessibles par défilement.
- **Accès à l'assistant et conversation :** une touche vide propose Demander à l’IA et ouvre Atelier → Assistant. Demandes et réponses apparaissent dans l'ordre. Les propositions reconnues affichent leur nombre de blocs et un aperçu structuré, avec branches imbriquées, avant le choix Ajouter ou Appliquer.
- **Règles déterministes et IA locale facultative :** les règles déterministes intégrées fonctionnent sans modèle IA. Un modèle local facultatif peut traiter les demandes au-delà de ces règles ; cela ne signifie pas qu'il comprend tous les plugins. Actions et réglages des plugins sont validés, pas supposés.

## Qualification

The README sections include eight 0.15.0 captures generated and visually reviewed through the off-screen Windows UI harness at 1366 × 768 in English and French, dark and light themes. Targeted checks reported BUILD=0 and TESTBUILD=0; `T_main015_captures=0`, `T_main015_` ran 7 tests, `T_assistant015_` ran 3 tests, and `T_i18n::` ran 3 tests. These results cover the named harness checks only, not all tests. The proposal screenshot's English example request is French fixture data; the clarification screenshot uses an offline SoundPad test fixture, not a real installed plugin. No D6 hardware, real plugin behavior, or local model was tested, and no publication or blanket all-tests-passing claim is made.

Les sections README comprennent huit captures 0.15.0 générées et relues visuellement avec le harnais UI Windows hors écran, en 1366 × 768, en anglais et en français, dans les thèmes sombre et clair. Les contrôles ciblés ont indiqué BUILD=0 et TESTBUILD=0 ; `T_main015_captures=0`, `T_main015_` a exécuté 7 tests, `T_assistant015_` en a exécuté 3 et `T_i18n::` en a exécuté 3. Ces résultats couvrent uniquement les contrôles nommés du harnais, pas tous les tests. La demande anglaise de la capture de proposition est une donnée de fixture en français ; la capture de clarification utilise une fixture SoundPad hors ligne, pas un plugin réellement installé. Aucun matériel D6, comportement réel de plugin ou modèle local n'a été testé ; aucune publication ni réussite générale de l'ensemble des tests n'est revendiquée.
