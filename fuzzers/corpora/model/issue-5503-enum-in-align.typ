
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// `align` is block-level and should interrupt an enum.
+ a
+ b
#align(right)[+ c]
+ d