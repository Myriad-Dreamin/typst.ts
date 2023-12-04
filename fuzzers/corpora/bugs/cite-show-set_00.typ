
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#show cite: set text(red)
A @netwok @arrgh.
B #cite(<netwok>) #cite(<arrgh>).

#show bibliography: none
#bibliography("/assets/files/works.bib")
