
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#let red = rgb("fc0030")

// Basic strikethrough.
#strike[Statements dreamt up by the utterly deranged.]

// Move underline down.
#underline(offset: 5pt)[Further below.]

// Different color.
#underline(stroke: red, evade: false)[Critical information is conveyed here.]

// Inherits font color.
#text(fill: red, underline[Change with the wind.])

// Both over- and underline.
#overline(underline[Running amongst the wolves.])
