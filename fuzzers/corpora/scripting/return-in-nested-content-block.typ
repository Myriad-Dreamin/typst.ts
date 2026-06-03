
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test return with joining and content.

#let f(text, caption: none) = {
  text
  if caption == none [\.#return]
  [, ]
  emph(caption)
  [\.]
}

#f(caption: [with caption])[My figure]

#f[My other figure]