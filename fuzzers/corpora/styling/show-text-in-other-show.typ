// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Replace worlds but only in lists.
#show list: it => [
  #show "World": [🌎]
  #it
]

World
- World