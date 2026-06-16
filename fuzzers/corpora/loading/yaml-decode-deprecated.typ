// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 15-21 `yaml.decode` is deprecated, directly pass bytes to `yaml` instead
// Hint: 15-21 it will be removed in Typst 0.15.0
#let _ = yaml.decode
