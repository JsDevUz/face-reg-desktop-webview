import { defineConfig, loadEnv } from "vite";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const appMode = (process.env.MODE || env.MODE || "").toUpperCase();
  if (!["DEV", "PROD"].includes(appMode)) {
    throw new Error(".env ichida MODE=DEV yoki MODE=PROD bo‘lishi shart");
  }

  const apiBaseUrl =
    appMode === "DEV"
      ? "https://api.tayin.uz"
      : "https://api.pharma-cosmos.uz:4443";

  return {
    root: "frontend",
    define: {
      __APP_MODE__: JSON.stringify(appMode),
      __API_BASE_URL__: JSON.stringify(apiBaseUrl),
    },
    build: {
      outDir: "../dist",
      emptyOutDir: true,
    },
    clearScreen: false,
  };
});
