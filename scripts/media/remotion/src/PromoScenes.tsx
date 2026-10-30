import React from 'react';
import {loadFont} from '@remotion/fonts';
import {AbsoluteFill, Easing, Img, interpolate, staticFile, useCurrentFrame, spring, useVideoConfig, delayRender, continueRender, cancelRender} from 'remotion';
import {Deck3D} from './Deck3D';

const fontWait = delayRender('Polices NovyDeck');
Promise.all([
  loadFont({family:'Novy Inter',url:staticFile('fonts/Inter-Regular.otf'),weight:'400'}),
  loadFont({family:'Novy Inter',url:staticFile('fonts/Inter-Medium.otf'),weight:'500'}),
  loadFont({family:'Novy Inter',url:staticFile('fonts/Inter-SemiBold.otf'),weight:'600'}),
  loadFont({family:'Novy Inter',url:staticFile('fonts/InterDisplay-Bold.otf'),weight:'700'}),
]).then(() => continueRender(fontWait)).catch(cancelRender);
const clamp = {extrapolateLeft:'clamp',extrapolateRight:'clamp'} as const;
const ease = {...clamp,easing:Easing.bezier(.17,1,.32,1)};
const blue = '#68d9f2';
const stage:React.CSSProperties = {background:'radial-gradient(ellipse at 70% 35%,#173650 0%,#0b1a2d 48%,#050b17 90%)',color:'#eaf5ff',fontFamily:'Novy Inter, sans-serif',overflow:'hidden'};
const rise = (f:number, start:number) => interpolate(f,[start,start+24],[60,0],ease);
const visible = (f:number,start:number) => interpolate(f,[start,start+18],[0,1],ease);
const Base:React.FC<{children:React.ReactNode; number?:string}> = ({children,number}) => <AbsoluteFill style={stage}>
  <div style={{position:'absolute',inset:35,border:'1px solid #67cce11b',borderRadius:32,pointerEvents:'none'}} />
  <div style={{position:'absolute',left:80,right:80,top:72,height:2,background:'linear-gradient(90deg,#65d6ed88,transparent 78%)'}} />
  <div style={{position:'absolute',left:82,bottom:62,fontSize:23,letterSpacing:4,color:'#79a8c0'}}>NOVYDECK <span style={{color:blue}}> / 0.18.1</span></div>
  {number && <div style={{position:'absolute',right:86,bottom:62,fontSize:23,letterSpacing:3,color:'#79a8c0'}}>{number} / 09</div>}
  {children}
