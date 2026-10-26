"use client";

import { useEffect, useRef, useState } from "react";
import dynamic from "next/dynamic";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { ArrowDown, ArrowUpRight, Check, ChevronRight, Download, Menu, Pause, Play, Sparkles, X } from "lucide-react";
import AnimationEngine from "./AnimationEngine";

const DeckScene = dynamic(() => import("./DeckScene"), { ssr: false });
const release = "https://github.com/HatiFlash/NovyDeck/releases/download/v0.14.0/NovyDeck-0.14.0-Setup.exe";
const releases = "https://github.com/HatiFlash/NovyDeck/releases/tag/v0.14.0";
const images = {
  keys: "assets/main-hi.webp",
  macros: "assets/atelier.webp",
  state: "assets/states.webp",
  crop: "assets/crop.webp",
  assistant: "assets/assistant.webp",
  plugins: "assets/store.webp",
  dark: "assets/dark.webp",
  light: "assets/light.webp",
};

const copy = {
  fr: {
    skip: "Aller au contenu", nav: ["Produit", "Fonctionnalités", "Atelier", "Plugins"],
    download: "Télécharger", preview: "Découvrir le produit", locale: "EN", menu: "Ouvrir le menu", close: "Fermer le menu",
    heroOver: "NOVYDECK / V0.14.0 — WINDOWS", heroTitle1: "Votre deck.", heroTitle2: "Vos règles.",
    heroLead: "Donnez une forme à chaque idée. Une application native pour créer vos touches, orchestrer vos actions et étendre votre deck — sans renoncer au contrôle.",
    heroSupport: "FIFINE AmpliGame D6 · Elgato Stream Deck testés", heroCaption: "La liberté de créer, touche après touche.",
    scroll: "Défiler pour explorer", chapter: "CHAPITRE", productOver: "UN OUTIL, TOUT UN UNIVERS", productTitle: "Le bureau devient votre terrain de jeu.",
    productLead: "Un rendu 3D inspiré du D6 présente les possibilités. Les visuels du logiciel plus bas sont de véritables captures de NovyDeck.",
    story: [
      { n: "01 / LE DECK", title: "À chaque touche, sa personnalité.", body: "Icône, image, GIF ou vidéo courte : composez l’apparence des 15 touches du D6. Organisez profils, pages et dossiers, puis prévisualisez avant d’envoyer.", note: "Médias personnels préservés ; aperçu issu de l’application." },
      { n: "02 / LES ACTIONS", title: "L’intention devient une action.", body: "Un appui court, un appui long, des macros en séquence. Les plugins compatibles peuvent recevoir les événements keyDown et keyUp, selon leur implémentation.", note: "La prise en charge des événements dépend du plugin installé." },
      { n: "03 / L’IDENTITÉ", title: "Une interface qui vous ressemble.", body: "Personnalisez les faces ON/OFF et choisissez votre thème. NovyDeck distingue l’état confirmé par un plugin d’une simple bascule locale.", note: "Aucun état inconnu n’est présenté comme confirmé." },
    ],
    sceneLabel: "Reconstitution 3D illustrative du FIFINE D6 · textures issues du projet", sceneFoot: "Interaction liée au défilement · sans vidéo simulant une interface",
    showcaseOver: "VOIR, PUIS AGIR", showcaseTitle: "Le détail change tout.", showcaseLead: "Nouveautés 0.15 : état des touches actif ou en pause, bibliothèque Modèles et Enregistrés, réglages Apparence dans l’inspecteur, conversation avec l’assistant et aperçu des propositions en blocs. Actions et réglages de plugins validés, pas supposés. Les captures ci-dessous sont historiques 0.14.",
    showcase: [
      { label: "Touches", number: "01", title: "Le deck, en grand.", body: "Glissez une idée sur une touche. Réglez son action, son image et son comportement avant l’exécution.", detail: "Capture réelle du profil de démonstration ; aucun périphérique connecté.", img: images.keys, alt: "Interface NovyDeck avec les 15 touches du profil de démonstration" },
      { label: "États ON / OFF", number: "02", title: "Le bon état, au bon moment.", body: "Deux faces pour une touche ; une source d’état plugin compatible peut confirmer la face active.", detail: "Un état inconnu garde la face de base au lieu d’inventer OFF.", img: images.state, alt: "Configuration des faces ON et OFF de NovyDeck" },
      { label: "Médias", number: "03", title: "Cadrez sans abîmer.", body: "Zoom, déplacement et aperçu de votre média, sans modifier le fichier original.", detail: "Le recadrage concerne le rendu dans l’application et sur la touche.", img: images.crop, alt: "Éditeur de recadrage média de NovyDeck" },
      { label: "Atelier 0.14", number: "04", title: "Glissez, imbriquez, dupliquez.", body: "Réorganisez les blocs par glisser-déposer, placez-les dans Alors, Sinon ou Répéter et maintenez Ctrl pour dupliquer pendant le glissement.", detail: "Capture réelle de l’Atelier en français, rendue hors écran ; aucun D6 physique testé.", img: "assets/v014-workshop-fr.webp", alt: "Capture française de l’Atelier 0.14 avec bibliothèque, blocs et branches Alors/Sinon" },
      { label: "Quoi de neuf", number: "05", title: "Une mise à jour, expliquée.", body: "L’interface actualisée présente les nouveautés dans une fenêtre Quoi de neuf et sépare téléchargement, vérification, installation et relance.", detail: "Capture réelle de la fenêtre Quoi de neuf 0.14.0 en français.", img: "assets/v014-whats-new-fr.webp", offer: "assets/v014-update-fr.webp", alt: "Fenêtre Quoi de neuf présentant les nouveautés 0.14.0" },
      { label: "Assistant", number: "06", title: "Des actions parmi vos plugins installés.", body: "L’assistant s’appuie sur les manifestes des plugins installés. Il peut demander une précision ; cela ne promet pas de comprendre toute demande.", detail: "L’IA locale reste facultative ; relisez les propositions. Pas de test sur D6 physique.", img: "assets/assistant.webp", alt: "Proposition de macro dans l’assistant NovyDeck" },
    ],
    workflowOver: "ATELIER / 0.14.0", workflowTitle: "Glissez les blocs. Gardez la main.", workflowLead: "Réordonnez ou imbriquez les blocs dans Alors, Sinon ou Répéter ; maintenez Ctrl pendant le glissement pour dupliquer. Annulez ou rétablissez vos changements.",
    flowLabels: ["Déclencheur", "Attendre", "Raccourci", "Condition", "Notification"], flowValues: ["Appui sur la touche", "300 ms", "Ctrl + C", "Application active ?", "Action terminée"],
    workflowNote: "Illustration de la logique de l’Atelier. Les blocs visibles dans la capture ci-dessous proviennent de l’application.", workflowShot: "Capture réelle de l’Atelier de blocs de NovyDeck.",
    themesOver: "APPARENCE", themesTitle: "Le fond change. Pas votre façon de travailler.", themesLead: "Un thème sombre pour la concentration, un thème clair pour la netteté. Le deck et vos images restent les vôtres.", dark: "Sombre", light: "Clair", themesAltDark: "Interface NovyDeck en thème sombre", themesAltLight: "Interface NovyDeck en thème clair", themeCaption: "Captures réelles du profil de démonstration.",
    aiOver: "ASSISTANT LOCAL / 0.14.0", aiTitle: "Des actions issues des plugins installés.", aiLead: "L’assistant s’appuie sur les manifestes des plugins installés pour proposer des actions et peut demander une précision si la cible est ambiguë. Il ne comprend pas nécessairement toute demande.",
    aiAvail: "Disponible", aiAvailText: "Suggestions fondées sur des règles locales et aperçu des blocs.", aiOpt: "Optionnel", aiOptText: "Modèle IA local via composants à configurer, désactivé par défaut.", aiFuture: "À explorer", aiFutureText: "Des aides plus avancées restent une piste, pas une fonction annoncée.", aiCaption: "Capture réelle d’une proposition dans l’assistant local.",
    pluginOver: "ÉCOSYSTÈME", pluginTitle: "Le deck ne s’arrête pas à quinze touches.", pluginLead: "Parcourez les plugins officiels et communautaires, consultez leurs prérequis puis installez après confirmation. Les actions des plugins s’enchaînent aussi dans l’Atelier.",
    pluginItems: ["Magasin intégré", "Actions de plugins dans les macros", "Mises à jour avec sauvegarde ciblée", "Compatibilité avec des plugins Stream Deck non chiffrés"],
    pluginCaveat: "La compatibilité varie selon les plugins. Les archives chiffrées ne sont pas prises en charge.", pluginCaption: "Capture réelle du magasin de plugins NovyDeck.",
    bridgeOver: "VOTRE ENVIRONNEMENT, VOS CHOIX", bridgeTitle: "Des liens utiles. Pas une boîte noire.", bridgeLead: "Home Assistant et Google Assistant passent par des plugins. Le serveur MCP propose des opérations à votre client compatible ; l’utilisateur garde la validation.",
    bridgeNote: "Les intégrations tierces peuvent nécessiter leurs propres services et comptes.",
    filmOver: "LE FILM", filmTitle: "Voyez le produit prendre forme.", filmLead: "Un extrait 1080p de la vidéo officielle. Le D6 est une animation 3D ; les captures de l’application sont réelles.", play: "Lire le film", pause: "Mettre en pause",
    ctaOver: "À VOUS DE JOUER", ctaTitle: "Donnez un nouveau sens à chaque touche.", ctaLead: "NovyDeck est libre et gratuit. Téléchargez l’installateur Windows 0.14.0 ou consultez les autres fichiers publiés.", other: "Autres téléchargements", ctaSmall: "Windows 10/11 · 64 bits · licence MIT · FIFINE D6 et Elgato Stream Deck testés", footer: "Logiciel libre. Conçu pour que chaque touche soit à vous.", source: "Code source", top: "Retour en haut", file: "Installateur Windows v0.14.0",
  },
  en: {
    skip: "Skip to content", nav: ["Product", "Features", "Workshop", "Plugins"],
    download: "Download", preview: "Explore the product", locale: "FR", menu: "Open menu", close: "Close menu",
    heroOver: "NOVYDECK / V0.14.0 — WINDOWS", heroTitle1: "Your deck.", heroTitle2: "Your rules.",
    heroLead: "Give every idea a place. A native app to design keys, orchestrate actions and extend your deck — while keeping control.",
    heroSupport: "FIFINE AmpliGame D6 · tested Elgato Stream Deck", heroCaption: "The freedom to create, one key at a time.",
    scroll: "Scroll to explore", chapter: "CHAPTER", productOver: "ONE TOOL, A WHOLE UNIVERSE", productTitle: "Make your desktop your playground.",
    productLead: "A D6-inspired 3D rendering introduces the possibilities. App imagery below is made of genuine NovyDeck screenshots.",
    story: [
      { n: "01 / THE DECK", title: "A personality for every key.", body: "Icon, image, GIF or short video: compose the 15 D6 key faces. Organize profiles, pages and folders, then preview before sending.", note: "Personal media is preserved; app preview shown below." },
      { n: "02 / ACTIONS", title: "Turn intention into action.", body: "Short press, long press and sequenced macros. Compatible plugins may handle keyDown and keyUp events, depending on their implementation.", note: "Event support depends on the installed plugin." },
      { n: "03 / IDENTITY", title: "An interface that feels like yours.", body: "Customize ON/OFF key faces and pick a theme. NovyDeck distinguishes a plugin-confirmed state from a local toggle.", note: "An unknown state is never presented as confirmed." },
    ],
    sceneLabel: "Illustrative 3D reconstruction of the FIFINE D6 · textures from the project", sceneFoot: "Scroll-linked interaction · no simulated app video",
    showcaseOver: "SEE, THEN DO", showcaseTitle: "The details make the difference.", showcaseLead: "0.15 highlights: keys shown as running or paused, Templates and Saved library sections, Appearance controls in the inspector, assistant conversation, and block previews for proposals. Plugin actions and settings are validated, not assumed. The screenshots below are historical 0.14 captures.",
    showcase: [
      { label: "Keys", number: "01", title: "The whole deck at a glance.", body: "Put an idea on a key. Set its action, image and behavior before execution.", detail: "Real demo-profile screenshot; no device connected.", img: images.keys, alt: "NovyDeck interface with the 15 demo-profile keys" },
      { label: "ON / OFF", number: "02", title: "The right state at the right time.", body: "Two faces for one key; a compatible plugin state source can confirm the active face.", detail: "An unknown state keeps the base face instead of claiming OFF.", img: images.state, alt: "NovyDeck ON and OFF key face configuration" },
      { label: "Media", number: "03", title: "Crop without damage.", body: "Zoom, position and preview your media without changing the original file.", detail: "Cropping affects the app and key preview.", img: images.crop, alt: "NovyDeck media crop editor" },
      { label: "Workshop 0.14", number: "04", title: "Drag, nest, duplicate.", body: "Reorder blocks with drag and drop, nest them under Then, Else or Repeat, and hold Ctrl while dragging to duplicate.", detail: "Real off-screen English UI capture with French sample data; no physical D6 tested.", img: "assets/v014-workshop-en.webp", alt: "English 0.14 Workshop screenshot with library, blocks and Then/Else branches" },
      { label: "What’s new", number: "05", title: "An update, clearly explained.", body: "The refreshed interface presents changes in a What’s New window and separates download, verification, installation and relaunch.", detail: "Real screenshot of the 0.14.0 What’s New window in English.", img: "assets/v014-whats-new-en.webp", offer: "assets/v014-update-en.webp", alt: "What’s New window presenting the 0.14.0 changes" },
      { label: "Assistant", number: "06", title: "Actions from installed plugins.", body: "The assistant draws on manifests from installed plugins. It may ask for clarification; this is not a promise to understand every request.", detail: "Local AI remains optional; review suggestions. No physical D6 test.", img: images.assistant, alt: "Macro proposal in the NovyDeck assistant" },
    ],
    workflowOver: "WORKSHOP / 0.14.0", workflowTitle: "Drag blocks. Stay in control.", workflowLead: "Reorder or nest blocks under Then, Else or Repeat; hold Ctrl while dragging to duplicate. Undo or redo your changes.",
    flowLabels: ["Trigger", "Wait", "Shortcut", "Condition", "Notification"], flowValues: ["Key pressed", "300 ms", "Ctrl + C", "App in focus?", "Action finished"],
    workflowNote: "Illustration of the workshop logic. The blocks in the screenshot below come from the real app.", workflowShot: "Real screenshot of the NovyDeck block workshop.",
    themesOver: "APPEARANCE", themesTitle: "Change the canvas. Keep your workflow.", themesLead: "A dark theme for focus, a light theme for clarity. Your deck and images remain yours.", dark: "Dark", light: "Light", themesAltDark: "NovyDeck interface in dark theme", themesAltLight: "NovyDeck interface in light theme", themeCaption: "Real demo-profile screenshots.",
    aiOver: "LOCAL ASSISTANT / 0.14.0", aiTitle: "Actions from your installed plugins.", aiLead: "The assistant uses manifests from installed plugins to suggest actions and may ask for clarification when a target is ambiguous. It cannot be assumed to understand every request.",
    aiAvail: "Available", aiAvailText: "Local rule-based suggestions and block preview.", aiOpt: "Optional", aiOptText: "Local AI model via components you configure, off by default.", aiFuture: "Exploratory", aiFutureText: "More advanced assistance is a possibility, not an announced feature.", aiCaption: "Real screenshot of a proposal in the local assistant.",
    pluginOver: "ECOSYSTEM", pluginTitle: "Your deck goes beyond fifteen keys.", pluginLead: "Browse first-party and community plugins, review requirements and confirm before installing. Plugin actions can also be chained in the workshop.",
    pluginItems: ["Built-in store", "Plugin actions inside macros", "Updates with targeted backup", "Support for unencrypted Stream Deck plugins"],
    pluginCaveat: "Support varies by plugin. Encrypted archives are not supported.", pluginCaption: "Real screenshot of the NovyDeck plugin store.",
    bridgeOver: "YOUR SETUP, YOUR CHOICE", bridgeTitle: "Useful connections. No black box.", bridgeLead: "Home Assistant and Google Assistant work through plugins. The MCP server proposes operations to a compatible client; the user retains approval.",
    bridgeNote: "Third-party integrations may require their own services and accounts.",
    filmOver: "THE FILM", filmTitle: "Watch the product take shape.", filmLead: "A 1080p excerpt of the official film. The D6 is a 3D animation; app screenshots are real.", play: "Play film", pause: "Pause film",
    ctaOver: "YOUR TURN", ctaTitle: "Make every key count.", ctaLead: "NovyDeck is free and open-source. Download the Windows 0.14.0 installer or browse the other published files.", other: "Other downloads", ctaSmall: "Windows 10/11 · 64-bit · MIT license · tested FIFINE D6 and Elgato Stream Deck", footer: "Open-source software. Made so every key belongs to you.", source: "Source code", top: "Back to top", file: "Windows installer v0.14.0",
  },
};

