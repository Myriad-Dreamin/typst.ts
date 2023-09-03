
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test justified breaks.
#set par(justify: true)
With a soft #linebreak(justify: true)
break you can force a break without #linebreak(justify: true)
breaking justification. #linebreak(justify: false)
Nice!
