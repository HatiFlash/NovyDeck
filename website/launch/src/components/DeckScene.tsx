"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { Canvas, useFrame, useLoader } from "@react-three/fiber";
import { Group, LinearFilter, SRGBColorSpace, TextureLoader } from "three";
import { RoundedBoxGeometry } from "three/addons/geometries/RoundedBoxGeometry.js";

// Geometry is adapted from NovyDeck's own Remotion Deck3D scene, not a third-party model.
const shell = new RoundedBoxGeometry(5.94, 0.36, 3.9, 6, 0.30);
const base = new RoundedBoxGeometry(6.02, 0.10, 3.98, 6, 0.33);
const face = new RoundedBoxGeometry(5.78, 0.04, 3.72, 4, 0.22);
const cap = new RoundedBoxGeometry(1.035, 0.31, 1.02, 4, 0.12);
const keys = Array.from({ length: 15 }, (_, i) => `assets/key-${String(i).padStart(2, "0")}.webp`);

function Hardware() {
  const group = useRef<Group>(null);
  const [lit, setLit] = useState(0);
  const target = useRef({ x: 0, y: 0, scroll: 0 });
  const reduced = useRef(false);
  const textures = useLoader(TextureLoader, keys);
  useMemo(() => {
    textures.forEach((texture) => {
      texture.colorSpace = SRGBColorSpace;
      texture.minFilter = LinearFilter;
      texture.magFilter = LinearFilter;
    });
  }, [textures]);

  useEffect(() => {
    const preference = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updatePreference = () => { reduced.current = preference.matches; };
    const onPointer = (event: Event) => {
      const { x, y } = (event as CustomEvent<{ x: number; y: number }>).detail;
      target.current.x = x;
      target.current.y = y;
      setLit(Math.max(0, Math.min(14, Math.floor((x + .5) * 5) + Math.floor((y + .5) * 3) * 5)));
    };
    const onScroll = () => {
      target.current.scroll = Math.max(0, Math.min(1, window.scrollY / Math.max(window.innerHeight, 1)));
      if (!reduced.current) setLit(Math.min(14, Math.floor(target.current.scroll * 14)));
    };
    updatePreference();
    preference.addEventListener("change", updatePreference);
    window.addEventListener("novydeck:hero-pointer", onPointer);
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
    return () => { preference.removeEventListener("change", updatePreference); window.removeEventListener("novydeck:hero-pointer", onPointer); window.removeEventListener("scroll", onScroll); };
  }, []);

  useFrame(({ clock }, delta) => {
    if (!group.current) return;
    const t = target.current;
    const quiet = reduced.current;
    const s = quiet ? 0 : t.scroll;
    const idle = quiet ? 0 : Math.sin(clock.elapsedTime * .55) * .045;
    const ease = 1 - Math.exp(-Math.min(delta, .05) * (quiet ? 12 : 5));
    group.current.rotation.x += (-.11 + s * .28 + t.y * .14 - group.current.rotation.x) * ease;
    group.current.rotation.y += (-.35 + s * .62 + t.x * .38 + idle - group.current.rotation.y) * ease;
    group.current.rotation.z += (-.055 + t.x * .07 - group.current.rotation.z) * ease;
    group.current.position.y += ((quiet ? .2 : .2 + Math.sin(clock.elapsedTime * .8) * .06 - s * .1) - group.current.position.y) * ease;
    group.current.scale.setScalar(1 + s * .12);
  });

  return (
    <group ref={group} rotation={[-0.11, -0.35, -0.055]} position={[0, 0.2, 0]}>
      <mesh geometry={shell} receiveShadow castShadow>
        <meshStandardMaterial color="#101b2d" metalness={0.62} roughness={0.44} />
      </mesh>
      <mesh geometry={base} position={[0, -0.23, 0]}>
        <meshStandardMaterial color="#176780" metalness={0.67} roughness={0.36} />
      </mesh>
      <mesh geometry={face} position={[0, 0.12, 0]}>
        <meshStandardMaterial color="#26324b" metalness={0.65} roughness={0.47} />
      </mesh>
      {textures.map((texture, i) => (
        <group key={i} position={[(i % 5 - 2) * 1.13, 0.27, (Math.floor(i / 5) - 1) * 1.13]}>
          <mesh geometry={cap} castShadow>
            <meshStandardMaterial color={i === lit ? "#377cbb" : "#151d30"} emissive={i === lit ? "#2997ff" : "#000000"} emissiveIntensity={i === lit ? 0.75 : 0} metalness={0.40} roughness={0.47} />
          </mesh>
          <mesh position={[0, 0.162, 0]} rotation={[-Math.PI / 2, 0, 0]}>
            <planeGeometry args={[0.87, 0.86]} />
            <meshBasicMaterial map={texture} transparent toneMapped={false} />
          </mesh>
        </group>
      ))}
    </group>
  );
}

export default function DeckScene() {
  return (
    <Canvas
      aria-label="Reconstitution 3D illustrative du FIFINE D6 à 15 touches"
      role="img"
      frameloop="always"
      dpr={[1, 1.6]}
      camera={{ position: [0, 5.4, 8.8], fov: 39, near: 0.1, far: 50 }}
      gl={{ antialias: true, alpha: true, powerPreference: "low-power" }}
      style={{ width: "100%", height: "100%" }}
    >
      <ambientLight intensity={1.55} />
      <hemisphereLight args={["#9bd8fb", "#0b1425", 1.7]} />
      <directionalLight position={[-3, 7, 6]} intensity={2.7} color="#a8e9ff" />
      <pointLight position={[4, 2, -4]} intensity={26} distance={15} color="#087db7" />
      <Hardware />
    </Canvas>
  );
}
