
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test abusing dynamic labels for styling.
#show <red>: set text(red)
#show <blue>: set text(blue)

*A* *B* <red> *C* #label("bl" + "ue") *D*