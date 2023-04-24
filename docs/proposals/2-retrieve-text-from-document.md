### Retrieve Text from Document

The text item can be retrieved from the document, which can replace the role of `page.getTextContent()`.

From declaration of pdf.js, we can see that `page.getTextContent()` returns a `TextContent` object, which contains an array of `TextItem` objects. The `TextItem` object contains the text string, the position of the text, and the font size.

```typescript
/**
 * Page text content.
 */
export type TextContent = {
    /**
     * - Array of
     * {@link TextItem } and {@link TextMarkedContent } objects. TextMarkedContent
     * items are included when includeMarkedContent is true.
     */
    items: Array<TextItem | TextMarkedContent>;
    /**
     * - {@link TextStyle } objects,
     * indexed by font name.
     */
    styles: {
        [x: string]: TextStyle;
    };
};
/**
 * Page text content part.
 */
export type TextItem = {
    /**
     * - Text content.
     */
    str: string;
    /**
     * - Text direction: 'ttb', 'ltr' or 'rtl'.
     */
    dir: string;
    /**
     * - Transformation matrix.
     */
    transform: Array<any>;
    /**
     * - Width in device space.
     */
    width: number;
    /**
     * - Height in device space.
     */
    height: number;
    /**
     * - Font name used by PDF.js for converted font.
     */
    fontName: string;
    /**
     * - Indicating if the text content is followed by a
     * line-break.
     */
    hasEOL: boolean;
};
/**
 * Page text marked content part.
 */
export type TextMarkedContent = {
    /**
     * - Either 'beginMarkedContent',
     * 'beginMarkedContentProps', or 'endMarkedContent'.
     */
    type: string;
    /**
     * - The marked content identifier. Only used for type
     * 'beginMarkedContentProps'.
     */
    id: string;
};
/**
 * Text style.
 */
export type TextStyle = {
    /**
     * - Font ascent.
     */
    ascent: number;
    /**
     * - Font descent.
     */
    descent: number;
    /**
     * - Whether or not the text is in vertical mode.
     */
    vertical: boolean;
    /**
     * - The possible font family.
     */
    fontFamily: string;
};
```

These information can be easily mapped from `typst::doc::TextItem`:
