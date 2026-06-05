
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 100pt, columns: 3)
#set place(float: true, clearance: 10pt)
#set rect(width: 70%)

// The most important part of this test is that we get the backlog of the
// conifer (green) block right.
#place(top + center, scope: "parent", rect[I])
#block(fill: aqua, width: 100%, height: 70pt)
#block(fill: conifer, width: 100%, height: 160pt)
#place(bottom + center, scope: "parent", rect[II])
#place(top, rect(height: 40%)[III])
#block(fill: yellow, width: 100%, height: 60pt)