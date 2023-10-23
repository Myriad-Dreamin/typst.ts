
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test highlight.
This is the built-in #highlight[highlight with default color].
We can also specify a customized value
#highlight(fill: green.lighten(80%))[to highlight].
