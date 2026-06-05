
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test quoting a query.

#quote[ABC] & #quote[EFG]

#context query(selector(quote).before(here())).first()

#quote(block: true)[HIJ]
#quote(block: true)[KLM]

#context query(selector(quote).before(here())).last()

#quote[NOP] <nop>

#context query(<nop>).first()