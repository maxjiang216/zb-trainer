import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Static SPA; Vercel serves the built `dist/` (see vercel.json rewrite).
export default defineConfig({
  plugins: [react()],
});
