
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let src = ```yaml
hi:
  type: Book
```

#show heading: none
#bibliography((
  "/assets/bib/works.bib",
  path("/assets/bib/works_too.bib"),
  bytes(src.text)
))