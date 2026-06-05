
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Citing a second time showed all authors instead of "et al".
@mcintosh_anxiety \
@mcintosh_anxiety
#show bibliography: none
#bibliography("/assets/bib/works.bib", style: "chicago-author-date")