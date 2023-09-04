
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#let memes = "ArE mEmEs gReAt?";
#test(lower(memes), "are memes great?")
#test(upper(memes), "ARE MEMES GREAT?")
#test(upper("Ελλάδα"), "ΕΛΛΆΔΑ")
