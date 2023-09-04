
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: 250pt)

// Test simple quotations in various languages.
#set text(lang: "en")
"The horse eats no cucumber salad" was the first sentence ever uttered on the 'telephone.'

#set text(lang: "de")
"Das Pferd frisst keinen Gurkensalat" war der erste jemals am 'Fernsprecher' gesagte Satz.

#set text(lang: "de", region: "CH")
"Das Pferd frisst keinen Gurkensalat" war der erste jemals am 'Fernsprecher' gesagte Satz.

#set text(lang: "es", region: none)
"El caballo no come ensalada de pepino" fue la primera frase pronunciada por 'teléfono'.

#set text(lang: "es", region: "MX")
"El caballo no come ensalada de pepino" fue la primera frase pronunciada por 'teléfono'.

#set text(lang: "fr", region: none)
"Le cheval ne mange pas de salade de concombres" est la première phrase jamais prononcée au 'téléphone'.

#set text(lang: "fi")
"Hevonen ei syö kurkkusalaattia" oli ensimmäinen koskaan 'puhelimessa' lausuttu lause.

#set text(lang: "he")
"הסוס לא אוכל סלט מלפפונים" היה המשפט ההראשון שנאמר ב 'טלפון'.

#set text(lang: "ro")
"Calul nu mănâncă salată de castraveți" a fost prima propoziție rostită vreodată la 'telefon'.

#set text(lang: "ru")
"Лошадь не ест салат из огурцов" - это была первая фраза, сказанная по 'телефону'.
