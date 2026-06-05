
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#table(
	columns: 2,
	table.hline(stroke: 2pt + blue),
	table.footer([*foo*], [*bar*]),
	table.hline(stroke: 8pt),
)