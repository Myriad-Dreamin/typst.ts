
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test horizontal stretch interactions with attachments.
#set page(width: auto)

$stretch(stretch(=, size: #4em))_A$
$stretch(arrow.hook, size: #5em)^"injective map"$
$stretch(arrow.hook, size: #200%)^"injective map"$

$ P = Q
    stretch(=)^(k = 0)_(forall i) R
    stretch(=, size: #150%)^(k = 0)_(forall i) S
    stretch(=, size: #2mm)^(k = 0)_(forall i) T \
  U stretch(equiv)^(forall i)_"Chern-Weil" V
    stretch(equiv, size: #(120% + 2mm))^(forall i)_"Chern-Weil" W $