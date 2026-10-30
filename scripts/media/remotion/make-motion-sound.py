"""Deterministic 55-second stereo score and action cues; Python standard library only."""
import math
import struct
import wave
from pathlib import Path

SR = 48000
DURATION = 55
N = SR * DURATION
left = [0.0] * N
right = [0.0] * N


def cue(time, kind, gain=.14, pan=0.0):
    length = {'click':.09, 'drop':.32, 'toggle':.30, 'whoosh':.48, 'type':.055}[kind]
    count = int(length * SR)
    start = int(time * SR)
    for j in range(count):
        x = j / SR
        env = math.exp(-x * (75 if kind in ('click', 'type') else 13))
        if kind == 'whoosh':
            env = math.sin(math.pi * j / count)**2
        noise = math.sin(j*1.6180339887)*math.sin(j*2.41421356)
        if kind in ('click', 'type'):
            v = env*(math.sin(2*math.pi*(1300 if kind=='click' else 1700)*x)*.65+noise*.35)
        elif kind == 'drop':
            v = env*(math.sin(2*math.pi*(420-180*x)*x)*.75+noise*.25)
        elif kind == 'toggle':
            v = env*(math.sin(2*math.pi*(460+900*x)*x)*.7+noise*.3)
        else:
            v = env*noise
        k = start+j
        if 0 <= k < N:
            left[k] += gain*v*math.sqrt((1-pan)*.5)
            right[k] += gain*v*math.sqrt((1+pan)*.5)

# Score is continuous, harmonic and rhythmic, with fixed-duration percussive accents.
chords = [(146.83,220.0,293.66), (164.81,246.94,329.63), (174.61,261.63,349.23), (196.0,293.66,392.0)]
for j in range(N):
    sec = j / SR
    chord = chords[int(sec//6.875)%4]
    fade = min(1.,sec/1.5,(55-sec)/2.5)
    beat = sec*2.0
    kick_phase = (beat % 1.)
    kick = math.sin(2*math.pi*(55+65*math.exp(-kick_phase*28))*kick_phase)*math.exp(-kick_phase*24)*.06
    pad = sum(math.sin(2*math.pi*hz*sec + math.sin(sec*.37+i)*.08) for i,hz in enumerate(chord))*.018
    shimmer = math.sin(2*math.pi*chord[0]*2*sec)*.006
    left[j] += fade*(pad+kick+shimmer*math.sin(sec*.67))
    right[j] += fade*(pad+kick+shimmer*math.cos(sec*.71))

# Cut points: 3, 7, 14, 21, 29, 35, 42 and 49 seconds.
for s in (3,7,14,21,29,35,42,49):
    cue(s-.12,'whoosh',.14,-.25 if s%2 else .25)
for i in range(15):
    cue(.12+i*.065,'click',.065,(i%5-2)*.28)
for t in (1.5,3.7,5.3,7.8,8.4,9.1,14.7,16.0,18.2,22.1,24.4,26.0,36.4,38.7,40.2,44.4,46.9):
    cue(t,'click',.12)
for t in (19.0,24.5,39.0,46.5):
    cue(t,'drop',.18,.25)
for t in (30.4,32.2,33.7):
    cue(t,'toggle',.14,-.2)
for start, end, count in ((9.6,11.6,11),(43.1,45.5,31)):
    for i in range(count):
        cue(start+(end-start)*i/count,'type',.045,(-1 if i%2 else 1)*.3)

# Leave AAC headroom, with the decoded peak below -1 dBFS.
peak = max(max(abs(v) for v in left),max(abs(v) for v in right))
gain = .70/peak
path = Path('public/motion-soundtrack.wav')
with wave.open(str(path),'wb') as out:
    out.setnchannels(2)
    out.setsampwidth(2)
    out.setframerate(SR)
    buffer = bytearray()
    for a,b in zip(left,right):
        buffer.extend(struct.pack('<hh',int(a*gain*32767),int(b*gain*32767)))
        if len(buffer)>=262144:
            out.writeframesraw(buffer)
            buffer.clear()
    out.writeframes(buffer)
print(f'{path}: {DURATION}s stereo 48kHz; peak={20*math.log10(peak*gain):.2f} dBFS')
