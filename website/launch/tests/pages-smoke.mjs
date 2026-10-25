import { chromium } from "playwright";

const target = process.argv[2] || "http://127.0.0.1:8767/NovyDeck/";
const browser = await chromium.launch({ headless: true, args: ["--enable-unsafe-swiftshader"] });
const report = [];
let failed = false;
for (const [name, width, height] of [["desktop", 1440, 900], ["mobile", 390, 844]]) {
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
    await page.getByRole("tab", { name: /02.*États ON/ }).click();
    await page.getByRole("button", { name: "Clair" }).click();
    await page.getByRole("button", { name: /Switch to English/ }).click();
    if (await page.locator("html").getAttribute("lang") !== "en") problems.push("Language toggle failed");
    const info = await page.evaluate(() => ({
      title: document.title,
      width: innerWidth,
      overflow: document.documentElement.scrollWidth > innerWidth,
      brokenImages: [...document.images].filter(i => i.complete && !i.naturalWidth).map(i => i.src),
      displayFont: getComputedStyle(document.querySelector("h1")).fontFamily,
      loadedFonts: [...document.fonts].filter(f => f.status === "loaded").map(f => f.family),
      downloadable: [...document.querySelectorAll("a")].some(a => a.href.includes("NovyDeck-0.11.1-Setup.exe")),
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
