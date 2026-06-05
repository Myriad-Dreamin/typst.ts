
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show "hey \"": [@arrgh]
#show "dis": [@distress]
@netwok hey " dis

#show bibliography: none
#bibliography("/assets/bib/works.bib", style: "american-physics-society")