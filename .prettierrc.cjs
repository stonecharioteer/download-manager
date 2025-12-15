module.exports = {
  printWidth: 80,
  tabWidth: 2,
  useTabs: false,
  semi: false,
  singleQuote: true,
  bracketSpacing: true,
  trailingComma: 'es5',
  arrowParens: 'avoid',
  quoteProps: 'preserve',
  embeddedLanguageFormatting: 'auto',
  proseWrap: 'always',
  htmlWhitespaceSensitivity: 'css',
  endOfLine: 'lf',
  overrides: [
    {
      files: '*.md',
      options: {
        printWidth: 80,
        proseWrap: 'always'
      }
    },
    {
      files: ['*.json', '*.jsonc'],
      options: {
        printWidth: 120
      }
    }
  ]
}
