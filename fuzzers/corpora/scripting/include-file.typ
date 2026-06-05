
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: 200pt)

// Include a file
#include "modules/chap1.typ"

// Expression as a file name.
#let chap2 = include "modu" + "les/chap" + "2.typ"

-- _Intermission_ --
#chap2