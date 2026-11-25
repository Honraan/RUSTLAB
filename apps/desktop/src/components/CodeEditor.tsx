import React, { useEffect, useRef } from 'react';
import { EditorState, Compartment } from '@codemirror/state';
import {
  EditorView,
  keymap,
  lineNumbers,
  highlightActiveLine,
  highlightActiveLineGutter,
} from '@codemirror/view';
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
import { rust } from '@codemirror/lang-rust';
import {
  bracketMatching,
  defaultHighlightStyle,
  indentOnInput,
  syntaxHighlighting,
} from '@codemirror/language';
import { autocompletion, type CompletionContext } from '@codemirror/autocomplete';
import { linter, lintGutter, type Diagnostic as CmDiagnostic } from '@codemirror/lint';
import { searchKeymap } from '@codemirror/search';
import { oneDark } from '@codemirror/theme-one-dark';
import type { DiagnosticDto } from '../lib/types';

interface CodeEditorProps {
  value: string;
  onChange: (next: string) => void;
  dark: boolean;
  diagnostics?: DiagnosticDto[];
  completions?: string[];
  readOnly?: boolean;
  ariaLabel?: string;
  minHeight?: number;
  onRunShortcut?: () => void;
  onSaveShortcut?: () => void;
  onHintShortcut?: () => void;
  onLibrariesShortcut?: () => void;
}

export function CodeEditor({
  value,
  onChange,
  dark,
  diagnostics = [],
  completions = [],
  readOnly = false,
  ariaLabel = 'Editor de código Rust',
  minHeight = 250,
  onRunShortcut,
  onSaveShortcut,
  onHintShortcut,
  onLibrariesShortcut,
}: CodeEditorProps): React.ReactElement {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const viewRef = useRef<EditorView | null>(null);
  const themeComp = useRef(new Compartment());
  const lintComp = useRef(new Compartment());
  const compComp = useRef(new Compartment());
  const callbacksRef = useRef({
    onChange,
    onRunShortcut,
    onSaveShortcut,
    onHintShortcut,
    onLibrariesShortcut,
  });
  callbacksRef.current = {
    onChange,
    onRunShortcut,
    onSaveShortcut,
    onHintShortcut,
    onLibrariesShortcut,
  };

  const makeLinter = (diags: DiagnosticDto[]) =>
    linter((view) => {
      const result: CmDiagnostic[] = [];
      const doc = view.state.doc;
      for (const d of diags) {
        const lineNum = Math.min(Math.max(1, d.line ?? 1), doc.lines);
        const line = doc.line(lineNum);
        const colOffset = Math.max(0, (d.col ?? 1) - 1);
        const from = Math.min(line.from + colOffset, line.to);
        const to = Math.min(Math.max(from + 1, line.to), doc.length);
        result.push({
          from,
          to: Math.max(from, to),
          severity: d.level === 'error' ? 'error' : d.level === 'warning' ? 'warning' : 'info',
          message: d.message,
        });
      }
      return result;
    });

  const makeCompletion = (items: string[]) =>
    autocompletion({
      override: [
        (context: CompletionContext) => {
          const word = context.matchBefore(/\w*/);
          if (!word || (word.from === word.to && !context.explicit)) return null;
          const rustBuiltins = [
            'println!',
            'eprintln!',
            'format!',
            'vec!',
            'let mut',
            'match',
            'Result',
            'Option',
            'Some',
            'None',
            'Ok',
            'Err',
            'String::from',
            'unwrap_or',
          ];
          const merged = Array.from(new Set([...items, ...rustBuiltins]));
          return {
            from: word.from,
            options: merged.map((label) => ({ label, type: 'keyword' })),
          };
        },
      ],
    });

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    const customKeys = keymap.of([
      {
        key: 'F5',
        run: () => {
          callbacksRef.current.onRunShortcut?.();
          return Boolean(callbacksRef.current.onRunShortcut);
        },
      },
      {
        key: 'Mod-Enter',
        run: () => {
          callbacksRef.current.onRunShortcut?.();
          return Boolean(callbacksRef.current.onRunShortcut);
        },
      },
      {
        key: 'Mod-s',
        run: () => {
          callbacksRef.current.onSaveShortcut?.();
          return Boolean(callbacksRef.current.onSaveShortcut);
        },
      },
      {
        key: 'F1',
        run: () => {
          callbacksRef.current.onHintShortcut?.();
          return Boolean(callbacksRef.current.onHintShortcut);
        },
      },
      {
        key: 'F9',
        run: () => {
          callbacksRef.current.onLibrariesShortcut?.();
          return Boolean(callbacksRef.current.onLibrariesShortcut);
        },
      },
    ]);

    const state = EditorState.create({
      doc: value,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightActiveLine(),
        history(),
        bracketMatching(),
        indentOnInput(),
        rust(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        lintGutter(),
        themeComp.current.of(dark ? oneDark : []),
        lintComp.current.of(makeLinter(diagnostics)),
        compComp.current.of(makeCompletion(completions)),
        EditorState.readOnly.of(readOnly),
        EditorView.contentAttributes.of({ 'aria-label': ariaLabel }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            callbacksRef.current.onChange(update.state.doc.toString());
          }
        }),
        customKeys,
        keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap, indentWithTab]),
      ],
    });

    const view = new EditorView({ state, parent: host });
    viewRef.current = view;
    return () => {
      view.destroy();
      viewRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [readOnly, ariaLabel]);

  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const current = view.state.doc.toString();
    if (current !== value) {
      view.dispatch({
        changes: { from: 0, to: current.length, insert: value },
      });
    }
  }, [value]);

  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({
      effects: themeComp.current.reconfigure(dark ? oneDark : []),
    });
  }, [dark]);

  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({
      effects: lintComp.current.reconfigure(makeLinter(diagnostics)),
    });
  }, [diagnostics]);

  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({
      effects: compComp.current.reconfigure(makeCompletion(completions)),
    });
  }, [completions]);

  return (
    <div
      ref={hostRef}
      className="cm-host"
      style={{ minHeight, height: '100%' }}
      data-testid="code-editor"
    />
  );
}
