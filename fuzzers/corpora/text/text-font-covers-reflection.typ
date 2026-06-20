
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// reflect "latin-in-cjk" covers
#set text(font: (name: "Ubuntu", covers: "latin-in-cjk"))
#context test(text.font, (name: "ubuntu", covers: "latin-in-cjk"))

// reflect regex covers
#set text(font: (name: "Ubuntu", covers: regex("\d")))
#context test(text.font, (name: "ubuntu", covers: regex("\d")))

// reflect font list with covers
#set text(font: ((name: "Ubuntu", covers: regex("\d")), "IBM Plex Serif"))
#context test(text.font, ((name: "ubuntu", covers: regex("\d")), "ibm plex serif"))