# Chantier plugins — suivi d'exécution

Plan : [tasks/plan.md](tasks/plan.md). Base 379689b (NovyDeck 0.10.1), demandes Hatim du 25/10/2026. Jamais annoncer terminé sans preuve d'exécution Windows.

- [ ] T1 — **En cours**, worker Sol : transactions de mise à jour/désinstallation avec sauvegarde/rollback et contrôle ciblé du runtime (L).
- [ ] T2 — **En cours**, worker Luna : bibliothèque repliable et sélecteur installé, classé, défilable (M ; parallèle T1, fichiers UI distincts).
- [ ] T3 — Boutons Mettre à jour/Désinstaller, versions locales et confirmations/usages (M, Luna ; après T1+T2).
- [ ] T4 — Vrai inspecteur par bloc, ouverture au clic/bouton, réglages isolés et persistants (L, Sol ; après T1+T2).
- [ ] T5 — Tester pour de vrai pour une touche plugin seule, simulation sans effet (S/M, Luna ou Sol avec T4).
- [ ] T6 — Images ON/OFF configurables sur aperçu et deck, état inconnu/local distinct, pas de faux toggle sur erreur (L, Sol ; après T4).
- [ ] T7 — Revue ciblée puis suite Windows --locked et captures FR/EN/HiDPI (M).
- [ ] T8 — README FR/EN, notes/version cohérentes et présentation à Hatim (M ; après tests).

## Journal / preuves

- Démarrage : clone propre de main dans app-next-plugins, branche feat/plugin-management-next. Inventaire Haiku en lecture seule : pas d'instructions racine AGENTS/CLAUDE ni de plan préexistant. Supports existants identifiés : StateDisplay/StateFace, contextes virtuels d'actions de plugin.
- T2, premier parcours validé sur Windows au SHA `10d97b2` : BUILD=0, TESTBUILD=0, `workshop_plugin_catalogue_adds_action_card_and_capture` 1/1. Journal `navigation-011-parent-check.log`. Capture `workshop-plugins.png` effectivement relue : bloc et noms lisibles, pas de débordement visible ; cette capture de l'atelier ne prouve ni la navigation multi-plugins/HiDPI ni l'ouverture des vrais réglages (T4 reste à faire).
- Aucun changement au PC ni publication effectué.

## Résultat / revue

À compléter avec commits intégrés, sorties réelles des builds/tests, captures et limites restantes.
