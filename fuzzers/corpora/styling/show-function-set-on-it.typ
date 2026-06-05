
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// This doesn't have an effect. An element is materialized before any show
// rules run.
#show heading: it => { set heading(numbering: "(I)"); it }
= Heading