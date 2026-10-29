import { chromium } from "playwright";
const target = process.argv[2] || "http://127.0.0.1:8767/NovyDeck/";
const browser = await chromium.launch({ headless: true, args: ["--enable-unsafe-swiftshader"] });
const results = [];
let failed = false;
for (const width of [320, 390, 768, 1024, 1440]) {
  const page = await browser.newPage({ viewport: { width, height: 900 } });
  const problems = [];
  page.on("pageerror", error => problems.push(`JS ${error.message}`));
  page.on("response", response => { if (response.status() >= 400) problems.push(`HTTP ${response.status()} ${response.url()}`); });
  try {
    const response = await page.goto(target, { waitUntil: "networkidle", timeout: 45_000 });
    if (response?.status() !== 200) problems.push(`Document HTTP ${response?.status()}`);
    await page.locator("h1").waitFor();
    for (const section of ["story", "features", "compatibility", "faq", "download"]) {
      await page.locator(`#${section}`).scrollIntoViewIfNeeded();
      if (await page.evaluate(() => document.documentElement.scrollWidth > innerWidth)) problems.push(`overflow at ${section}`);
    }
    for (const image of ["v0161-onoff-cards-fr", "v0161-plugins-tab-fr"]) {
      const img = page.locator(`.story-visual img[src*='${image}']`);
      await img.scrollIntoViewIfNeeded();
      await img.evaluate(node => (node instanceof HTMLImageElement ? node.decode() : Promise.reject("not an image")));
    }
    if (width < 900 && await page.locator("canvas").count()) problems.push("WebGL on mobile/tablet");
    if (width >= 900) { await page.locator("#top").scrollIntoViewIfNeeded(); await page.locator(".stage-deck canvas").waitFor({ timeout: 15_000 }); }
    await page.getByRole("button", { name: "Passer en anglais" }).click();
    if (await page.locator("html").getAttribute("lang") !== "en") problems.push("EN switch failed");
    if (!(await page.locator("a[href*='v0.16.1/NovyDeck-0.16.1-Setup.exe']").count())) problems.push("installer missing");
    results.push({ width, problems });
  } catch (error) { problems.push(`Test ${error.message}`); results.push({ width, problems }); }
  failed ||= problems.length > 0;
  await page.close();
}
await browser.close();
console.log(JSON.stringify(results, null, 2));
if (failed) process.exitCode = 1;
