
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Without the word joiner between the dots and the page number,
// the page number would be alone in its line.
#set page(width: 125pt)
#set heading(numbering: "1.a.")
#show heading: none

#outline()

= A
== This just fits here