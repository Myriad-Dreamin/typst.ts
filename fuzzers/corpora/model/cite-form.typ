
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show: it => context { set page(width: 200pt) if target() == "paged"; it }

Nothing: #cite(<arrgh>, form: none)

#cite(<netwok>, form: "prose") say stuff.

#bibliography("/assets/bib/works.bib", style: "apa")