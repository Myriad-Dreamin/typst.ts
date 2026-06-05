
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
|longscript| \
|#super(typographic: true)[longscript]| \
|#super(typographic: false)[longscript]| \
|#sub(typographic: true)[longscript]| \
|#sub(typographic: false)[longscript]|