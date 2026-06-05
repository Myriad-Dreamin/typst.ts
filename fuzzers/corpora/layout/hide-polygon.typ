
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
Hidden:
#hide[
  #polygon((20%, 0pt),
    (60%, 0pt),
    (80%, 2cm),
    (0%,  2cm),)
]
#polygon((20%, 0pt),
  (60%, 0pt),
  (80%, 2cm),
  (0%,  2cm),)