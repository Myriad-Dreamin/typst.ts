
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 12-22 empty raw text
// Hint: 12-22 Typst is treating `lang` as the language tag
// Hint: 15-19 to treat this as text, add a space after the initial backticks
#let raw = ```lang```
#test(raw.lang, "lang")
#test(raw.text, "")
#test(raw.block, false)