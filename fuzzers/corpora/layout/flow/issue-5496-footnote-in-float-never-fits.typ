
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test whether an overlarge footnote in a float also does not cause an
// infinite loop.
#set page(width: 20pt, height: 20pt)

#place(
  top,
  float: true,
  footnote(text(size: 15pt)[a] * 100)
)