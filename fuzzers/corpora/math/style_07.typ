
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test using rules for symbols
#show sym.tack: it => $#h(1em) it #h(1em)$
$ a tack b $
