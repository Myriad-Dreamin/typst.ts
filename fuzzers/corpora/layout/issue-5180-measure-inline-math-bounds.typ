
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#context {
  let height = measure(text(top-edge: "bounds", $x$)).height
  assert(height > 4pt)
  assert(height < 5pt)
}