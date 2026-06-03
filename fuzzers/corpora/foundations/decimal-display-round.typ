
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Display less digits.
#calc.round(decimal("-3.9191919191919191919191919195"), digits: 4) \
#calc.round(decimal("5.0000000000"), digits: 4)