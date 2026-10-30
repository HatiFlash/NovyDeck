import React from 'react';
import {Img, interpolate, spring, staticFile} from 'remotion';

// Palette exacte du thème « Bleu nuit » (src/theme.rs), accent unique bleu.
export const t = {blue:'#3478f6', bg:'#0b0f17', surface:'#111723', card:'#18202e', sunken:'#090e16', overlay:'#1e2737', line:'#222b3a', border:'#455266', ink:'#eff3f9', muted:'#a9b4c5', secondary:'#c7d1df', radius:[6,10,14] as const};
const easing = {extrapolateLeft:'clamp',extrapolateRight:'clamp'} as const;
const keyPath = (i:number) => staticFile(`keys/key-${String(i).padStart(2,'0')}.png`);
const card:React.CSSProperties = {background:t.card,border:`1px solid ${t.line}`,borderRadius:10};

export const DeckGrid:React.FC<{frame:number; target:number; size?:number; gap?:number}> = ({frame,target,size=84,gap=12}) => <div style={{display:'grid',gridTemplateColumns:`repeat(5, ${size}px)`,gap,padding:17,borderRadius:26,background:'linear-gradient(140deg,#292f38,#11151c)',boxShadow:'0 20px 42px #0009, inset 0 2px 3px #6c75824a',border:'2px solid #394351'}}>{Array.from({length:15},(_,i)=>{
 const light=interpolate(frame,[14+i*5,25+i*5],[0,1],easing);
 const hover=interpolate(frame,[239,263,279,297],[0,0,1,0],easing);
 const assigned=i===target && frame>=300;
 const bounce=i===target?spring({frame:frame-299,fps:30,config:{damping:10,stiffness:160}}):1;
 return <div key={i} style={{width:size,height:size,borderRadius:10,padding:5,boxSizing:'border-box',background: i===target?`linear-gradient(135deg,#253752,#161b23)`:'#20242d',boxShadow:i===target?`0 0 ${hover*26}px #3478f6aa, inset 0 0 0 ${1+hover*2}px ${t.blue}`:'inset 0 2px 2px #ffffff24,0 3px 4px #0009',transform:`scale(${i===target&&frame>=299?1+(1-bounce)*.18:1})`}}>
  <div style={{width:'100%',height:'100%',borderRadius:6,background:'#0a101a',overflow:'hidden',boxShadow:`inset 0 0 14px #000,0 0 ${light*8}px #3478f660`}}><Img src={assigned?keyPath(4):keyPath(i)} style={{width:'100%',height:'100%',objectFit:'cover',opacity:.12+light*.88,transform:`scale(${1.12-light*.12})`}}/></div>
 </div>;})}</div>;

export const Inspector:React.FC<{frame:number; compact?:boolean}> = ({frame,compact=false}) => {
 const assigned=frame>=300;
 const appearing=spring({frame:frame-300,fps:30,config:{damping:15,stiffness:160}});
 return <section style={{...card,padding:compact?16:18,flex:1,minHeight:0,overflow:'hidden'}}>
  <div style={{display:'flex',alignItems:'center',gap:12,borderBottom:`1px solid ${t.line}`,paddingBottom:13}}><div style={{background:t.sunken,borderRadius:8,padding:6,width:38,height:38}}>{assigned&&<Img src={keyPath(4)} style={{width:'100%',height:'100%',transform:`scale(${Math.max(.1,appearing)})`}}/>}</div><div><div style={{fontWeight:700,fontSize:18}}>Touche 8</div><div style={{fontSize:12,color:t.muted}}>Inspecteur de touche</div></div></div>
  <div style={{display:'flex',gap:8,margin:'13px 0'}}>{['Action','Apparence','Avancé'].map((x,i)=><div key={x} style={{fontSize:12,background:i===0?t.blue:t.sunken,borderRadius:7,padding:'9px 14px',color:i===0?'white':t.muted}}>{x}</div>)}</div>
  <div style={{color:t.secondary,fontWeight:600,fontSize:14}}>Actions · {assigned?'1 étape':'0 étape'}</div>
  <div style={{...card,marginTop:10,padding:12,fontSize:13,color:assigned?t.ink:t.muted,opacity:assigned?Math.max(.2,appearing):1}}>{assigned?'Lancer une application':'Glisse une action ou un raccourci sur cette touche.'}</div>
 </section>;
};

