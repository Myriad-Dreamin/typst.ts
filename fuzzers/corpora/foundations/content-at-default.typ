
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test .at() default values for content.
#test(auto, [a].at("doesn't exist", default: auto))