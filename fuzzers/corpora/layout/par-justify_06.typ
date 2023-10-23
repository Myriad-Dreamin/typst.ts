
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that justification cannot lead to a leading space
#set par(justify: true)
#set text(size: 12pt)
#set page(width: 45mm, height: auto)

lorem ipsum 1234, lorem ipsum dolor sit amet

#"  leading whitespace should still be displayed"
