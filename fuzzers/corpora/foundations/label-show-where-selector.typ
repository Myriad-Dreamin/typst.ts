
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test labelled headings.
#show heading: set text(10pt)
#show heading.where(label: <intro>): underline

= Introduction <intro>
The beginning.

= Conclusion
The end.