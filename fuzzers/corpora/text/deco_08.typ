
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test overline background
#set overline(background: true, stroke: (thickness: 0.5em, paint: red, cap: "round"))
#overline[This is in the background]

