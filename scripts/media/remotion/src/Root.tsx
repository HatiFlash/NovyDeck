import React from 'react';
import {Composition} from 'remotion';
import {NovyDeckDemo} from './NovyDeckDemo';
import {MotionPreview, MotionPreviewVertical} from './MotionPreview';

export const RemotionRoot: React.FC = () => <>
  <Composition id="MotionPreview" component={MotionPreview} fps={30} durationInFrames={360} width={1920} height={1080}/>
  <Composition id="MotionPreviewVertical" component={MotionPreviewVertical} fps={30} durationInFrames={360} width={1080} height={1920}/>
  <Composition
  id="NovyDeckDemo"
  component={NovyDeckDemo}
  fps={30}
  durationInFrames={1600}
  width={1920}
  height={1080}
  />
</>;
