
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test text turning off (standard) ligatures of the font.
#text(ligatures: false)[fi Qu] vs fi Qu \
// Test text turning on historical ligatures of the font.
abstract vs #text(historical-ligatures: true)[abstract] \
// Test text turning on discretionary ligatures of the font.
waltz vs #text(discretionary-ligatures: true)[waltz]