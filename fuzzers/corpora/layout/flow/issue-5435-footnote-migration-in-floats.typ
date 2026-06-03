
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that a footnote should not prompt migration when in a float that was
// queued to the next page (due to the float being too large), even if the
// footnote does not fit, breaking the footnote invariant.
#set page(height: 50pt)

#place(
  top,
  float: true,
  {
    v(100pt)
    footnote[a]
  }
)
#place(
  top,
  float: true,
  footnote[b]
)