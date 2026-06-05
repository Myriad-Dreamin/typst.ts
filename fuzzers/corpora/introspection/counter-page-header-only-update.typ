
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Header should not be affected by default.
// To affect it, put the counter update before the `set page`.
#set page(
  numbering: "1",
  number-align: top + center,
  margin: (top: 20pt),
)

#counter(page).update(5)