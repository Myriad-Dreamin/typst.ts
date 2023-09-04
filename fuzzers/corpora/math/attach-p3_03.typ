
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Show and let rules for limits and scripts
#let eq = $ ∫_a^b iota_a^b $
#eq
#show "∫": math.limits
#show math.iota: math.limits.with(inline: false)
#eq
$iota_a^b$
