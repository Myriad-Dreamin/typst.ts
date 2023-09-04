
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test stroke folding.
#set underline(stroke: 2pt, offset: 2pt)
#underline(text(red, [DANGER!]))
