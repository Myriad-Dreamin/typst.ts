
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test strike background
#set strike(background: true, stroke: 5pt + red)
#strike[This is in the background]
