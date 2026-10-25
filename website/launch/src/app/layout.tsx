import type { Metadata, Viewport } from "next";
import "./globals.css";
import { Geist } from "next/font/google";
import { cn } from "@/lib/utils";

const geist = Geist({subsets:['latin'],variable:'--font-sans'});

export const metadata: Metadata = {
  title: "NovyDeck — Votre deck. Vos règles.",
  description: "NovyDeck, application libre pour personnaliser les touches du FIFINE D6 et des Stream Deck, créer des macros visuelles et étendre votre espace de travail.",
  applicationName: "NovyDeck",
  openGraph: {
    title: "NovyDeck — Votre deck. Vos règles.",
    description: "Le deck devient votre terrain de jeu. Touches, automatisations visuelles, thèmes et plugins.",
    type: "website",
  },
  icons: { icon: "/assets/novydeck-logo.webp" },
};

export const viewport: Viewport = { width: "device-width", initialScale: 1, themeColor: "#080d16" };

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="fr" className={cn("font-sans", geist.variable)}><body>{children}</body></html>;
}
