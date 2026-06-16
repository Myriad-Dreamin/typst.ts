// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show "\u{0302}": box(inset: (bottom: 5pt), text(0.5em, sym.diamond.small))
$hat(X)$, $hat(x)$