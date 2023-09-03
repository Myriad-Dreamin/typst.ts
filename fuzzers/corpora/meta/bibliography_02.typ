
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test unconventional order.
#set page(width: 200pt)
#bibliography(
  "/assets/files/works.bib",
  title: [Works to be cited],
  style: "chicago-author-date",
)
#line(length: 100%)

#[#set cite(brackets: false)
As described by @netwok],
the net-work is a creature of its own.
This is close to piratery! @arrgh
And quark! @quark
