
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Spaces must be interior to strong textual elements for matching to work.
// For outer spaces, it is hard to say whether they would collapse.
#show "a\n": set text(blue)
#show "b\n ": set text(blue)
#show " c ": set text(blue)
a \ #h(0pt, weak: true)
b \ #h(0pt, weak: true)
$x$ c $y$