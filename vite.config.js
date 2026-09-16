import { defineConfig, loadEnv } from "vite";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const appMode = (
    process.env.MODE ||
    env.MODE ||
    process.env.VITE_MODE ||
    env.VITE_MODE ||
    "PROD"
  ).toUpperCase();

  const apiBaseUrl =
    process.env.API_BASE_URL ||
    env.API_BASE_URL ||
    process.env.API_URL ||
    env.API_URL ||
    process.env.VITE_API_BASE_URL ||
    env.VITE_API_BASE_URL ||
    (appMode === "DEV"
      ? "https://api.tayin.uz"
      : "https://api.pharma-cosmos.uz:4443");

  const appUrl =
    process.env.APP_URL ||
    env.APP_URL ||
    process.env.VERCEL_URL ||
    env.VERCEL_URL ||
    "https://face-reg-cyan.vercel.app/";

  return {
    root: "frontend",
    define: {
      __APP_MODE__: JSON.stringify(appMode),
      __API_BASE_URL__: JSON.stringify(apiBaseUrl),
      __APP_URL__: JSON.stringify(appUrl),
    },
    build: {
      outDir: "../dist",
      emptyOutDir: true,
    },
    clearScreen: false,
  };
});