type Language = keyof typeof copy;
function DeviceFallback() {
  return <div className="device-fallback" aria-hidden="true"><div className="device-face">{Array.from({ length: 15 }, (_, i) => <div className="device-key" key={i}><img src={`assets/key-${String(i).padStart(2, "0")}.webp`} alt="" /></div>)}</div></div>;
}

function Film({ text }: { text: typeof copy.fr }) {
  const ref = useRef<HTMLVideoElement>(null);
  const [playing, setPlaying] = useState(false);
  const manuallyPaused = useRef(false);
  useEffect(() => {
    const film = ref.current;
    if (!film) return;
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)");
    const observer = new IntersectionObserver(([entry]) => {
      if (!entry.isIntersecting || document.hidden || reduce.matches) { film.pause(); return; }
      if (window.innerWidth >= 768 && !manuallyPaused.current && !((navigator as Navigator & { connection?: { saveData?: boolean } }).connection?.saveData)) film.play().catch(() => setPlaying(false));
    }, { threshold: 0.2 });
    const visibility = () => { if (document.hidden) film.pause(); };
    const motionChange = () => { if (reduce.matches) film.pause(); };
    observer.observe(film);
    document.addEventListener("visibilitychange", visibility);
    reduce.addEventListener("change", motionChange);
    return () => { observer.disconnect(); document.removeEventListener("visibilitychange", visibility); reduce.removeEventListener("change", motionChange); film.pause(); };
  }, []);
  const toggle = () => {
    const film = ref.current;
    if (!film) return;
    if (film.paused) { manuallyPaused.current = false; film.play().catch(() => setPlaying(false)); }
    else { manuallyPaused.current = true; film.pause(); }
  };
  return <section id="film" className="film-section" aria-labelledby="film-title">
    <video ref={ref} muted loop playsInline preload="none" poster="assets/video-poster.webp" onPlay={() => setPlaying(true)} onPause={() => setPlaying(false)} aria-hidden="true"><source src="assets/novydeck-motion.mp4" type="video/mp4" /></video>
    <div className="film-shade" />
    <div className="film-copy frame"><p className="eyebrow">{text.filmOver}</p><h2 id="film-title">{text.filmTitle}</h2><p>{text.filmLead}</p><button type="button" className="film-toggle" onClick={toggle} aria-pressed={playing}>{playing ? <Pause size={16} /> : <Play size={16} />}{playing ? text.pause : text.play}</button></div>
  </section>;
}

