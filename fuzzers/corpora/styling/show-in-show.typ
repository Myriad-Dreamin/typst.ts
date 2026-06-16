// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test set and show in code blocks.
#show heading: it => {
  set text(red)
  show "ding": [🛎]
  it.body
}

= Heading