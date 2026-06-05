
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test whitespace after memoized part.
#( (x: () => 1 ) => 1 )
//     -------
//     This is memoized and we want to ensure that whitespace after this
//     is handled correctly.