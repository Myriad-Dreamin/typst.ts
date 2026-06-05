
#let test(lhs, rhs) = {
  assert(lhs == rhs, message: "Expected \(lhs) to equal \(rhs)")
}

#let test-repr(lhs, rhs) = {
  assert(repr(lhs) == repr(rhs), message: "Expected \(repr(lhs)) to equal \(repr(rhs))")
}

#let print(..args) = {}

#let lines(count, ..args) = {
  let pattern = args.named().at("pattern", default: args.pos().at(0, default: "A"))

  for n in range(1, count + 1) {
    numbering(pattern, n)
    if n < count {
      linebreak()
    }
  }
}

#let conifer = rgb("9feb52")
#let forest = rgb("43a127")

#let test-page(content) = {
  set page(width: 120pt, height: auto, margin: 10pt)
  set text(size: 10pt)

  content
}
