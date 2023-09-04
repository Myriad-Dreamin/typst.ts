
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Display
#set page(width: auto)
$ a + b + cancel(b + c) - cancel(b) - cancel(c) - 5 + cancel(6) - cancel(6) $
$ e + (a dot.c cancel((b + c + d)))/(cancel(b + c + d)) $
