
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let t(expected) = context {
  let elems = query(selector(metadata).after(here()))
  let val = elems.first().value
  test(val, expected)
}

#{
  t("a")
  place(metadata("a"))
}

#{
  t("b")
  block(height: 1fr, metadata("b"))
}