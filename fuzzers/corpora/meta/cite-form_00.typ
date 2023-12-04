
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: 200pt)

Nothing: #cite(<arrgh>, form: none)

#cite(<netwok>, form: "prose") say stuff.

#bibliography("/assets/files/works.bib", style: "apa")
