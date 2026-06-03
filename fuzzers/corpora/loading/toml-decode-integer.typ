
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "edge-case.typ": representable-integer

#for (name, source) in representable-integer {
  assert.eq(
    // The `key` trick is necessary because a TOML documents must be a table.
    type(toml(bytes("key = " + source)).key),
    int,
    message: "failed to decode " + name,
  )
}