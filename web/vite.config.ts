import { resolve } from "node:path";

import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  build: {
    rollupOptions: {
      // The alert and stinger pages are what the display itself loads. They are separate entry
      // points so neither pulls in the admin interface, which would slow down the one moment
      // where load time is visible on the wall.
      input: {
        main: resolve(import.meta.dirname, "index.html"),
        notify: resolve(import.meta.dirname, "notify.html"),
        stinger: resolve(import.meta.dirname, "stinger.html"),
      },
    },
  },
  server: {
    port: 5173,
    proxy: {
      "/api": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
    },
  },
});
