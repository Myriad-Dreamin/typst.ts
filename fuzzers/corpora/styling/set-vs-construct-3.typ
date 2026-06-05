
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The inner rectangle should also be yellow here.
// (and therefore invisible)
#[#set rect(fill: yellow);#text(1em, rect(inset: 5pt, rect()))]