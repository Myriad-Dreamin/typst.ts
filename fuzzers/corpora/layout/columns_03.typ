
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test the expansion behaviour.
#set page(height: 2.5cm, width: 7.05cm)

#rect(inset: 6pt, columns(2, [
    ABC \
    BCD
    #colbreak()
    DEF
]))
