
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "edge-case.typ": special-types-for-human
#for value in special-types-for-human {
  test(
    toml.encode((key: value)),
    toml.encode((key: repr(value))),
  )
}