</AbsoluteFill>;
const Heading:React.FC<{tag:string; title:string; detail:string; number:string}> = ({tag,title,detail,number}) => {
 const f=useCurrentFrame();
 return <div style={{position:'absolute',left:83,top:195,width:550,zIndex:5,pointerEvents:'none'}}>
  <div style={{fontSize:24,fontWeight:600,letterSpacing:4,color:blue,opacity:visible(f,4),transform:`translateY(${rise(f,4)}px)`}}>{number} · {tag.toUpperCase()}</div>
  <div style={{fontSize:76,fontWeight:700,letterSpacing:-3,lineHeight:1.08,whiteSpace:'pre-line',marginTop:36,opacity:visible(f,13),transform:`translateY(${rise(f,13)}px)`,textShadow:'0 14px 34px #061425'}}>{title}</div>
  <div style={{fontSize:31,lineHeight:1.35,color:'#b8d6e4',marginTop:38,maxWidth:495,opacity:visible(f,29),transform:`translateY(${rise(f,29)}px)`}}>{detail}</div>
  <div style={{height:4,width:interpolate(f,[28,75],[0,150],clamp),background:blue,marginTop:46,borderRadius:9,boxShadow:'0 0 22px #54ccea88'}} />
 </div>;
};
type ScreenProps={shot:string; secondary?:string; flip?:'drag'|'tabs'|'atelier'|'settings'; count?:number; zoom?:number; focus?:string};
const Screen:React.FC<ScreenProps> = ({shot,secondary,flip,count=14,zoom=1.04,focus='50% 50%'}) => {
 const f=useCurrentFrame();const {fps}=useVideoConfig();
 const trigger=35;
 const image=flip && f>=trigger ? `ui018r-${flip}-${String(Math.min(count-1,Math.max(0,f-trigger))).padStart(2,'0')}` : shot;
 const lift=spring({frame:f-5,fps,config:{damping:200,mass:1,stiffness:110}});
 const pulse=interpolate(f,[46,105],[0,1],clamp);
 return <>
   {secondary && <div style={{position:'absolute',right:105,top:96,width:1070,height:655,borderRadius:24,border:'1px solid #87def444',background:'#0a1825',transform:`perspective(1800px) rotateY(-11deg) rotateZ(3deg) translate(${rise(f,0)*.5}px,${rise(f,0)*.5}px)`,opacity:visible(f,0)*.5,overflow:'hidden'}}><Img src={staticFile(`shots/${secondary}.png`)} style={{width:'100%',height:'100%',objectFit:'cover'}} /></div>}
   <div style={{position:'absolute',right:82,top:181,width:1220,height:730,padding:9,background:'linear-gradient(135deg,#63d2e9,#1b3449 15%,#09121e 85%,#49b8d9)',border:'1px solid #80d9ee77',borderRadius:23,boxShadow:'0 42px 110px #000b,0 0 75px #40cde432',opacity:visible(f,5),transform:`perspective(1900px) translateY(${(1-lift)*94}px) rotateY(${interpolate(f,[0,95],[-8,-2],ease)}deg) rotateX(${interpolate(f,[0,110],[4,0],ease)}deg)`}}>
    <div style={{position:'relative',height:'100%',width:'100%',borderRadius:14,overflow:'hidden',background:'#091626'}}>
      <Img src={staticFile(`shots/${image}.png`)} style={{width:'100%',height:'100%',objectFit:'contain',objectPosition:focus,transform:`scale(${interpolate(pulse,[0,1],[1,zoom],clamp)})`,transformOrigin:focus}} />
      <div style={{position:'absolute',inset:0,boxShadow:'inset 0 0 95px #040e2399',pointerEvents:'none'}} />
    </div>
   </div>
 </>;
};
const Feature:React.FC<{tag:string;title:string;detail:string;number:string;screen:ScreenProps}> = ({tag,title,detail,number,screen}) => <Base number={number}><Screen {...screen}/><Heading tag={tag} title={title} detail={detail} number={number}/></Base>;
export const Intro:React.FC = () => {const f=useCurrentFrame();return <Base>
 <Deck3D offset={24} zoom={1.1}/>
 <div style={{position:'absolute',left:100,top:136,display:'flex',alignItems:'center',gap:30,opacity:visible(f,10),transform:`translateY(${rise(f,10)}px)`}}><Img src={staticFile('logo.png')} style={{height:140,width:140,borderRadius:28,boxShadow:'0 12px 55px #27b7ef55'}}/><div style={{fontSize:34,letterSpacing:5,color:blue}}>TON DECK. TON UNIVERS.</div></div>
 <div style={{position:'absolute',left:100,bottom:160,fontSize:137,fontWeight:700,letterSpacing:-7,opacity:visible(f,27),transform:`translateY(${rise(f,27)}px)`,textShadow:'0 18px 42px #061425'}}><span style={{color:blue}}>Novy</span>Deck</div>
 <div style={{position:'absolute',left:105,bottom:113,fontSize:31,color:'#c0dbe9',opacity:visible(f,48)}}>Le contrôle prend vie. · Windows · FIFINE D6</div>
 </Base>};
