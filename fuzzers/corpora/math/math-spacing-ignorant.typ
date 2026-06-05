
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test spacing with ignorant elements
$#metadata(none) "text"$ \
$#place(dx: 5em)[Placed] "text"$ \
// Operator spacing
$#counter("test").update(3) + b$ \
$#place(dx: 5em)[a] + b$
// Validate that ignorant elements are layouted
#context test(counter("test").get(), (3,))