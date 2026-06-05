
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
Hidden:
#hide(table(rows: 2, columns: 2)[a][b][c][d])
#table(rows: 2, columns: 2)[a][b][c][d]