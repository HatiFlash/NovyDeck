import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

const output = (name: string) => `test-results/novy-launch-${name}.png`;

test("desktop: sections, media, interactions, download", async ({ page }) => {
  const errors: string[] = [];
  const failed: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  page.on("response", response => { if (response.status() >= 400) failed.push(`${response.status()} ${response.url()}`); });
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/", { waitUntil: "networkidle" });
  await expect(page.getByRole("heading", { level: 1 })).toContainText("Votre deck.");
  await expect(page.locator("main section")).toHaveCount(10);
  await page.screenshot({ path: output("hero-desktop") });
  await page.locator("#product-story").scrollIntoViewIfNeeded();
  await expect(page.locator(".scene-canvas canvas")).toBeVisible();
  await page.screenshot({ path: output("story-desktop") });
  await page.locator("#showcase").scrollIntoViewIfNeeded();
  await page.screenshot({ path: output("showcase-desktop") });
  await expect(page.getByRole("tab")).toHaveCount(10);
  for (const [index, label, file] of [["01", "Touches 0.15", "main"], ["02", "Apparence 0.15", "appearance"], ["03", "Assistant 0.15", "assistant"], ["04", "Précision plugin 0.15", "question"]]) {
    await page.getByRole("tab", { name: new RegExp(`${index}.*${label}`) }).click();
    const panel = page.locator("#showcase [role=tabpanel][data-state=active]");
    await expect(panel.locator(".micro")).toHaveText(`${index} / 10`);
    await expect(panel.locator(`a[href*='v015-${file}-fr.webp']`)).toHaveAttribute("aria-label", /Agrandir la capture/);
    await expect(panel.locator("img")).toBeVisible();
    await panel.locator("img").scrollIntoViewIfNeeded();
    await expect.poll(() => panel.locator("img").evaluate(img => (img as HTMLImageElement).naturalWidth), { timeout: 10_000 }).toBeGreaterThan(0);
  }
  await expect(page.locator("#showcase [role=tabpanel][data-state=active] img")).toHaveAttribute("alt", /fixture plugin SoundPad inerte/);
  await page.getByRole("tab", { name: /06.*États ON/ }).click();
  await expect(page.getByRole("heading", { name: "Le bon état, au bon moment." })).toBeVisible();
  await page.getByRole("tab", { name: /08.*Atelier 0\.14/ }).click();
  await expect(page.getByRole("img", { name: /Capture française de l’Atelier 0\.14/ })).toBeVisible();
  await page.screenshot({ path: output("workshop-014-desktop") });
  await page.getByRole("tab", { name: /09.*Quoi de neuf/ }).click();
  await expect(page.getByRole("link", { name: /offre de mise à jour 0\.13/ })).toHaveAttribute("href", /v014-update-fr\.webp/);
  await page.screenshot({ path: output("whats-new-014-desktop") });
  await page.getByRole("button", { name: "Clair" }).click();
  await expect(page.getByRole("img", { name: "Interface NovyDeck en thème clair" })).toBeVisible();
  await page.locator("#workflow").scrollIntoViewIfNeeded();
  await page.screenshot({ path: output("workflow-desktop") });
  await expect(page.locator(".flow-step")).toHaveCount(5);
  const href = await page.getByRole("link", { name: "Installateur Windows v0.15.0" }).getAttribute("href");
  expect(href).toContain("v0.15.0/NovyDeck-0.15.0-Setup.exe");
  for (const section of ["assistant", "plugins", "integrations", "film", "download"]) {
    await page.locator(`#${section}`).scrollIntoViewIfNeeded();
    await page.waitForTimeout(200);
  }
  const broken = await page.locator("img").evaluateAll(images => images.filter(i => !(i as HTMLImageElement).complete || !(i as HTMLImageElement).naturalWidth).map(i => (i as HTMLImageElement).src));
  expect(broken).toEqual([]);
  expect(errors).toEqual([]);
  expect(failed).toEqual([]);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
});

test("mobile: responsive, navigation and lightweight scene", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/", { waitUntil: "networkidle" });
  await page.screenshot({ path: output("hero-mobile") });
  await expect(page.locator(".scene-canvas canvas")).toHaveCount(0);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.getByRole("button", { name: "Ouvrir le menu" }).click();
  await page.getByRole("navigation", { name: "Navigation principale" }).getByRole("link", { name: "Atelier" }).click();
  await expect(page.locator("#workflow")).toBeInViewport();
  await page.screenshot({ path: output("workflow-mobile") });
  await page.getByRole("tab", { name: /07.*Médias/ }).click();
  await expect(page.getByRole("heading", { name: "Cadrez sans abîmer." })).toBeVisible();
  await page.getByRole("button", { name: "Switch to English" }).click();
  await expect(page.getByRole("heading", { level: 1 })).toContainText("Your deck.");
  await expect(page.locator("html")).toHaveAttribute("lang", "en");
  await expect(page.getByRole("tab")).toHaveCount(10);
  await page.getByRole("tab", { name: /03.*Assistant 0\.15/ }).click();
  await expect(page.locator("#showcase [role=tabpanel][data-state=active] img")).toHaveAttribute("alt", /French example request/);
  await page.getByRole("tab", { name: /06.*ON \/ OFF/ }).click();
  await expect(page.getByRole("heading", { name: "The right state at the right time." })).toBeVisible();
  await page.getByRole("tab", { name: /07.*Media/ }).click();
  await expect(page.getByRole("heading", { name: "Crop without damage." })).toBeVisible();
  await page.getByRole("tab", { name: /09.*What’s new/ }).click();
  await expect(page.getByRole("link", { name: /update offer/ })).toHaveAttribute("href", /v014-update-en\.webp/);
  expect(errors).toEqual([]);
});

test("reduced motion: no 3D, no smooth scrolling, static poster", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/", { waitUntil: "networkidle" });
  await expect(page.locator(".scene-canvas canvas")).toHaveCount(0);
  await expect(page.locator(".device-fallback")).toBeVisible();
  expect(await page.locator("#film video").evaluate(v => (v as HTMLVideoElement).paused)).toBe(true);
  await page.screenshot({ path: output("reduced-motion") });
});

test("intermediate widths and accessible landmarks", async ({ page }) => {
  for (const width of [768, 1024]) {
    await page.setViewportSize({ width, height: 800 });
    await page.goto("/", { waitUntil: "networkidle" });
    for (const selector of ["#top", "#product-story", "#showcase", "#workflow", "#plugins"]) {
      await page.locator(selector).scrollIntoViewIfNeeded();
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `overflow at ${width}px ${selector}`).toBe(true);
    }
  }
  await page.goto("/", { waitUntil: "networkidle" });
  const results = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"]).analyze();
  expect(results.violations.filter(v => ["serious", "critical"].includes(v.impact ?? ""))).toEqual([]);
});

test("small phone: no overflow across chapters", async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 690 });
  await page.goto("/", { waitUntil: "domcontentloaded" });
  for (const selector of ["#top", "#product-story", "#showcase", "#workflow", "#assistant", "#download"]) {
    await page.locator(selector).scrollIntoViewIfNeeded();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `overflow at ${selector}`).toBe(true);
  }
  await page.screenshot({ path: output("phone-320") });
});
