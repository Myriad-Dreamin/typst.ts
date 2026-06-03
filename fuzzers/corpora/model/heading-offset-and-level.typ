
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Passing level directly still overrides all other set values
#set heading(numbering: "1.1", offset: 1)
#heading(level: 1)[Still level 1]