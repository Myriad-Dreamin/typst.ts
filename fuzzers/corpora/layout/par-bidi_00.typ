
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test reordering with different top-level paragraph directions.
#let content = par[Text טֶקסט]
#text(lang: "he", content)
#text(lang: "de", content)
