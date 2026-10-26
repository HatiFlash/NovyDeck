import { chromium } from "playwright";

const target = process.argv[2] || "http://127.0.0.1:8767/NovyDeck/";
const browser = await chromium.launch({ headless: true, args: ["--enable-unsafe-swiftshader"] });
const report = [];
let failed = false;
for (const [name, width, height] of [["desktop", 1440, 900], ["mobile", 390, 844], ["narrow", 320, 700]]) {
  const page = await browser.newPage({ viewport: { width, height } });
  const problems = [];
  page.on("pageerror", error => problems.push(`JS: ${error.message}`));
  page.on("response", response => { if (response.status() >= 400) problems.push(`HTTP ${response.status()}: ${response.url()}`); });
  try {
    const response = await page.goto(target, { waitUntil: "networkidle", timeout: 30_000 });
    if (response?.status() !== 200) problems.push(`Document HTTP ${response?.status()}`);
    await page.locator("h1").waitFor({ timeout: 10_000 });
    await page.evaluate(() => document.fonts.ready);
    await page.locator("#product-story").scrollIntoViewIfNeeded();
    if (name === "desktop") await page.locator(".scene-canvas canvas").waitFor({ timeout: 15_000 });
    else if (await page.locator(".scene-canvas canvas").count()) problems.push("WebGL mounted on mobile");
    for (const id of ["showcase", "workflow", "themes", "assistant", "plugins", "integrations", "film", "download"]) {
      await page.locator(`#${id}`).scrollIntoViewIfNeeded();
      await page.waitForTimeout(100);
    }
    if (await page.getByRole("tab").count() !== 10) problems.push("Expected ten showcase tabs");
    const checkImage = async (panel, description, file, language) => {
      const image = panel.locator("img");
      if (!(await panel.locator(`a[href*='${file}']`).count())) problems.push(`${description} full-size image link missing`);
      await image.scrollIntoViewIfNeeded();
      try {
        await image.evaluate(img => new Promise((resolve, reject) => {
          const timer = setTimeout(() => reject(new Error("image load timeout")), 10_000);
          const check = () => { if (img.naturalWidth > 0) { clearTimeout(timer); resolve(); } else if (img.complete) { clearTimeout(timer); reject(new Error("broken image")); } };
          img.addEventListener("load", check, { once: true });
          img.addEventListener("error", check, { once: true });
          check();
        }));
      } catch (error) { problems.push(`${description} ${language} screenshot failed to load: ${error.message}`); }
    };
    const cards = [
      ["01", "Touches 0.15", "Keys 0.15", "v015-main"],
      ["02", "Apparence 0.15", "Appearance 0.15", "v015-appearance"],
      ["03", "Assistant 0.15", "Assistant 0.15", "v015-assistant"],
      ["04", "Précision plugin 0.15", "Plugin clarification 0.15", "v015-question"],
    ];
    for (const language of ["fr", "en"]) {
      if (language === "en") await page.getByRole("button", { name: /Switch to English/ }).click();
      if (await page.locator("html").getAttribute("lang") !== language) problems.push(`${language} language toggle failed`);
      for (const [index, frLabel, enLabel, file] of cards) {
        await page.getByRole("tab", { name: new RegExp(`${index}.*${language === "fr" ? frLabel : enLabel}`) }).click();
        const panel = page.locator("#showcase [role=tabpanel][data-state=active]");
        if (!(await panel.locator(".micro").textContent())?.includes(`${index} / 10`)) problems.push(`${language} counter ${index} incorrect`);
        await checkImage(panel, file, `${file}-${language}.webp`, language);
        if (file === "v015-assistant" && !(await panel.locator("img").getAttribute("alt"))?.includes(language === "fr" ? "en français" : "French example request")) problems.push(`${language} assistant request-language disclosure missing`);
        if (file === "v015-question" && !(await panel.locator("img").getAttribute("alt"))?.includes(language === "fr" ? "inerte" : "inert")) problems.push(`${language} inert fixture disclosure missing`);
        if (file === "v015-main" && !(await panel.locator("img").getAttribute("alt"))?.includes(language === "fr" ? "connexion simulée" : "simulated connection")) problems.push(`${language} simulated connection disclosure missing`);
      }
      for (const [index, frLabel, enLabel, file] of [["05", "Touches", "Keys", "main-hi.webp"], ["06", "États ON", "ON / OFF", "states.webp"], ["07", "Médias", "Media", "crop.webp"], ["08", "Atelier 0.14", "Workshop 0.14", `v014-workshop-${language}.webp`], ["09", "Quoi de neuf", "What’s new", `v014-whats-new-${language}.webp`], ["10", "Assistant", "Assistant", "assistant.webp"]]) {
        await page.getByRole("tab", { name: new RegExp(`^${index}.*${language === "fr" ? frLabel : enLabel}`) }).click();
        const panel = page.locator("#showcase [role=tabpanel][data-state=active]");
        await checkImage(panel, `${index} historic`, file, language);
        if (index === "09" && !(await panel.locator(`a[href*='v014-update-${language}.webp']`).count())) problems.push(`${language} update offer image link missing`);
      }
    }
    await page.getByRole("button", { name: "Light" }).click();
    const info = await page.evaluate(() => ({
      title: document.title,
      width: innerWidth,
      overflow: document.documentElement.scrollWidth > innerWidth,
      brokenImages: [...document.images].filter(i => i.complete && !i.naturalWidth).map(i => i.src),
      displayFont: getComputedStyle(document.querySelector("h1")).fontFamily,
      loadedFonts: [...document.fonts].filter(f => f.status === "loaded").map(f => f.family),
      downloadable: [...document.querySelectorAll("a")].some(a => a.href.includes("NovyDeck-0.14.0-Setup.exe")),
    }));
    if (info.overflow) problems.push("Horizontal overflow");
    if (info.brokenImages.length) problems.push(`Broken images: ${info.brokenImages.join(", ")}`);
    if (!info.loadedFonts.length) problems.push("No font was loaded");
    if (!info.downloadable) problems.push("Download link missing");
    report.push({ name, ...info, problems });
  } catch (error) { problems.push(`Test: ${error.message}`); report.push({ name, problems }); }
  failed ||= problems.length > 0;
  await page.close();
}
await browser.close();
console.log(JSON.stringify(report, null, 2));
if (failed) process.exitCode = 1;
