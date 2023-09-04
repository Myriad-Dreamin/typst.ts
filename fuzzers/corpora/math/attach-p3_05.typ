
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test default of scripts attachments on integrals at display size
$ integral.sect_a^b  quad \u{2a1b}_a^b quad limits(\u{2a1b})_a^b $
$integral.sect_a^b quad \u{2a1b}_a^b quad limits(\u{2a1b})_a^b$
