
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#metadata(none)<a>A#metadata(none)<b>B

// The first metadata has its end tag before the paragraph, so it does not
// become part of the paragraph and thus its Y position is determined by the
// flow.
#context assert(
  locate(<a>).position().y < locate(<b>).position().y
)

// The first footnote becomes part of the paragraph. Thus, its Y position is
// determined by inline layout.
#footnote[c]<c>C#footnote[d]<d>D

#context test(
  locate(<c>).position().y,
  locate(<d>).position().y,
)