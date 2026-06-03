
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that show rules apply to queried elements, i.e. that the content
// returned from `query` isn't yet marked as prepared.
#set heading(numbering: "1.")
#show heading: underline
= Hi

#set heading(numbering: "I.")
#show heading: set text(blue)
#show heading: highlight.with(fill: aqua.lighten(50%))
= Bye

// New show rules apply to this, but its location and the materialized fields
// from the original are retained.
#context query(heading).join()