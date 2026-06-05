
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test a mix of alignment and fr units (fr wins).
#set page(height: 80pt)
A
#v(1fr)
B
#align(bottom + right)[C]