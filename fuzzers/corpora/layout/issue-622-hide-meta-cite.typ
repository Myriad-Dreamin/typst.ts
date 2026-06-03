
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that metadata of hidden stuff stays available.
#set cite(style: "chicago-shortened-notes")

A pirate. @arrgh \
#set text(2pt)
#hide[
  A @arrgh pirate.
  #bibliography("/assets/bib/works.bib")
]