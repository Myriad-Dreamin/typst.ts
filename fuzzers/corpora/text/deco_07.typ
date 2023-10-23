
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test underline background
#set underline(background: true, stroke: (thickness: 0.5em, paint: red, cap: "round"))
#underline[This is in the background]
