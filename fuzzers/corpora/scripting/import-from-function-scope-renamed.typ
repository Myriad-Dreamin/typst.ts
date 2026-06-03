
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Renamed module import with function scopes.
#import enum as othernum
#test(enum, othernum)