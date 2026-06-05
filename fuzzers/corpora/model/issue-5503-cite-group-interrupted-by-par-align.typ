
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// `par` and `align` are block-level and should interrupt a cite group
@netwok
@arrgh
#par(leading: 5em)[@netwok]
#par[@arrgh]
@netwok
@arrgh
#align(right)[@netwok]
@arrgh

#show bibliography: none
#bibliography("/assets/bib/works.bib")