
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "../loading/edge-case.typ": large-integer, representable-integer

#for (name, source) in representable-integer {
  if name == "i64-min" {
    // i64-min will be parsed as a float
    assert.eq(
      type(eval(source)),
      float,
      message: "failed to approximately parse " + name,
    )
    // but can still be obtained through integer arithmetic
    let n = -1 - eval(representable-integer.i64-max)
    assert(
      type(n) == int and n < 0,
      message: "failed to obtained i64-min through integer arithmetic",
    )
  } else {
    assert.eq(
      type(eval(source)),
      int,
      message: "failed to parse " + name,
    )
  }
}

#for (name, source) in large-integer {
  assert.eq(
    type(eval(source)),
    float,
    message: "failed to approximately parse " + name,
  )
}