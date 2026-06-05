
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// A forced `latn` script inhibits Devanagari font features.
#set text(font: ("Libertinus Serif", "IBM Plex Sans Devanagari"), script: "latn")
ABCअपार्टमेंट