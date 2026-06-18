import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    strictPort: true,
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  build: {
    outDir: "out",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, "index.html"),
        settings: path.resolve(__dirname, "settings.html"),
        "compact-popup": path.resolve(__dirname, "compact-popup.html"),
        "language-analysis": path.resolve(__dirname, "language-analysis.html"),
      },
    },
  },
});
