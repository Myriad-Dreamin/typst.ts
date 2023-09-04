
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that square does not overflow page.
#set page(width: 100pt, height: 75pt)
#square(fill: conifer)[
  But, soft! what light through yonder window breaks?
]
