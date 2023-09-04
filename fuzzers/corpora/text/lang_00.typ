
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Ensure that setting the language does have effects.
#set text(hyphenate: true)
#grid(
  columns: 2 * (20pt,),
  gutter: 1fr,
  text(lang: "en")["Eingabeaufforderung"],
  text(lang: "de")["Eingabeaufforderung"],
)
