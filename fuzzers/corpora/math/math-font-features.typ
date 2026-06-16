// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
$ nothing $
$ "hi ∅ hey" $
$ sum_(i in NN) 1 + i $
#show math.equation: set text(features: ("cv01",), fallback: false)
$ nothing $
$ "hi ∅ hey" $
$ sum_(i in NN) 1 + i $