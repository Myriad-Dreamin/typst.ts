
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The two aligned elements should be displayed in separate lines.
#align(right)[@netwok]
#align(right)[b]

#show bibliography: none
#bibliography("/assets/bib/works.bib")