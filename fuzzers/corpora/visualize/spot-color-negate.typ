
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test negate on spot colors.
#let pantone = color.spot("PANTONE 185 C", rgb(89.4%, 0.7%, 17%))
#let base = pantone.tint(70%)
#let neg = base.negate()
#let eps = 0.0001 * 100%
#let components = neg.components()
#if components.len() != 1 { panic("expected 1 component, got " + components.len()) }
#if calc.abs(components.at(0) - 30%) > eps { panic("spot color negation failed") }