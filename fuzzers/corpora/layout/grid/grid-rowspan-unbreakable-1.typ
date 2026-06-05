
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#table(
    columns: 3,
    rows: (auto, auto, auto, 2em),
    gutter: 3pt,
    table.cell(rowspan: 4)[a \ b\ c\ d\ e], [c], [d],
    [e], table.cell(breakable: false, rowspan: 2)[f],
    [g]
)