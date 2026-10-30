import React from 'react';
import {Audio} from '@remotion/media';
import {loadFont} from '@remotion/fonts';
import {AbsoluteFill, Easing, Img, interpolate, spring, staticFile, Sequence, useCurrentFrame, useVideoConfig, delayRender, continueRender, cancelRender} from 'remotion';
import {Deck3D} from './Deck3D';
import {NovyWindow,t} from './ui/NovyUI';

const fontWait=delayRender('Polices NovyDeck preview');
Promise.all([
 loadFont({family:'Novy Inter',url:staticFile('fonts/Inter-Regular.otf'),weight:'400'}),
 loadFont({family:'Novy Inter',url:staticFile('fonts/Inter-SemiBold.otf'),weight:'600'}),
 loadFont({family:'Novy Inter',url:staticFile('fonts/InterDisplay-Bold.otf'),weight:'700'}),
]).then(()=>continueRender(fontWait)).catch(cancelRender);
const clamp={extrapolateLeft:'clamp',extrapolateRight:'clamp'} as const;
const ease={...clamp,easing:Easing.bezier(.16,1,.3,1)};
const Words:React.FC<{text:string;frame:number;start:number;vertical:boolean}> = ({text,frame,start,vertical}) => <div style={{display:'flex',flexWrap:'wrap',gap:vertical?15:22,justifyContent:'center'}}>{text.split(' ').map((word,i)=>{const s=spring({frame:frame-start-i*8,fps:30,config:{damping:13,stiffness:110}});return <span key={i} style={{display:'inline-block',opacity:interpolate(frame,[start+i*8,start+i*8+8],[0,1],clamp),transform:`translateY(${(1-s)*65}px) rotate(${(1-s)*-5}deg)`,color:i===1?t.blue:'#fff',textShadow:'0 9px 30px #0008'}}>{word}</span>;})}</div>;

export const MotionPreview:React.FC<{vertical?:boolean}> = ({vertical=false}) => {
 const f=useCurrentFrame(); const {width,height}=useVideoConfig();
 const transform=interpolate(f,[120,169],[0,1],ease);
 const title2=interpolate(f,[170,194,325,355],[0,1,1,0],clamp);
 const introTitle=interpolate(f,[35,60,118,154],[0,1,1,0],clamp);
 const winScale=vertical?1:1;
 const windowY=vertical?255:118;
 const windowX=vertical?(width-940)/2:(width-1450)/2;
 return <AbsoluteFill style={{background:'radial-gradient(ellipse at 52% 33%,#172d52 0%,#0b1323 58%,#060a12 100%)',fontFamily:'Novy Inter,Inter,sans-serif',color:'white',overflow:'hidden'}}>
  <div style={{position:'absolute',inset:vertical?30:28,border:`1px solid #3478f62a`,borderRadius:28,pointerEvents:'none'}}/>
  <div style={{position:'absolute',left:vertical?58:80,top:vertical?63:52,fontWeight:700,letterSpacing:4,fontSize:vertical?27:21,zIndex:9}}>NOVY<span style={{color:t.blue}}>DECK</span> <span style={{color:'#8293ae',fontWeight:400,fontSize:16}}>0.18.1</span></div>
  <div style={{position:'absolute',right:vertical?60:85,top:vertical?69:60,fontSize:15,color:'#a9b4c5',letterSpacing:2}}>FIFINE D6 · 15 TOUCHES LCD</div>
  {f<175 && <div style={{position:'absolute',inset:0,opacity:interpolate(f,[0,17,124,170],[0,1,1,0],clamp),transform:`translateY(${transform*85}px) scale(${interpolate(transform,[0,1],[vertical?.92:.94,.8])})`}}><Deck3D offset={18} zoom={vertical?.98:1.13}/></div>}
  {f<125 && <div style={{position:'absolute',left:'50%',top:vertical?295:132,transform:`translateX(-50%) scale(${interpolate(f,[0,105,124],[1,1,.13],clamp)})`,width:vertical?280:186,height:vertical?280:186,display:'grid',gridTemplateColumns:'repeat(5,1fr)',gap:3,zIndex:3,opacity:interpolate(f,[0,12,95,124],[1,1,1,0],clamp)}}>{Array.from({length:15},(_,i)=>{const appear=spring({frame:f-4-i*3,fps:30,config:{damping:13}});return <div key={i} style={{overflow:'hidden',borderRadius:4,opacity:Math.max(0,appear),transform:`translate(${(1-appear)*(i%2?60:-60)}px,${(1-appear)*-50}px)`,boxShadow:'0 5px 17px #3478f64d'}}><Img src={staticFile('logo.png')} style={{width:vertical?280:186,height:vertical?280:186,maxWidth:'none',transform:`translate(${-(i%5)*(vertical?56:37.2)}px,${-Math.floor(i/5)*(vertical?93.333:62)}px)`}}/></div>;})}</div>}
  {f>=125 && <div style={{position:'absolute',left:windowX,top:windowY,transform:`translateY(${interpolate(transform,[0,1],[180,0])}px) scale(${interpolate(transform,[0,1],[.72,winScale])})`,opacity:interpolate(f,[124,157],[0,1],clamp),transformOrigin:'top center',zIndex:4}}><NovyWindow frame={f} vertical={vertical}/></div>}
  <div style={{position:'absolute',left:vertical?60:95,right:vertical?60:95,top:vertical?1140:890,textAlign:'center',fontWeight:700,letterSpacing:-3,lineHeight:1.05,fontSize:vertical?86:76,opacity:introTitle,transform:`translateY(${(1-introTitle)*30}px)`,zIndex:8}}><Words frame={f} start={42} vertical={vertical} text="Ton deck. Ton univers."/></div>
  <div style={{position:'absolute',left:vertical?60:95,right:vertical?60:95,top:vertical?1510:890,textAlign:'center',fontWeight:700,letterSpacing:-3,lineHeight:1.05,fontSize:vertical?74:76,opacity:title2,transform:`translateY(${(1-title2)*30}px)`,zIndex:8}}><Words frame={f} start={173} vertical={vertical} text="Attrape. Glisse. C'est posé."/></div>
  {vertical && <div style={{position:'absolute',left:80,right:80,bottom:106,textAlign:'center',fontSize:27,color:'#a9b4c5',letterSpacing:2}}>LE DECK, À TA FAÇON.</div>}
  <Audio src={staticFile('soundtrack.wav')} volume={interpolate(f,[0,30,330,360],[0,.48,.48,0],clamp)} durationInFrames={360}/>
  <Sequence from={202} durationInFrames={6} layout="none"><Audio src={staticFile('click.wav')} volume={.8}/></Sequence>
  <Sequence from={300} durationInFrames={14} layout="none"><Audio src={staticFile('drop.wav')} volume={.8}/></Sequence>
 </AbsoluteFill>;
};
export const MotionPreviewVertical:React.FC = () => <MotionPreview vertical/>;