export const Deck:React.FC = () => <Base number="01"><Deck3D offset={18} zoom={1.2}/><Heading tag="Touches LCD" title={'Chaque touche,\nune idée.'} detail="15 écrans sur le FIFINE D6. Profils, pages et actions à portée de main." number="01"/></Base>;
export const Dashboard:React.FC = () => <Feature number="02" tag="Interface" title={'Tout commence\nici.'} detail="Choisis une touche. Compose son apparence et ses actions." screen={{shot:'v017-deck-white-fr',secondary:'v017-activity-collapsed-fr',zoom:1.1,focus:'49% 45%'}}/>;
export const Personalize:React.FC = () => <Feature number="03" tag="Personnalisation" title={'Couleurs. Icônes.\nTa signature.'} detail="Libère l’apparence du deck et de chaque touche." screen={{shot:'v017-deck-color-free-fr',secondary:'v015-appearance-fr',zoom:1.13,focus:'48% 50%'}}/>;
export const Drag:React.FC = () => <Feature number="04" tag="Glisser-déposer" title={'Attrape. Glisse.\nC’est posé.'} detail="La carte suit ton pointeur et plonge dans la touche." screen={{shot:'ui018r-drag-00',flip:'drag',count:14,zoom:1.23,focus:'65% 50%'}}/>;
export const Workshop:React.FC = () => <Feature number="05" tag="Atelier de blocs" title={'Des macros\nqui bougent.'} detail="Assemble et réorganise tes actions, visuellement." screen={{shot:'v016-workshop-cards-fr',secondary:'v016-workshop-library-fr',flip:'atelier',count:14,zoom:1.16,focus:'58% 50%'}}/>;
export const OnOff:React.FC = () => <Feature number="06" tag="Deux états" title={'Un appui.\nDeux visages.'} detail="Actions ON/OFF, icônes dédiées, état local si le plugin ne répond pas." screen={{shot:'v0161-onoff-cards-fr',secondary:'v015-appearance-fr',zoom:1.17,focus:'63% 46%'}}/>;
export const Plugins:React.FC = () => <Feature number="07" tag="Plugins & magasin" title={'Plus loin,\nensemble.'} detail="Installe et retrouve tes plugins depuis le magasin." screen={{shot:'v0161-plugins-tab-fr',secondary:'v016-workshop-library-fr',zoom:1.1}}/>;
export const Assistant:React.FC = () => <Feature number="08" tag="Assistant IA" title={'Décris. Relis.\nCrée.'} detail="L’assistant propose des blocs. Tu gardes le dernier mot." screen={{shot:'v015-assistant-fr',zoom:1.16,focus:'70% 45%'}}/>;
export const Motion:React.FC = () => <Feature number="09" tag="Fluidité 0.18" title={'Tout coule\nde source.'} detail="Onglets glissants, réglages en fondu, animations désactivables." screen={{shot:'ui018r-tabs-00',flip:'tabs',count:13,secondary:'v014-update-fr',zoom:1.08}}/>;
export const Outro:React.FC = () => {const f=useCurrentFrame();const {fps}=useVideoConfig();return <Base>
 <div style={{position:'absolute',left:145,top:190,width:620,height:620,borderRadius:'50%',background:'radial-gradient(circle,#4dcde977,transparent 68%)',filter:'blur(42px)',transform:`scale(${interpolate(f,[0,100],[.5,1.1],clamp)})`}} />
 <Img src={staticFile('logo.png')} style={{position:'absolute',left:175,top:200,width:580,height:580,objectFit:'contain',opacity:visible(f,9),transform:`scale(${spring({frame:f-6,fps,config:{damping:16}})})`}}/>
 <div style={{position:'absolute',left:850,top:290}}><div style={{fontSize:29,letterSpacing:5,color:blue,opacity:visible(f,12)}}>LE DECK, À TA FAÇON.</div><div style={{fontSize:125,letterSpacing:-5,fontWeight:700,marginTop:25,opacity:visible(f,24),transform:`translateY(${rise(f,24)}px)`}}><span style={{color:blue}}>Novy</span>Deck</div><div style={{fontSize:53,color:'#b6d9e6',marginTop:20,opacity:visible(f,51)}}>0.18.1</div><div style={{fontSize:31,color:'#e3f3fa',marginTop:115,opacity:visible(f,76)}}>github.com/HatiFlash/NovyDeck</div></div>
 </Base>};
