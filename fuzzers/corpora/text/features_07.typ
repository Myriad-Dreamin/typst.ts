
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test raw features.
#text(features: ("smcp",))[Smcp] \
fi vs. #text(features: (liga: 0))[No fi]
