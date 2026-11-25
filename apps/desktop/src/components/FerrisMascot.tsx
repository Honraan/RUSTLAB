import React from 'react';
import { SafeRichText } from '../lib/safeRender';

export const FERRIS_FACES: Record<string, string> = {
  neutral: 'c(o.o)c',
  happy: 'c(ovo)c',
  error: 'c(o,x,o)c',
  thinking: 'c(o,0)c',
};

interface FerrisMascotProps {
  mood: string;
  ascii?: string;
  title?: string;
  message: string;
  compact?: boolean;
}

export function FerrisMascot({
  mood,
  ascii,
  title = 'Ferris',
  message,
  compact = false,
}: FerrisMascotProps): React.ReactElement {
  const face = ascii && ascii.trim().length > 0 ? ascii : (FERRIS_FACES[mood] ?? 'c(o.o)c');
  return (
    <div className="ferris-card" role="status" aria-live="polite">
      <div className="ferris-face" aria-label={`Ferris (${mood})`}>
        {face}
      </div>
      <div style={{ minWidth: 0, flex: 1 }}>
        {!compact ? <div style={{ fontWeight: 700, marginBottom: 2 }}>{title}</div> : null}
        <div style={{ color: 'var(--fg-secondary)', fontSize: '0.92rem' }}>
          <SafeRichText text={message} />
        </div>
      </div>
    </div>
  );
}
