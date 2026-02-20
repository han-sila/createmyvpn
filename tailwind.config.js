/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        gray: {
          50:  "#F0F4FF",
          100: "#E4EAF8",
          200: "#C9D4EC",
          300: "#A8B8D8",
          400: "#7B91B0",
          500: "#5A6F8F",
          600: "#3D5068",
          700: "#263546",
          800: "#16243A",
          900: "#0D1420",
          950: "#080C14",
        },
        primary: {
          50:  "#ECFEFF",
          100: "#CFFAFE",
          200: "#A5F3FC",
          300: "#67E8F9",
          400: "#22D3EE",
          500: "#06B6D4",
          600: "#0891B2",
          700: "#0E7490",
          800: "#155E75",
          900: "#164E63",
          950: "#083344",
        },
      },
      animation: {
        "glow-pulse": "glow-pulse 2.5s ease-in-out infinite",
        "fade-in":    "fade-in 0.25s ease-out",
        "scale-in":   "scale-in 0.2s cubic-bezier(0.175, 0.885, 0.32, 1.275)",
      },
      keyframes: {
        "glow-pulse": {
          "0%, 100%": { boxShadow: "0 0 15px rgba(16,185,129,0.20)" },
          "50%":       { boxShadow: "0 0 50px rgba(16,185,129,0.50)" },
        },
        "fade-in": {
          from: { opacity: "0", transform: "translateY(6px)" },
          to:   { opacity: "1", transform: "translateY(0)" },
        },
        "scale-in": {
          from: { opacity: "0", transform: "scale(0.4)" },
          to:   { opacity: "1", transform: "scale(1)" },
        },
      },
    },
  },
  plugins: [],
};
