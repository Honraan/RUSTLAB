import React from 'react';

interface InlineSegment {
  kind: 'text' | 'code' | 'strong';
  value: string;
}

function splitInlineMarkdown(text: string): InlineSegment[] {
  const segments: InlineSegment[] = [];
  const tokenRe = /(`[^`]+`|\*\*[^*]+\*\*)/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = tokenRe.exec(text)) !== null) {
    if (match.index > lastIndex) {
      segments.push({ kind: 'text', value: text.slice(lastIndex, match.index) });
    }
    const raw = match[0];
    if (raw.startsWith('`') && raw.endsWith('`')) {
      segments.push({ kind: 'code', value: raw.slice(1, -1) });
    } else if (raw.startsWith('**') && raw.endsWith('**')) {
      segments.push({ kind: 'strong', value: raw.slice(2, -2) });
    }
    lastIndex = tokenRe.lastIndex;
  }
  if (lastIndex < text.length) {
    segments.push({ kind: 'text', value: text.slice(lastIndex) });
  }
  return segments;
}

/**
 * Renderiza texto com marcações seguras (`código` e **negrito**) usando apenas nós React
 * (nunca usa dangerouslySetInnerHTML).
 */
export function SafeRichText({ text }: { text: string }): React.ReactElement {
  const lines = text.split('\n');
  return (
    <>
      {lines.map((line, li) => (
        <React.Fragment key={li}>
          {splitInlineMarkdown(line).map((seg, si) => {
            if (seg.kind === 'code') {
              return (
                <code key={si} className="inline-code">
                  {seg.value}
                </code>
              );
            }
            if (seg.kind === 'strong') {
              return <strong key={si}>{seg.value}</strong>;
            }
            return <React.Fragment key={si}>{seg.value}</React.Fragment>;
          })}
          {li < lines.length - 1 ? <br /> : null}
        </React.Fragment>
      ))}
    </>
  );
}

interface AnsiSpan {
  text: string;
  classes: string[];
}

const ANSI_SGR_RE = /\x1b\[([0-9;]*)m/g;

function applySgrCodes(codes: number[], active: Set<string>): void {
  if (codes.length === 0 || codes.includes(0)) {
    active.clear();
  }
  for (const code of codes) {
    if (code === 1) active.add('ansi-bold');
    else if (code === 31 || code === 91) active.add('ansi-red');
    else if (code === 32 || code === 92) active.add('ansi-green');
    else if (code === 33 || code === 93) active.add('ansi-yellow');
    else if (code === 36 || code === 96) active.add('ansi-cyan');
    else if (code === 39) {
      active.delete('ansi-red');
      active.delete('ansi-green');
      active.delete('ansi-yellow');
      active.delete('ansi-cyan');
    }
  }
}

export function parseAnsiLine(line: string): AnsiSpan[] {
  const spans: AnsiSpan[] = [];
  const active = new Set<string>();
  let last = 0;
  let m: RegExpExecArray | null;
  ANSI_SGR_RE.lastIndex = 0;
  while ((m = ANSI_SGR_RE.exec(line)) !== null) {
    if (m.index > last) {
      spans.push({ text: line.slice(last, m.index), classes: [...active] });
    }
    const rawCodes = m[1]
      .split(';')
      .filter((s) => s.length > 0)
      .map((s) => Number.parseInt(s, 10))
      .filter((n) => !Number.isNaN(n));
    applySgrCodes(rawCodes, active);
    last = ANSI_SGR_RE.lastIndex;
  }
  if (last < line.length) {
    spans.push({ text: line.slice(last), classes: [...active] });
  }
  return spans;
}

/**
 * Renderiza linhas ANSI de terminal/Cargo escapando HTML como texto puro React.
 */
export function SafeAnsiOutput({ lines }: { lines: string[] }): React.ReactElement {
  return (
    <div className="terminal-box" role="log" aria-live="polite">
      {lines.length === 0 ? (
        <span className="muted">Sem saída ainda.</span>
      ) : (
        lines.map((line, idx) => (
          <div key={idx}>
            {parseAnsiLine(line).map((span, sidx) =>
              span.classes.length > 0 ? (
                <span key={sidx} className={span.classes.join(' ')}>
                  {span.text}
                </span>
              ) : (
                <React.Fragment key={sidx}>{span.text}</React.Fragment>
              ),
            )}
          </div>
        ))
      )}
    </div>
  );
}
