
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The language tag does not have to be a valid identifier.
// Warning: 15-25 no whitespace between language tag and raw text
// Hint: 15-19 if the current behavior is correct, please add a space after `lang`
// Hint: 15-19 otherwise, add a space or newline after the initial backticks
// Hint: 15-25 currently, Typst is treating `lang` as the language tag
// Hint: 15-25 in the next version of Typst, this will change and we will treat all text until the first whitespace as the language tag
#let raw = ```lang.tag++ test```
#test(raw.lang, "lang")
#test(raw.text, ".tag++ test")
#test(raw.block, false)