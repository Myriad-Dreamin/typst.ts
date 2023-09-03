
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test hyphenating english and greek.
#set text(hyphenate: true)
#set page(width: auto)
#grid(
  columns: (50pt, 50pt),
  [Warm welcomes to Typst.],
  text(lang: "el")[διαμερίσματα. \ λατρευτός],
)
