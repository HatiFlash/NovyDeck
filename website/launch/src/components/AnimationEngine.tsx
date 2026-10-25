"use client";

import { useEffect } from "react";
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import Lenis from "lenis";

gsap.registerPlugin(ScrollTrigger);

/** Native scrolling on phones and whenever reduced motion is preferred. */
export default function AnimationEngine() {
  useEffect(() => {
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)");
    const desktop = window.matchMedia("(min-width: 900px)");
    let dispose = () => {};
    const configure = () => {
      dispose();
      dispose = () => {};
      if (reduced.matches || !desktop.matches) {
        document.querySelectorAll<HTMLElement>("[data-appear]").forEach(node => {
          gsap.set(node, { clearProps: "opacity,transform" });
        });
        document.querySelectorAll<HTMLElement>(".flow-step").forEach(node => node.classList.add("is-lit"));
        return;
      }
      const lenis = new Lenis({ autoRaf: false, duration: 1.1, smoothWheel: true });
      const tick = (time: number) => lenis.raf(time * 1000);
      gsap.ticker.add(tick);
      lenis.on("scroll", ScrollTrigger.update);
      const context = gsap.context(() => {
        ScrollTrigger.create({
          trigger: "#product-story", start: "top top", end: "bottom bottom",
          onUpdate(self) { window.dispatchEvent(new CustomEvent("novydeck:product-progress", { detail: self.progress })); },
        });
        ScrollTrigger.create({
          trigger: "#workflow", start: "top center", end: "bottom bottom",
          onUpdate(self) {
            const stage = Math.min(4, Math.floor(self.progress * 5));
            document.querySelectorAll<HTMLElement>(".flow-step").forEach((node, i) => node.classList.toggle("is-lit", i <= stage));
          },
        });
        gsap.utils.toArray<HTMLElement>("[data-appear]").forEach(element => {
          gsap.fromTo(element, { opacity: 0, y: 34 }, {
            opacity: 1, y: 0, duration: 0.85, ease: "power2.out",
            scrollTrigger: { trigger: element, start: "top 89%", once: true },
          });
        });
      });
      dispose = () => { context.revert(); gsap.ticker.remove(tick); lenis.destroy(); };
    };
    configure();
    reduced.addEventListener("change", configure);
    desktop.addEventListener("change", configure);
    return () => { reduced.removeEventListener("change", configure); desktop.removeEventListener("change", configure); dispose(); };
  }, []);
  return null;
}
