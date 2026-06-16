// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test format manual
// Warning: 8-14 `image.decode` is deprecated, directly pass bytes to `image` instead
// Hint: 8-14 it will be removed in Typst 0.15.0
#image.decode(read("/assets/images/tiger.jpg", encoding: none), format: "jpg", width: 80%)
