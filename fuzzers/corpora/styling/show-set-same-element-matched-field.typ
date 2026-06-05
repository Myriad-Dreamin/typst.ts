
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test setting the thing we just matched on.
// This is quite cursed, but it works.
#set heading(numbering: "(I)")
#show heading.where(numbering: "(I)"): set heading(numbering: "1.")
= Heading