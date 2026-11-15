# Rule index

Every rule the linter enforces, grouped by category. Severity `error` means the server refuses the widget; `warn` means the client silently changes or removes behavior; `info` is advisory.

## CSS

| Rule | Severity | Confidence | Title |
|---|---|---|---|
| [TGW200](TGW200.md) | warn | certain | viewport height units are unsupported |
| [TGW201](TGW201.md) | warn | certain | overflow auto/scroll is rewritten to hidden |
| [TGW202](TGW202.md) | warn | certain | position sticky is rewritten to relative |
| [TGW203](TGW203.md) | warn | certain | logical properties are unsupported |
| [TGW204](TGW204.md) | warn | certain | multi-column layout is unsupported |
| [TGW205](TGW205.md) | warn | certain | this display value is unsupported |
| [TGW206](TGW206.md) | warn | certain | 3D transforms are unsupported |
| [TGW207](TGW207.md) | warn | certain | this filter function is unsupported |
| [TGW208](TGW208.md) | warn | certain | clipping, masking and blending are unsupported |
| [TGW209](TGW209.md) | warn | certain | this text effect is unsupported |
| [TGW210](TGW210.md) | warn | certain | modern color function is unsupported |
| [TGW211](TGW211.md) | warn | certain | env()/url() argument is unsupported |
| [TGW212](TGW212.md) | warn | certain | this at-rule or nested CSS is unsupported |
| [TGW213](TGW213.md) | warn | certain | this selector pseudo-class is unsupported |
| [TGW214](TGW214.md) | warn | certain | this @media feature is unsupported |
| [TGW215](TGW215.md) | warn | certain | named font families are not available |
| [TGW216](TGW216.md) | info | heuristic | property is not recognized by the CSS engine |

## Document limits

| Rule | Severity | Confidence | Title |
|---|---|---|---|
| [TGW300](TGW300.md) | error | certain | more than 400 CSS rules |
| [TGW301](TGW301.md) | error | certain | more than 32 @keyframes rules |
| [TGW302](TGW302.md) | error | certain | attribute or inline style longer than 8 K UTF-16 units |
| [TGW303](TGW303.md) | error | certain | more than 2000 DOM nodes |
| [TGW304](TGW304.md) | error | certain | text value over 32 K UTF-16 units or document over 48 K |

## HTML and SVG

| Rule | Severity | Confidence | Title |
|---|---|---|---|
| [TGW100](TGW100.md) | warn | certain | form controls are removed together with their content |
| [TGW101](TGW101.md) | info | certain | form/label degrade to plain boxes; form semantics do not work |
| [TGW102](TGW102.md) | warn | certain | media and embedding elements are removed |
| [TGW103](TGW103.md) | warn | certain | service elements are removed |
| [TGW104](TGW104.md) | error | certain | external <script src> never executes |
| [TGW105](TGW105.md) | warn | certain | <img> not from a data: URL shows a placeholder |
| [TGW106](TGW106.md) | warn | certain | data: image uses an unsupported type |
| [TGW108](TGW108.md) | warn | certain | legacy presentational element or attribute |
| [TGW109](TGW109.md) | warn | certain | unsupported interactive attribute |
| [TGW110](TGW110.md) | info | heuristic | unknown element will render as an inline box |
| [TGW111](TGW111.md) | warn | certain | <a href> with a scheme other than https: or #fragment is ignored |
| [TGW113](TGW113.md) | info | certain | aria-* attributes are kept but have no effect |
| [TGW114](TGW114.md) | info | heuristic | element implies text input or focus, but there is no keyboard |
| [TGW150](TGW150.md) | warn | certain | unsupported SVG element |

## JavaScript

| Rule | Severity | Confidence | Title |
|---|---|---|---|
| [TGW400](TGW400.md) | error | certain | network APIs are disabled in widgets |
| [TGW401](TGW401.md) | warn | certain | navigation APIs do not work in widgets |
| [TGW402](TGW402.md) | warn | certain | dialogs are unavailable in widgets |
| [TGW403](TGW403.md) | warn | certain | this navigator API is unavailable in widgets |
| [TGW404](TGW404.md) | warn | certain | this DOM or file API is unavailable in widgets |
| [TGW405](TGW405.md) | info | heuristic | focus/blur/select do nothing in widgets |
| [TGW406](TGW406.md) | warn | certain | this canvas feature is outside the profile |
| [TGW408](TGW408.md) | warn | certain | TelegramWidget.sendMessage does not exist |
| [TGW411](TGW411.md) | info | certain | deprecated TelegramWidget theme alias |

## Size

| Rule | Severity | Confidence | Title |
|---|---|---|---|
| [TGW001](TGW001.md) | error | certain | raw HTML exceeds 32768 bytes |
| [TGW002](TGW002.md) | error | certain | compressed size exceeds 4096 bytes |
| [TGW003](TGW003.md) | warn | certain | compressed size exceeds the soft budget |
| [TGW004](TGW004.md) | error | certain | invalid UTF-8 or a NUL character in the input |
| [TGW005](TGW005.md) | info | certain | compressed size is within 64 bytes of the hard limit |