export default function LaunchExperience() {
  const [language, setLanguage] = useState<Language>("fr");
  const [menuOpen, setMenuOpen] = useState(false);
  const [theme, setTheme] = useState<"dark" | "light">("dark");
  const [canRender3D, setCanRender3D] = useState(false);
  const [sceneNear, setSceneNear] = useState(false);
  const reduced = useReducedMotion();
  const text = copy[language];
  useEffect(() => {
    const saved = window.localStorage.getItem("novydeck-launch-language");
    if (saved === "en") setLanguage("en");
    const media = window.matchMedia("(min-width: 900px) and (prefers-reduced-motion: no-preference)");
    const update = () => setCanRender3D(media.matches);
    update(); media.addEventListener("change", update);
    return () => media.removeEventListener("change", update);
  }, []);
  useEffect(() => {
    const node = document.querySelector(".scene-canvas");
    if (!node) return;
    const observer = new IntersectionObserver(([entry]) => {
      if (entry.isIntersecting) { setSceneNear(true); observer.disconnect(); }
    }, { rootMargin: "100px" });
    observer.observe(node);
    return () => observer.disconnect();
  }, []);
  useEffect(() => { document.documentElement.lang = language; window.localStorage.setItem("novydeck-launch-language", language); }, [language]);
  const anchors = ["#product-story", "#showcase", "#workflow", "#plugins"];
  return <>
    <AnimationEngine />
    <a className="skip-link" href="#main">{text.skip}</a>
    <header className="site-header"><div className="header-inner frame">
      <a className="brand" href="#top" aria-label="NovyDeck"><img src="assets/novydeck-logo.webp" alt="" width="28" height="28" /><span>NovyDeck</span><i aria-hidden="true" /></a>
      <nav className={`nav-links ${menuOpen ? "open" : ""}`} aria-label={language === "fr" ? "Navigation principale" : "Main navigation"}>{anchors.map((href, i) => <a href={href} key={href} onClick={() => setMenuOpen(false)}>{text.nav[i]}</a>)}</nav>
      <div className="header-actions"><button type="button" className="locale" onClick={() => setLanguage(language === "fr" ? "en" : "fr")} aria-label={language === "fr" ? "EN — Switch to English" : "FR — Passer en français"}>{text.locale}</button><a className="header-download" href="#download">{text.download}<ArrowUpRight size={14} /></a><button className="mobile-menu" type="button" aria-label={menuOpen ? text.close : text.menu} aria-expanded={menuOpen} onClick={() => setMenuOpen(!menuOpen)}>{menuOpen ? <X size={21} /> : <Menu size={21} />}</button></div>
    </div></header>
    <main id="main"><section id="top" className="hero" aria-labelledby="hero-title"><div className="hero-grain" aria-hidden="true" /><div className="frame hero-layout"><div className="hero-type"><p className="eyebrow hero-eyebrow"><span className="status-dot" />{text.heroOver}</p><h1 id="hero-title"><span>{text.heroTitle1}</span><em>{text.heroTitle2}</em></h1><p className="hero-lead">{text.heroLead}</p><div className="hero-actions"><a className="primary-link" href={release}><Download size={17} />{text.download}<ArrowUpRight size={17} /></a><a className="under-link" href="#product-story">{text.preview}<ChevronRight size={17} /></a></div><p className="hero-support">{text.heroSupport}</p></div><div className="hero-art" aria-hidden="true"><div className="hero-orbit orbit-a" /><div className="hero-orbit orbit-b" /><div className="hero-snapshot"><img src="assets/main-hi.webp" alt="" fetchPriority="high" /></div><div className="hero-key key-float-a"><img src="assets/key-09.webp" alt="" /></div><div className="hero-key key-float-b"><img src="assets/key-05.webp" alt="" /></div><div className="hero-art-label">NOVY / D6 — 15 KEYS</div></div></div><div className="hero-bottom frame"><span>{text.heroCaption}</span><a href="#product-story">{text.scroll}<ArrowDown size={16} /></a></div></section>
      <section id="product-story" className="product-story" aria-labelledby="product-title"><div className="frame"><div className="story-intro"><p className="eyebrow">01 / {text.productOver}</p><h2 id="product-title">{text.productTitle}</h2><p>{text.productLead}</p></div><div className="product-grid"><div className="scene-column"><div className="scene-sticky"><div className="scene-halo" /><div className="scene-canvas">{canRender3D && sceneNear && !reduced ? <DeckScene /> : <DeviceFallback />}</div><div className="scene-chrome"><span>FIFINE / D6</span><span>15 / LCD</span></div><p className="scene-label">{text.sceneLabel}</p><small>{text.sceneFoot}</small></div></div><div className="story-chapters">{text.story.map((chapter) => <article className="story-chapter" key={chapter.n} data-appear><span className="chapter-number">{chapter.n}</span><h3>{chapter.title}</h3><p>{chapter.body}</p><small>{chapter.note}</small></article>)}</div></div></div></section>
      <section id="showcase" className="showcase-section" aria-labelledby="showcase-title"><div className="frame"><div className="section-header" data-appear><p className="eyebrow">02 / {text.showcaseOver}</p><h2 id="showcase-title">{text.showcaseTitle}</h2><p>{text.showcaseLead}</p></div><Tabs className="showcase-tabs" defaultValue="0"><TabsList variant="line" aria-label={language === "fr" ? "Fenêtres NovyDeck" : "NovyDeck windows"}>{text.showcase.map((item, i) => <TabsTrigger value={String(i)} key={i}>{item.number}<span>{item.label}</span></TabsTrigger>)}</TabsList>{text.showcase.map((item, i) => <TabsContent value={String(i)} key={i}><div className="showcase-body"><div className="capture-wrap"><div className="capture-top"><span>● ● ●</span><span>NOVYDECK / {item.number}</span></div><a href={item.img} target="_blank" rel="noopener noreferrer" aria-label={language === "fr" ? `Agrandir la capture : ${item.title}` : `Enlarge screenshot: ${item.title}`}><img src={item.img} alt={item.alt} loading="lazy" /></a><div className="capture-hover"><strong>{item.title}</strong><span>{item.detail}</span></div></div><div className="showcase-info"><span className="micro">{item.number} / 06</span><h3>{item.title}</h3><p>{item.body}</p><div className="info-rule" /><small>{item.detail}</small>{"offer" in item && <a className="showcase-offer-link" href={item.offer} target="_blank" rel="noopener noreferrer">{language === "fr" ? "Voir l’offre de mise à jour 0.13 → 0.14 (capture hors ligne)" : "View the 0.13 → 0.14 update offer (offline capture)"}<ArrowUpRight size={15} /></a>}</div></div></TabsContent>)}</Tabs></div></section>
      <section id="workflow" className="workflow-section" aria-labelledby="workflow-title"><div className="frame"><div className="section-header workflow-head" data-appear><p className="eyebrow">03 / {text.workflowOver}</p><h2 id="workflow-title">{text.workflowTitle}</h2><p>{text.workflowLead}</p></div><div className="workflow-visual"><div className="workflow-board"><div className="board-head"><span className="board-led" /> NOVYDECK <span> / BLOCK WORKSHOP</span></div><div className="flow-steps">{text.flowLabels.map((label, i) => <div className={`flow-step ${i === 0 ? "is-lit" : ""}`} key={i}><span className="step-index">{String(i + 1).padStart(2, "0")}</span><div><strong>{label}</strong><small>{text.flowValues[i]}</small></div><span className="step-port" /></div>)}</div><div className="flow-connect" aria-hidden="true" /></div><div className="workflow-context"><span className="micro">MACRO / 05 BLOCKS</span><p>{text.workflowNote}</p><div className="workflow-shot"><img src={language === "fr" ? "assets/v014-workshop-fr.webp" : "assets/v014-workshop-en.webp"} alt={text.workflowShot} loading="lazy" /></div><small>{text.workflowShot}</small></div></div></div></section>
      <section id="themes" className="themes-section" aria-labelledby="themes-title"><div className="frame theme-layout"><div className="theme-copy" data-appear><p className="eyebrow">04 / {text.themesOver}</p><h2 id="themes-title">{text.themesTitle}</h2><p>{text.themesLead}</p><div className="theme-switch" role="group" aria-label={language === "fr" ? "Choix du thème illustré" : "Illustrated theme choice"}><button type="button" className={theme === "dark" ? "active" : ""} aria-pressed={theme === "dark"} onClick={() => setTheme("dark")}>{text.dark}</button><button type="button" className={theme === "light" ? "active" : ""} aria-pressed={theme === "light"} onClick={() => setTheme("light")}>{text.light}</button></div><small>{text.themeCaption}</small></div><div className="theme-viewport"><AnimatePresence mode="wait"><motion.img key={theme} src={theme === "dark" ? images.dark : images.light} alt={theme === "dark" ? text.themesAltDark : text.themesAltLight} initial={reduced ? false : { opacity: 0, scale: 1.025 }} animate={{ opacity: 1, scale: 1 }} exit={reduced ? undefined : { opacity: 0, scale: 0.985 }} transition={{ duration: 0.35 }} /></AnimatePresence><span className="theme-watermark">{theme === "dark" ? text.dark : text.light}</span></div></div></section>
      <section id="assistant" className="assistant-section" aria-labelledby="assistant-title"><div className="frame"><div className="section-header" data-appear><p className="eyebrow">05 / {text.aiOver}</p><h2 id="assistant-title">{text.aiTitle}</h2><p>{text.aiLead}</p></div><div className="assistant-layout"><figure className="assistant-shot"><img src={images.assistant} alt={text.aiCaption} loading="lazy" /><figcaption>{text.aiCaption}</figcaption></figure><div className="ai-status"><div><span className="status-tag available"><Check size={14} />{text.aiAvail}</span><p>{text.aiAvailText}</p></div><div><span className="status-tag optional"><Sparkles size={14} />{text.aiOpt}</span><p>{text.aiOptText}</p></div><div><span className="status-tag future">↗ {text.aiFuture}</span><p>{text.aiFutureText}</p></div></div></div></div></section>
      <section id="plugins" className="plugins-section" aria-labelledby="plugins-title"><div className="frame"><div className="section-header" data-appear><p className="eyebrow">06 / {text.pluginOver}</p><h2 id="plugins-title">{text.pluginTitle}</h2><p>{text.pluginLead}</p></div><figure className="plugins-shot"><img src={images.plugins} alt={text.pluginCaption} loading="lazy" /><figcaption>{text.pluginCaption}</figcaption></figure><div className="plugin-facts">{text.pluginItems.map((item, i) => <div key={item}><span className="micro">0{i + 1}</span><p>{item}</p></div>)}</div><p className="fine-print">{text.pluginCaveat}</p></div></section>
      <section id="integrations" className="integrations-section" aria-labelledby="integrations-title"><div className="frame integration-layout"><div><p className="eyebrow">07 / {text.bridgeOver}</p><h2 id="integrations-title">{text.bridgeTitle}</h2><p>{text.bridgeLead}</p><small>{text.bridgeNote}</small></div><div className="integration-orbit" aria-hidden="true"><span>NOVYDECK</span><i className="orbit-plugin plugin-a">HA</i><i className="orbit-plugin plugin-b">MCP</i><i className="orbit-plugin plugin-c">D6</i></div></div></section>
      <Film text={text} />
      <section id="download" className="download-section" aria-labelledby="download-title"><div className="frame download-inner"><p className="eyebrow">08 / {text.ctaOver}</p><h2 id="download-title">{text.ctaTitle}</h2><p>{text.ctaLead}</p><div className="download-actions"><a className="primary-link" href={release}><Download size={18} />{text.file}</a><a className="secondary-link" href={releases} target="_blank" rel="noopener noreferrer">{text.other}<ArrowUpRight size={17} /></a></div><small>{text.ctaSmall}</small></div></section>
    </main>
    <footer className="site-footer"><div className="frame footer-layout"><div><a className="brand" href="#top"><img src="assets/novydeck-logo.webp" alt="" width="27" height="27" />NovyDeck</a><p>{text.footer}</p></div><nav aria-label={language === "fr" ? "Pied de page" : "Footer"}><a href="https://github.com/HatiFlash/NovyDeck" target="_blank" rel="noopener noreferrer">{text.source}</a><a href={releases} target="_blank" rel="noopener noreferrer">v0.14.0</a><a href="#top">{text.top} ↑</a></nav></div></footer>
  </>;
}
