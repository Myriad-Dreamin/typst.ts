
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that a repeated hard hyphen keeps its styles.
#set page(width: 2cm)
#set text(lang: "es")
Hello-#text(red)[world]