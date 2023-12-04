
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Basic.
_Emphasized and *strong* words!_

// Inside of a word it's a normal underscore or star.
hello_world Nutzer*innen

// CJK characters will not need spaces.
中文一般使用*粗体*或者_楷体_来表示强调。

日本語では、*太字*や_斜体_を使って強調します。

中文中混有*Strong*和_Empasis_。

// Can contain paragraph in nested content block.
_Still #[

] emphasized._
