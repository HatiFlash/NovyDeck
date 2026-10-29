import { chromium } from 'playwright';
import { createHash } from 'node:crypto';
const browser = await chromium.launch({headless:true,args:['--enable-unsafe-swiftshader','--use-angle=swiftshader']});
for (const url of ['https://hatiflash.github.io/NovyDeck/','http://127.0.0.1:8766/']) for (const reduce of [false,true]) {
 const page = await browser.newPage({viewport:{width:1440,height:900},reducedMotion:reduce?'reduce':'no-preference'});
 const errors=[]; page.on('pageerror',e=>errors.push(e.message)); page.on('console',m=>{if(m.type()==='error')errors.push(m.text())});
 try {await page.goto(url,{waitUntil:'domcontentloaded',timeout:20000}); await page.waitForTimeout(3000); const canvas=page.locator('.stage-deck canvas'); const dimensions=await page.locator('.stage-deck').evaluate(e=>({width:e.getBoundingClientRect().width,height:e.getBoundingClientRect().height,scroll:document.documentElement.scrollWidth})); const grab=async()=>createHash('sha256').update(await page.screenshot({clip:{x:750,y:180,width:550,height:450},animations:'disabled'})).digest('hex'); const before=await grab(); await page.waitForTimeout(1100); const after=await grab(); await page.mouse.move(1120,450); await page.waitForTimeout(400); const pointer=await grab(); console.log(JSON.stringify({url,reduce,canvas:await canvas.count(),dimensions,animation:before!==after,pointer:after!==pointer,errors}));}catch(e){console.log(JSON.stringify({url,reduce,error:String(e),errors}))} await page.close();
}
await browser.close();