export const Library:React.FC<{frame:number; compact?:boolean}> = ({frame,compact=false}) => <aside style={{background:t.surface,borderLeft:`1px solid ${t.line}`,width:compact?'100%':290,padding:compact?15:18,boxSizing:'border-box',flexShrink:0,overflow:'hidden'}}>
 <div style={{fontSize:compact?18:17,fontWeight:700,whiteSpace:'nowrap'}}>Bibliothèque d'actions</div>
 <div style={{...card,background:t.sunken,color:t.muted,fontSize:12,padding:12,marginTop:13}}>⌕ &nbsp; Rechercher une action…</div>
 <div style={{display:'flex',gap:4,margin:'12px 0'}}>{['Actions','Modèles','Enregistrés','Plugins'].map((x,i)=><div key={x} style={{flex:1,textAlign:'center',borderRadius:8,background:t.sunken,border:i===0?`1px solid ${t.blue}`:'1px solid transparent',color:i===0?t.ink:t.muted,fontSize:compact?12:10,padding:'9px 1px'}}>{x}</div>)}</div>
 <div style={{color:t.muted,fontSize:11,lineHeight:1.4,marginBottom:12}}>Glisse une action sur une touche. Elle s’ajoute à la fin de sa macro.</div>
 {['Lancer une application','Raccourci clavier','Ouvrir un site','Contrôle multimédia'].map((x,i)=><div key={x} style={{...card,display:'flex',alignItems:'center',height:compact?60:65,marginBottom:7,padding:'0 10px',opacity:frame>300&&i===0?.55:1}}><div style={{width:36,height:36,borderRadius:8,display:'grid',placeItems:'center',background:'#263752',color:t.blue,fontSize:21}}>{['▣','⌘','↗','▶'][i]}</div><div style={{marginLeft:9,minWidth:0}}><div style={{fontSize:compact?14:12,fontWeight:600,whiteSpace:'nowrap'}}>{x}</div><div style={{fontSize:10,color:t.muted}}>Action · Glisser ou cliquer</div></div></div>)}
 </aside>;

export const Mouse:React.FC<{x:number;y:number;pressed:boolean}> = ({x,y,pressed}) => <svg style={{position:'absolute',left:x,top:y,width:35,height:42,filter:'drop-shadow(2px 4px 3px #000a)',zIndex:20,transform:`scale(${pressed?.88:1})`}} viewBox="0 0 35 42"><path d="M3 2 L3 33 L11 26 L17 39 L23 36 L17 23 L29 22 Z" fill="white" stroke="#111723" strokeWidth="2.5" strokeLinejoin="round"/></svg>;

