
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that consecutive, embedded  LTR runs stay LTR.
// Here, we have two runs: "A" and italic "B".
#let content = par[أنت A#emph[B]مطرC]
#set text(font: ("PT Sans", "Noto Sans Arabic"))
#text(lang: "ar", content)
#text(lang: "de", content)
