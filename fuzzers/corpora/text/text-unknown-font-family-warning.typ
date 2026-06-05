
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#text(font: "libertinus serif")[I exist,]
// Warning: 13-26 unknown font family: nonexistent
#text(font: "nonexistent")[but]
// Warning: 17-35 unknown font family: also-nonexistent
#set text(font: "also-nonexistent")
I
// Warning: 23-55 unknown font family: list-of
// Warning: 23-55 unknown font family: nonexistent-fonts
#let var = text(font: ("list-of", "nonexistent-fonts"))[don't]
#var