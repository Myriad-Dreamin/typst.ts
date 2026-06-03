
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Currently, presentation selectors do not cause font fallback when the main
// font supports at least one presentation, instead causing a fallback of the
// presentation form. This should probably be solved at some point, making the
// emojis below render with an emoji form.
// See: https://github.com/typst/typst/pull/6875.
#sym.copyright #emoji.copyright \
#sym.suit.heart #emoji.suit.heart