
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set cite(style: "chicago-notes")

A pirate. @arrgh \
#set text(2pt)
#hide[
  A @arrgh pirate.
  #bibliography("/assets/files/works.bib")
]
