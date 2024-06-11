import { Controller } from "@hotwired/stimulus";
import { basicSetup } from "codemirror";
import { sql } from "postgresml-lang-sql";
import { python } from "@codemirror/lang-python";
import { javascript } from "@codemirror/lang-javascript";
import { rust } from "@codemirror/lang-rust";
import { cpp } from "@codemirror/lang-cpp";
import { json } from "@codemirror/lang-json";
import { EditorView, ViewPlugin, Decoration } from "@codemirror/view";
import { RangeSetBuilder, Facet } from "@codemirror/state";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";

import {
  highlightStyle,
  editorTheme,
} from "../../../static/js/utilities/code_mirror_theme";

const buildEditorView = (target, content, languageExtension, classes) => {
  let editorView = new EditorView({
    doc: content,
    extensions: [
      basicSetup,
      languageExtension !== null ? languageExtension() : [], // if no language chosen do not highlight syntax
      EditorView.theme(editorTheme),
      syntaxHighlighting(HighlightStyle.define(highlightStyle)),
      EditorView.contentAttributes.of({ contenteditable: false }),
      addClasses.of(classes),
      highlight,
    ],
    parent: target,
    highlightActiveLine: false,
  });
  return editorView;
};

const highlight = ViewPlugin.fromClass(
  class {
    constructor(view) {
      this.decorations = highlightLine(view);
    }

    update(update) {
      if (update.docChanged || update.viewportChanged)
        this.decorations = highlightLine(update.view);
    }
  },
  {
    decorations: (v) => v.decorations,
  },
);

function highlightLine(view) {
  let builder = new RangeSetBuilder();
  let classes = view.state.facet(addClasses).shift();
  for (let { from, to } of view.visibleRanges) {
    for (let pos = from; pos <= to; ) {
      let lineClasses = classes.shift();
      let line = view.state.doc.lineAt(pos);
      builder.add(
        line.from,
        line.from,
        Decoration.line({ attributes: { class: lineClasses } }),
      );
      pos = line.to + 1;
    }
  }
  return builder.finish();
}

const addClasses = Facet.define({
  combone: (values) => values,
});

const language = (element) => {
  switch (element.getAttribute("language")) {
    case "sql":
      return sql;
    case "postgresql":
      return sql;
    case "python":
      return python;
    case "javascript":
      return javascript;
    case "rust":
      return rust;
    case "json":
      return json;
    case "cpp":
      return cpp;
    default:
      return null;
  }
};

const codeBlockCallback = (element) => {
  let highlights = element.getElementsByClassName("highlight");
  let classes = [];
  for (let lineNum = 0; lineNum < highlights.length; lineNum++) {
    classes.push(highlights[lineNum].classList);
  }

  let content = element.textContent.trim();
  element.innerHTML = "";

  return [element, content, classes];
};

// Add Codemirror with data controller
export default class extends Controller {
  connect() {
    let [element, content, classes] = codeBlockCallback(this.element);
    let lang = language(this.element);

    buildEditorView(element, content, lang, classes);
  }
}

// Add Codemirror with web component
class CodeBlockA extends HTMLElement {
  constructor() {
    super();

    this.language = language(this);
  }

  connectedCallback() {
    let [element, content, classes] = codeBlockCallback(this);

    buildEditorView(element, content, this.language, classes);
  }

  // component attributes
  static get observedAttributes() {
    return ["type"];
  }

  // attribute change
  attributeChangedCallback(property, oldValue, newValue) {
    if (oldValue === newValue) return;
    this[property] = newValue;
  }
}
customElements.define("code-block", CodeBlockA);
