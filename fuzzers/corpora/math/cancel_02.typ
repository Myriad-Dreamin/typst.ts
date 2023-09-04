
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Inverted
$a + cancel(x, inverted: #true) - cancel(x, inverted: #true) + 10 + cancel(y) - cancel(y)$
$ x + cancel("abcdefg", inverted: #true) $
