
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Footnote call with label
#footnote(<fn>)
#footnote[Hi]<fn>
#ref(<fn>)
#footnote(<fn>)
