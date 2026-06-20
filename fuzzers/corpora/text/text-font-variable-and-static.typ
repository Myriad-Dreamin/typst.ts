
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// This font exists both in its static and variable version.
#set text(font: "Source Serif 4")
Hello _world_ *with* #text(weight: 550)[_Source Serif._]