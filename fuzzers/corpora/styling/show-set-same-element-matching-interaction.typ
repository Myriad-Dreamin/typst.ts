// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that show-set rules on the same element don't affect each other. This
// could be implemented, but isn't as of yet.
#show heading.where(level: 1): set heading(numbering: "(I)")
#show heading.where(numbering: "(I)"): set text(red)
= Heading