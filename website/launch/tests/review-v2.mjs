import { chromium } from 'playwright';
import { mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
const url = process.argv[2] || 'http://127.0.0.1:8766/';
const shots = '/root/.hermes/novy-bench/site-v2-shots';
await mkdir(shots, { recursive: true });
const browser = await chromium.launch({ headless: true, args: ['--enable-unsafe-swiftshader', '--use-angle=swiftshader'] });
let failed = false;
for (const width of [1440,390]) {
 const page = await browser.newPage({ viewport: { width, height: 900 }, reducedMotion: 'reduce' });
 const errors=[]; page.on('pageerror',e=>errors.push(e.message)); page.on('console',m=>{if(m.type()==='error')errors.push(m.text())});
 const response=await page.goto(url,{waitUntil:'networkidle',timeout:45000});
 await page.screenshot({path:`${shots}/${width}-top.png`,animations:'disabled'});
 for(const section of ['story','features','compatibility']) { await page.locator(`#${section}`).scrollIntoViewIfNeeded(); await page.screenshot({path:`${shots}/${width}-${section}.png`,animations:'disabled',timeout:120000}); }
 await page.screenshot({path:`${shots}/${width}-full.png`,fullPage:true,animations:'disabled',timeout:120000});
 const warm=await page.evaluate(()=>{
  const selectors=['.button-primary','.header-cta','.eyebrow','.chapter-number','.bento-copy span','.bento-card','.story-visual','.compatibility-section','.spec-strip','.stage-rings'];
  const result=[];
  const warmColor=(str)=>{for(const match of str.matchAll(/(?:rgba?\()\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/g)){const [r,g,b]=match.slice(1,4).map(Number);if(r>110&&r>g*1.15&&g>b*1.2) return true;}return false;};
  for(const selector of selectors) for(const el of document.querySelectorAll(selector)) {const css=getComputedStyle(el); for(const prop of ['color','backgroundColor','borderTopColor','borderBottomColor','backgroundImage']) {const value=css[prop]; if(warmColor(value)) result.push({selector,prop,value});}}
  return result;
 });
 const result={url,width,http:response.status(),errors,warm,overflow:await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),canvas:await page.locator('.stage-deck canvas').count()};
 console.log(JSON.stringify(result)); failed ||= result.http!==200||errors.length>0||warm.length>0||result.overflow||(width===1440&&!result.canvas);
 await page.close();
}
const page=await browser.newPage({ viewport:{width:1440,height:900} });
const errors=[]; page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text())});
await page.goto(url,{waitUntil:'networkidle',timeout:45000});
const canvas=page.locator('.stage-deck canvas'); await canvas.waitFor({timeout:15000});
const box=await canvas.boundingBox();
const hash=async()=>createHash('sha256').update(await canvas.screenshot()).digest('hex');
const frames=[]; for(const [i,scroll] of [0,200,450,650].entries()){await page.evaluate(y=>scrollTo(0,y),scroll);await page.waitForTimeout(600);await page.screenshot({path:`${shots}/hero-scroll-${i}.png`});frames.push(await hash());}
await page.waitForTimeout(850);const animated=(await hash())!==frames.at(-1);
await page.mouse.move(box.x+box.width*.7,box.y+box.height*.4);await page.waitForTimeout(400);const pointer=(await hash())!==frames.at(-1);
console.log(JSON.stringify({url,canvasBox:box,distinctFrames:new Set(frames).size,animated,pointer,errors}));
failed ||= !animated||!pointer||new Set(frames).size<2||errors.length>0;
await page.close();await browser.close();if(failed)process.exitCode=1;
