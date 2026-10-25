"use client";

import { useEffect, useMemo, useRef } from "react";
import { Canvas, useLoader, useThree } from "@react-three/fiber";
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
  const textures = useLoader(TextureLoader, keys);
  const { invalidate } = useThree();
  useMemo(() => {
    textures.forEach((texture) => {
      texture.colorSpace = SRGBColorSpace;
      texture.minFilter = LinearFilter;
      texture.magFilter = LinearFilter;
    });
  }, [textures]);

  useEffect(() => {
    const update = (event: Event) => {
      const value = Math.max(0, Math.min(1, (event as CustomEvent<number>).detail));
      if (!group.current) return;
      group.current.rotation.set(-0.11 + value * 0.27, -0.35 + value * 0.83, -0.055 + value * 0.12);
      group.current.position.set(0, 0.2 - value * 0.18, -value * 0.55);
      invalidate();
    };
    window.addEventListener("novydeck:product-progress", update);
    return () => window.removeEventListener("novydeck:product-progress", update);
  }, [invalidate]);

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
            <meshStandardMaterial color="#151d30" metalness={0.40} roughness={0.47} />
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
      frameloop="demand"
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
