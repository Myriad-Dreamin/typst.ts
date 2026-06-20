
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test selecting between multiple alternates.
#set text(font: "Libertinus Serif")
#text(alternates: false, [ß]) vs #text(alternates: true, [ß]) vs #text(alternates: 2, [ß])