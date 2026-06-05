
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show: it => context { set page(width: 200pt) if target() == "paged"; it }

#set heading(numbering: "1.")
#show bibliography: set heading(numbering: "1.")

= Multiple Bibs
Now we have multiple bibliographies containing @glacier-melt @keshav2007read
#bibliography(("/assets/bib/works.bib", "/assets/bib/works_too.bib"))