export const NovyWindow:React.FC<{frame:number;vertical:boolean}> = ({frame,vertical}) => {
 const scale=vertical?1.4:1;
 const w=vertical?940:1450,h=vertical?1120:748;
 const drag=interpolate(frame,[175,203,244,275,299],[0,0,.25,.79,1],easing);
 const x=vertical?interpolate(drag,[0,.23,1],[710,690,445],easing):interpolate(drag,[0,.23,1],[1300,1270,560],easing);
 const y=vertical?interpolate(drag,[0,.23,1],[1040,1028,420],easing):interpolate(drag,[0,.23,1],[316,309,355],easing);
 const active=frame>=202&&frame<300;
 return <div style={{width:w,height:h,background:t.bg,color:t.ink,border:`1px solid ${t.border}`,borderRadius:16,boxShadow:'0 38px 100px #000b,0 0 90px #3478f626',overflow:'hidden',fontFamily:'Novy Inter,Inter,sans-serif',position:'relative',boxSizing:'border-box'}}>
  <div style={{height:54,background:t.surface,borderBottom:`1px solid ${t.line}`,display:'flex',alignItems:'center',justifyContent:'space-between',padding:'0 20px'}}><div style={{display:'flex',gap:12,alignItems:'center'}}><Img src={staticFile('logo.png')} style={{width:31,height:31,borderRadius:7}}/><strong style={{fontSize:18}}>NovyDeck</strong><span style={{fontSize:12,color:t.muted}}>0.18.1</span></div><span style={{color:t.muted,letterSpacing:9}}>─ □ ×</span></div>
  <div style={{height:52,background:t.surface,borderBottom:`1px solid ${t.line}`,display:'flex',alignItems:'center',gap:8,padding:'0 20px'}}>{['Deck','Atelier','Plugins','Réglages'].map((x,i)=><div key={x} style={{padding:'10px 17px',borderRadius:8,background:i===0?t.overlay:'transparent',color:i===0?t.ink:t.muted,borderBottom:i===0?`2px solid ${t.blue}`:'2px solid transparent',fontSize:14}}>{x}</div>)}<span style={{marginLeft:'auto',fontSize:12,color:t.muted}}>Touches : actives</span></div>
  {vertical?<div style={{height:h-106,display:'flex',flexDirection:'column'}}><div style={{flex:1,padding:20,minHeight:0}}><div style={{...card,padding:18,height:'100%',boxSizing:'border-box'}}><div style={{display:'flex',justifyContent:'space-between',fontSize:18,fontWeight:700}}>Mon profil <span style={{fontSize:12,fontWeight:400,color:t.muted}}>Page 1 / 1</span></div><p style={{fontSize:12,color:t.muted}}>Glisse un raccourci sur une touche, puis personnalise ses actions dans l’inspecteur.</p><div style={{display:'flex',justifyContent:'center',marginTop:30,transform:`scale(${scale})`,transformOrigin:'top center'}}><DeckGrid frame={frame} target={7} size={70} gap={9}/></div></div></div><div style={{display:'flex',height:390,gap:10,padding:'0 20px 20px'}}><Inspector frame={frame} compact/><div style={{width:395,overflow:'hidden',borderRadius:10}}><Library frame={frame} compact/></div></div></div>:
  <div style={{height:h-106,display:'flex'}}><div style={{flex:1,display:'flex',flexDirection:'column',padding:15,gap:12,minWidth:0}}><div style={{...card,flex:1,padding:16,overflow:'hidden'}}><div style={{display:'flex',justifyContent:'space-between',fontSize:20,fontWeight:700}}>Mon profil <span style={{fontSize:12,fontWeight:400,color:t.muted}}>Page 1 / 1</span></div><p style={{fontSize:12,color:t.muted}}>Glisse un raccourci sur une touche, puis personnalise ses actions dans l’inspecteur.</p><div style={{display:'flex',justifyContent:'center',marginTop:7}}><DeckGrid frame={frame} target={7} size={83}/></div></div><div style={{height:194,display:'flex'}}><Inspector frame={frame}/></div></div><Library frame={frame}/></div>}
  {frame>=160 && <Mouse x={x} y={y} pressed={active}/>}
  {active && <div style={{...card,position:'absolute',left:x+19,top:y-21,width:vertical?220:210,height:56,display:'flex',alignItems:'center',padding:8,boxShadow:'0 20px 40px #000b,0 0 24px #3478f680',zIndex:19,transform:`rotate(${interpolate(drag,[0,1],[-6,5],easing)}deg) scale(${interpolate(frame,[202,217],[.7,1],easing)})`,pointerEvents:'none'}}><div style={{width:36,height:36,borderRadius:7,background:'#263752',display:'grid',placeItems:'center',color:t.blue}}>▣</div><span style={{fontSize:12,marginLeft:8}}>Lancer une application</span></div>}
 </div>;
};
