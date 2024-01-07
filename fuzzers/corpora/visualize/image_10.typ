
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// // Error: 14-168 failed to parse SVG (missing root node)
// #image.decode(`<svg height="140" width="500"><ellipse cx="200" cy="80" rx="100" ry="50" style="fill:yellow;stroke:purple;stroke-width:2" /></svg>`.text, format: "svg")