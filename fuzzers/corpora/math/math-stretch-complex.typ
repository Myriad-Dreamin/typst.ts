
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test complex stretch.
$ H stretch(=)^"define" U + p V \
  x stretch(harpoons.ltrb, size: #3em) y
    stretch(\[, size: #150%) z \
  f : X stretch(arrow.hook, size: #150%)_"injective" Y \
  V stretch(->, size: #(100% + 1.5em))^("surjection") ZZ $