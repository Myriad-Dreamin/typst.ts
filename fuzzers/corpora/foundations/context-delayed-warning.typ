
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that the warning that triggers in the first layout iteration is not
// surfaced since it goes away in the second one. Just like errors in show
// rules.
#show heading: none

= A <a>
#context {
  let n = query(<a>).len()
  let fonts = ("nope", "Roboto")
  set text(font: fonts.at(n))
}