
#let test(lhs, rhs) = {
  assert(lhs == rhs, message: "Expected \(lhs) to equal \(rhs)")
}

#let print(..args) = {}

#let conifer = rgb("9feb52")
#let forest = rgb("43a127")

#let test-page(content) = {
  set page(width: 120pt, height: auto, margin: 10pt)
  set text(size: 10pt)

  content
}
