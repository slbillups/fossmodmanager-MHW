import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    headers: {
        'Cross-Origin-Opener-Policy': 'same-origin',
        'Cross-Origin-Embedder-Policy': 'require-corp',
        'Content-Security-Policy': "default-src * 'unsafe-inline' 'unsafe-eval' data: blob:; connect-src * ipc: http://ipc.localhost;",
        'Timing-Allow-Origin': 'https://developer.mozilla.org, https://*.tauri.app',
        'Access-Control-Expose-Headers': 'fossmodmanager-version',
        'Tauri-Custom-Header': "Application-Version 0.1.0; Application-Name fossmodmanager; Application-Sub fossmodmanager-vortexAPI-test; Application-Author FossModManager; Application-Website https://github.com/slbillups/fossmodmanager; Application-Description A mod manager for FossMods"
    },
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
