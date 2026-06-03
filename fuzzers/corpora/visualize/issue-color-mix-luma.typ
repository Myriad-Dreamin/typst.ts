
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// When mixing luma colors, we accidentally used the wrong component.
#rect(fill: gradient.linear(black, silver, space: luma))