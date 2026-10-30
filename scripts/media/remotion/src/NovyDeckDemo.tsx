import React from 'react';
import {Audio} from '@remotion/media';
import {TransitionSeries, linearTiming} from '@remotion/transitions';
import {fade} from '@remotion/transitions/fade';
import {slide} from '@remotion/transitions/slide';
import {interpolate, staticFile, useCurrentFrame} from 'remotion';
import {Intro, Deck, Dashboard, Personalize, Drag, Workshop, OnOff, Plugins, Assistant, Motion, Outro} from './PromoScenes';

export const NovyDeckDemo:React.FC = () => {
 const frame=useCurrentFrame();
 const t=linearTiming({durationInFrames:12});
 const scenes=[
  {name:'Ouverture · NovyDeck',duration:150,component:<Intro/>},
  {name:'Deck 3D · touches LCD',duration:150,component:<Deck/>},
  {name:'Interface',duration:150,component:<Dashboard/>},
  {name:'Personnalisation',duration:160,component:<Personalize/>},
  {name:'Glisser-déposer 0.18',duration:160,component:<Drag/>},
  {name:'Atelier de blocs',duration:155,component:<Workshop/>},
  {name:'Touches ON/OFF',duration:155,component:<OnOff/>},
  {name:'Plugins et magasin',duration:155,component:<Plugins/>},
  {name:'Assistant IA',duration:145,component:<Assistant/>},
  {name:'Fluidité 0.18',duration:145,component:<Motion/>},
  {name:'Signature NovyDeck 0.18.1',duration:195,component:<Outro/>},
 ];
 return <>
  <TransitionSeries>{scenes.map((scene,i)=><React.Fragment key={scene.name}>
   {i>0 && <TransitionSeries.Transition presentation={i%3===0 ? slide({direction:'from-right'}) : fade()} timing={t}/>}
   <TransitionSeries.Sequence name={scene.name} durationInFrames={scene.duration}>{scene.component}</TransitionSeries.Sequence>
  </React.Fragment>)}</TransitionSeries>
  <Audio name="Bande-son originale stéréo" src={staticFile('soundtrack.wav')} durationInFrames={1600} volume={interpolate(frame,[0,24,1535,1600],[0,1,1,0],{extrapolateLeft:'clamp',extrapolateRight:'clamp'})}/>
 </>;
};