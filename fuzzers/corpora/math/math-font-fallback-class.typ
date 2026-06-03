
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that math class is preserved even when the result is a tofu.
#show math.equation: set text(font: "Fira Math", fallback: false)
$ brace.stroked.l -1 brace.stroked.r $
$ lr(brace.stroked.l -1 brace.stroked.r) $