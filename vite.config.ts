import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Configuration recommandée par Tauri : port fixe, pas d'effacement du terminal.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: false,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "es2021",
    minify: "esbuild",
    sourcemap: false,
  },
});
