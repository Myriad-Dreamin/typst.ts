
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Trailing commas and empty args introduce blank content in math
$ sin(,x,y,,,) $
// with whitespace/trivia:
$ sin( ,/**/x/**/, , /**/y, ,/**/, ) $