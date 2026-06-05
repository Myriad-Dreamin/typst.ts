
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#table(
	columns: 2,
	table.hline(stroke: 2pt + blue),
	table.header([*foo*], [*bar*]),
	table.hline(stroke: 1.5pt + red),
	table.cell(colspan: 2)[_asdf_],
	table.hline(stroke: 1.5pt + red),
	[a], [b],
	[c], [d],
)