
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test format auto detect
// Warning: 8-14 `image.decode` is deprecated, directly pass bytes to `image` instead
// Hint: 8-14 it will be removed in Typst 0.15.0
#image.decode(read("/assets/images/tiger.jpg", encoding: none), width: 80%)