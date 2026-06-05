
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test joining content.
#([One], [Two], [Three]).join([, ], last: [ and ]).