
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that consecutive, embedded RTL runs stay RTL.
// Here, we have three runs: "גֶ", bold "שֶׁ", and "ם".
#let content = par[Aגֶ#strong[שֶׁ]םB]
#set text(font: ("Linux Libertine", "Noto Serif Hebrew"))
#text(lang: "he", content)
#text(lang: "de", content)
