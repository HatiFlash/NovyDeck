import type { Metadata, Viewport } from "next";
import localFont from "next/font/local";
import "./globals.css";

const novyDisplay = localFont({
  src: "../../public/fonts/InterDisplay-Bold.woff2",
  variable: "--font-novy-display",
  weight: "700",
  display: "swap",
});
const basePath = process.env.NEXT_PUBLIC_SITE_BASE_PATH || "";

export const metadata: Metadata = {
  title: "NovyDeck — Votre deck. Vos règles.",
  description: "NovyDeck, application libre pour personnaliser les touches du FIFINE D6 et des Stream Deck, créer des macros visuelles et étendre votre espace de travail.",
  applicationName: "NovyDeck",
  openGraph: {
    title: "NovyDeck — Votre deck. Vos règles.",
    description: "Le deck devient votre terrain de jeu. Touches, automatisations visuelles, thèmes et plugins.",
    type: "website",
  },
  icons: { icon: `${basePath}/assets/novydeck-logo.webp` },
};

export const viewport: Viewport = { width: "device-width", initialScale: 1, themeColor: "#080d16" };

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="fr" className={novyDisplay.variable}><body>{children}</body></html>;
}
