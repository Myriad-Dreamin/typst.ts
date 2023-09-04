
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Footnote ref in footnote
#footnote[Reference to next @fn]
#footnote[Reference to myself @fn]<fn>
#footnote[Reference to previous @fn]
