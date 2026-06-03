
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show: it => context { set page(width: 200pt) if target() == "paged"; it }

= Details
See also @arrgh #cite(<distress>, supplement: [p.~22]), @arrgh[p.~4], and @distress[p.~5].
#bibliography("/assets/bib/works.bib")