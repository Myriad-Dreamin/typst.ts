
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let ( key :  /* hi */ binding ) = ( key: "ok" )
#test(binding, "ok")