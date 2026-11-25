import { EditorState } from "@codemirror/state";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags } from "@lezer/highlight";
import { EditorView } from "@codemirror/view";
import { rust } from "@codemirror/lang-rust";
import { basicSetup } from "codemirror";
import { useEffect, useRef } from "react";
import type { Jump } from "./api";

const theme = EditorView.theme({
  "&": { minHeight: "17rem", backgroundColor: "var(--editor)" },
  ".cm-scroller": { overflow: "auto", fontFamily: "var(--mono)" },
  ".cm-content": { color: "var(--ink)", minHeight: "17rem" },
  ".cm-gutters": { backgroundColor: "var(--editor)", color: "var(--muted)", borderRight: "1px solid var(--line)" },
  ".cm-activeLine, .cm-activeLineGutter": { backgroundColor: "var(--line)" },
  ".cm-cursor": { borderLeftColor: "var(--ink)" },
  ".cm-selectionBackground": { backgroundColor: "var(--selection) !important" },
});

const highlighter = HighlightStyle.define([
  { tag: [tags.keyword, tags.operatorKeyword], color: "var(--accent)" },
  { tag: [tags.string, tags.special(tags.string), tags.character], color: "var(--success)" },
  { tag: [tags.comment, tags.lineComment, tags.blockComment], color: "var(--muted)", fontStyle: "italic" },
  { tag: [tags.number, tags.bool, tags.typeName, tags.atom], color: "var(--warn)" },
]);

export function Editor({ label, value, onChange, language, fontSize, onSave, jump }: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  language: "rust" | "text";
  fontSize: number;
  onSave: () => void;
  jump?: Jump | null;
}) {
  const mount = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const callbacks = useRef({ onChange, onSave, value, label });
  callbacks.current = { onChange, onSave, value, label };

  useEffect(() => {
    if (!mount.current) return;
    const editor = new EditorView({
      parent: mount.current,
      state: EditorState.create({
        doc: callbacks.current.value,
        extensions: [
          basicSetup, theme, syntaxHighlighting(highlighter), EditorView.lineWrapping,
          ...(language === "rust" ? [rust()] : []),
          EditorView.contentAttributes.of({ "aria-label": callbacks.current.label }),
          EditorView.domEventHandlers({
            keydown(event) {
              if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
                event.preventDefault();
                callbacks.current.onSave();
                return true;
              }
              return false;
            },
          }),
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              const text = update.state.doc.toString();
              if (text !== callbacks.current.value.replace(/\r\n/g, "\n")) callbacks.current.onChange(text);
            }
          }),
        ],
      }),
    });
    view.current = editor;
    return () => {
      editor.destroy();
      view.current = null;
    };
  }, [language]);

  useEffect(() => {
    const editor = view.current;
    if (editor && value.replace(/\r\n/g, "\n") !== editor.state.doc.toString()) {
      editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: value } });
    }
  }, [value]);

  useEffect(() => {
    const editor = view.current;
    if (!editor || !jump) return;
    const line = editor.state.doc.line(Math.max(1, Math.min(jump.line, editor.state.doc.lines)));
    const column = [...line.text].slice(0, Math.max(0, jump.column - 1)).join("").length;
    const anchor = line.from + column;
    editor.dispatch({ selection: { anchor }, effects: EditorView.scrollIntoView(anchor, { y: "center" }) });
    editor.focus();
  }, [jump]);

  return <div className="editor" style={{ fontSize }} ref={mount} />;
}
