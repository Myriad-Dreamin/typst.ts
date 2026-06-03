
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show "A": "B"
#show "[": "("
#show "]": ")"
#show "[2]": set text(red)

@netwok A \
@arrgh B

#show bibliography: none
#bibliography("/assets/bib/works.bib")