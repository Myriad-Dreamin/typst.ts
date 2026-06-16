// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// `align` is block-level and should interrupt an enum.
+ a
+ b
#align(right)[+ c]
+ d