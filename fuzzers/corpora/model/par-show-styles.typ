
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Variant 2: Prevent recursion by observing a style.
#let revoke = metadata("revoke")
#show par: it => {
  if bibliography.title == revoke { return it }
  set bibliography(title: revoke)
  let p = counter("p")
  par[#p.step()§#context p.display() #it.body]
}

= A

B

C