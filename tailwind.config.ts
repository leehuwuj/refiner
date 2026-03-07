import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./index.html", "./settings.html", "./compact-popup.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        accent: "var(--accent)",
        "accent-alt": "var(--accent-alt)",
      },
      backdropBlur: {
        glass: "24px",
      },
      borderRadius: {
        app: "20px",
        panel: "14px",
      },
      animation: {
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "fade-in": "fadeIn 0.2s ease-out",
        "slide-up": "slideUp 0.2s ease-out",
      },
      keyframes: {
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        slideUp: {
          "0%": { opacity: "0", transform: "translateY(4px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
      },
    },
  },
  plugins: [
    function ({ addUtilities }: any) {
      addUtilities({
        ".scrollbar-hide": {
          "-ms-overflow-style": "none",
          "scrollbar-width": "none",
          "&::-webkit-scrollbar": {
            display: "none",
          },
        },
        ".glass": {
          background: "var(--glass-panel-bg)",
          "backdrop-filter": "var(--glass-blur)",
          "-webkit-backdrop-filter": "var(--glass-blur)",
          border: "1px solid var(--glass-border)",
          "box-shadow": "var(--glass-shadow), var(--glass-inner-highlight)",
        },
        ".glass-app": {
          background: "var(--glass-app-bg)",
          "backdrop-filter": "var(--glass-blur)",
          "-webkit-backdrop-filter": "var(--glass-blur)",
          "box-shadow": "var(--glass-shadow)",
        },
        ".glass-control": {
          background: "var(--glass-control-bg)",
          border: "1px solid var(--glass-border)",
          transition: "background 0.15s ease",
          "&:hover": {
            background: "var(--glass-control-hover)",
          },
        },
        ".text-primary": { color: "var(--text-primary)" },
        ".text-secondary": { color: "var(--text-secondary)" },
        ".text-tertiary": { color: "var(--text-tertiary)" },
        ".border-glass": { "border-color": "var(--glass-border)" },
        ".border-glass-bright": { "border-color": "var(--glass-border-bright)" },
      });
    },
  ],
};

export default config;
