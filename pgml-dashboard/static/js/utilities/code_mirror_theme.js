import { tags as t } from "@lezer/highlight";

// Theme builder is taken from: https://github.com/codemirror/theme-one-dark#readme

const chalky = "#FF0"; // Set
const coral = "#F5708B"; // Set
const salmon = "#e9467a";
const blue = "#00e0ff";
const cyan = "#56b6c2";
const invalid = "#ffffff";
const ivory = "#abb2bf";
const stone = "#7d8799";
const malibu = "#61afef";
const sage = "#0F0"; // Set
const whiskey = "#ffb500";
const violet = "#F3F"; // Set
const darkBackground = "#17181A"; // Set
const highlightBackground = "#2c313a";
const background = "#17181A"; // Set
const tooltipBackground = "#353a42";
const selection = "#3E4451";
const cursor = "#528bff";

const editorTheme = {
  "&": {
    color: ivory,
    backgroundColor: background,
  },

  ".cm-content": {
    caretColor: cursor,
    paddingBottom: '1rem',
  },

  ".cm-cursor, .cm-dropCursor": { borderLeftColor: cursor },
  "&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection":
    { backgroundColor: selection },

  ".cm-panels": { backgroundColor: darkBackground, color: ivory },
  ".cm-panels.cm-panels-top": { borderBottom: "2px solid black" },
  ".cm-panels.cm-panels-bottom": { borderTop: "2px solid black" },

  ".cm-searchMatch": {
    backgroundColor: "#72a1ff59",
    outline: "1px solid #457dff",
  },
  ".cm-searchMatch.cm-searchMatch-selected": {
    backgroundColor: "#6199ff2f",
  },

  ".cm-activeLine": { backgroundColor: "#6699ff0b" },
  ".cm-selectionMatch": { backgroundColor: "#aafe661a" },

  "&.cm-focused .cm-matchingBracket, &.cm-focused .cm-nonmatchingBracket": {
    backgroundColor: "#bad0f847",
  },

  ".cm-gutters": {
    backgroundColor: background,
    color: stone,
    border: "none",
  },

  ".cm-activeLineGutter": {
    backgroundColor: highlightBackground,
  },

  ".cm-foldPlaceholder": {
    backgroundColor: "transparent",
    border: "none",
    color: "#ddd",
  },

  ".cm-tooltip": {
    border: "none",
    backgroundColor: tooltipBackground,
  },
  ".cm-tooltip .cm-tooltip-arrow:before": {
    borderTopColor: "transparent",
    borderBottomColor: "transparent",
  },
  ".cm-tooltip .cm-tooltip-arrow:after": {
    borderTopColor: tooltipBackground,
    borderBottomColor: tooltipBackground,
  },
  ".cm-tooltip-autocomplete": {
    "& > ul > li[aria-selected]": {
      backgroundColor: highlightBackground,
      color: ivory,
    },
  },
}

const highlightStyle = [
  { tag: [
      t.keyword,
      t.annotation,
      t.modifier,
      t.special(t.string),
      t.operatorKeyword,
    ],
    color: violet
  },
  {
    tag: [t.name, t.propertyName, t.deleted, t.character, t.macroName, t.function(t.variableName)],
    color: blue,
  },
  {
    tag: [],
    color: cyan,
  },
  { tag: [t.labelName], color: whiskey },
  { tag: [t.color, t.constant(t.name), t.standard(t.name)], color: whiskey },
  { tag: [t.definition(t.name), t.separator], color: ivory },
  {
    tag: [
      t.typeName,
      t.className,
      t.number,
      t.changed,
      t.self,
      t.namespace,
      t.bool,
    ],
    color: chalky,
  },
  { tag: [t.operator], color: whiskey },
  { tag: [
      t.processingInstruction,
      t.string,
      t.inserted,
      t.url,
      t.escape,
      t.regexp,
      t.link,
    ],
    color: sage
  },
  { tag: [t.meta, t.comment], color: stone },
  { tag: t.strong, fontWeight: "bold" },
  { tag: t.emphasis, fontStyle: "italic" },
  { tag: t.strikethrough, textDecoration: "line-through" },
  { tag: t.link, color: stone, textDecoration: "underline" },
  { tag: t.heading, fontWeight: "bold", color: salmon },
  { tag: [t.atom, t.special(t.variableName)], color: whiskey },
  { tag: t.invalid, color: invalid },
]


export  {highlightStyle, editorTheme};
