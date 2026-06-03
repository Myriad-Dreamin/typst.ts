
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test evaluation in other modes.
#eval("[_Hello" + " World!_]") \
#eval("_Hello" + " World!_", mode: "markup") \
#eval("RR_1^NN", mode: "math", scope: (RR: math.NN, NN: math.RR))