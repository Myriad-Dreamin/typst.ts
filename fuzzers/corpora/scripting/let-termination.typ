
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Termination.

// Terminated by line break.
#let v1 = 1
One

// Terminated by semicolon.
#let v2 = 2; Two

// Terminated by semicolon and line break.
#let v3 = 3;
Three

#test(v1, 1)
#test(v2, 2)
#test(v3, 3)