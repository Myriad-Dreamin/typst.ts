
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Mixing renamed module import from function with renamed item import.
#import assert as asrt
#import asrt: ne as asne
#asne(1, 2)