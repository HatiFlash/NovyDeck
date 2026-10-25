import type { NextConfig } from "next";

const basePath = process.env.NEXT_PUBLIC_SITE_BASE_PATH || "";

const nextConfig: NextConfig = {
  output: "export",
  images: { unoptimized: true },
  basePath: basePath || undefined,
  assetPrefix: basePath ? `${basePath}/` : "./",
  reactStrictMode: true,
};
export default nextConfig;
