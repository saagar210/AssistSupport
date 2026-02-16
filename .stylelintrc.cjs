module.exports = {
  extends: ["stylelint-config-standard"],
  ignoreFiles: ["**/*.{js,jsx,ts,tsx}"],
  rules: {
    "at-rule-no-unknown": [
      true,
      {
        ignoreAtRules: [
          "tailwind",
          "apply",
          "layer",
          "config",
          "theme",
          "utility",
          "variant",
          "custom-variant",
        ],
      },
    ],
    "selector-class-pattern": "^[a-z][a-z0-9\\-_:]*$",
    "import-notation": null,
    "custom-property-empty-line-before": null,
  },
};
