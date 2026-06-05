
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test color '.components()' without conversions

#let test-components(col, ref, has-alpha: true) = {
  // Perform an approximate scalar comparison.
  let are-equal((a, b)) = {
    let to-float(x) = if type(x) == angle { x.rad() } else { float(x) }
    let epsilon = 1e-4 // The maximum error between both numbers
    test(type(a), type(b))
    calc.abs(to-float(a) - to-float(b)) < epsilon
  }

  let ref-without-alpha = if has-alpha { ref.slice(0, -1) } else { ref }
  test(col.components().len(), ref.len())
  assert(col.components().zip(ref).all(are-equal))
  assert(col.components(alpha: false).zip(ref-without-alpha).all(are-equal))
}
#test-components(rgb(1, 2, 3, 4), (0.39%, 0.78%, 1.18%, 1.57%))
#test-components(luma(40), (15.69%, 100%))
#test-components(luma(40, 50%), (15.69%, 50%))
#test-components(cmyk(4%, 5%, 6%, 7%), (4%, 5%, 6%, 7%), has-alpha: false)
#test-components(oklab(10%, 0.2, 0.4), (10%, 0.2, 0.4, 100%))
#test-components(oklch(10%, 0.2, 90deg), (10%, 0.2, 90deg, 100%))
#test-components(oklab(10%, 50%, 200%), (10%, 0.2, 0.8, 100%))
#test-components(oklch(10%, 50%, 90deg), (10%, 0.2, 90deg, 100%))
#test-components(color.linear-rgb(10%, 20%, 30%), (10%, 20%, 30%, 100%))
#test-components(color.hsv(10deg, 20%, 30%), (10deg, 20%, 30%, 100%))
#test-components(color.hsl(10deg, 20%, 30%), (10deg, 20%, 30%, 100%))