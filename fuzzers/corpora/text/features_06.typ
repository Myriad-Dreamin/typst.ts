
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test extra number stuff.
#set text(font: "IBM Plex Serif")
0 vs. #text(slashed-zero: true)[0] \
1/2 vs. #text(fractions: true)[1/2]
