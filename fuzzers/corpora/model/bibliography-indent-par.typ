
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that an indent-based bibliography does not produce paragraphs.
#show par: highlight

@Zee04
@keshav2007read

#bibliography("/assets/bib/works_too.bib", style: "mla")