
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: 200pt)
#set heading(numbering: "1.")
#show bibliography: set heading(numbering: "1.")

= Multiple Bibs
Now we have multiple bibliographies containing #cite("glacier-melt", "keshav2007read")
#bibliography(("/assets/files/works.bib", "/assets/files/works_too.bib"))
