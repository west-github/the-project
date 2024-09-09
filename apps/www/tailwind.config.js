import defaultTheme from "tailwindcss/defaultTheme"
import plugins from "tailwindcss/plugin";


/** @type {import('tailwindcss').Config} */
export default {
  content: ['./routes/**/*.{html,js,svelte,ts}', './lib/**/*.{html,js,svelte,ts}'],
  theme: {},
  plugins: [plugins(function ({ addUtilities }) {
    addUtilities({
      ".center": {
        "display": "flex",
        "align-items": "center",
        "justify-content": "center"
      },

      ".between": {
        "display": "flex",
        "align-items": "center",
        "justify-content": "space-between"
      },
      ".void": {
        "cursor": "none",
        "pointer-events": "none",
        "user-select": "none"
      }
    })
  })]
};