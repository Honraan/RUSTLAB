import React, { useState } from 'react';
import { Bot, ShieldAlert, Send, X } from 'lucide-react';
import { formatIpcError, ipc } from '../lib/ipc';
import { SafeRichText } from '../lib/safeRender';
import type { MentorAskRequest, MentorReplyDto } from '../lib/types';

interface MentorDrawerProps {
  open: boolean;
  context: Omit<MentorAskRequest, 'action' | 'question'> | null;
  onClose: () => void;
}

export function MentorDrawer({
  open,
  context,
  onClose,
}: MentorDrawerProps): React.ReactElement | null {
  const [question, setQuestion] = useState('');
  const [loading, setLoading] = useState(false);
  const [reply, setReply] = useState<MentorReplyDto | null>(null);
  const [error, setError] = useState<string | null>(null);

  if (!open || !context) return null;

  const ask = async (action: string, customQuestion?: string) => {
    setLoading(true);
    setError(null);
    try {
      const res = await ipc.mentorAsk({
        ...context,
        action,
        question: customQuestion ?? (question.trim() || 'Explique o que está acontecendo aqui.'),
      });
      setReply(res);
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="drawer-backdrop" role="dialog" aria-modal="true" aria-label="Mentor Pedagógico RustLab">
      <div className="drawer-panel stack">
        <div className="between">
          <div className="row">
            <Bot size={18} color="var(--accent)" />
            <h2 style={{ margin: 0 }}>Mentor Pedagógico</h2>
          </div>
          <button type="button" className="btn btn-sm" onClick={onClose}>
            <X size={15} /> Fechar
          </button>
        </div>

        <div className="card-muted">
          <div><strong>Origem:</strong> {context.origin}</div>
          {context.topicTitle ? <div><strong>Tópico:</strong> {context.topicTitle}</div> : null}
        </div>

        {context.learnMode ? (
          <div className="banner banner-warn">
            <ShieldAlert size={17} />
            <span>
              <strong>Modo Aprender (Somente Leitura):</strong> o Mentor explica conceitos e erros
              sem entregar a solução pronta nem editar seu editor.
            </span>
          </div>
        ) : null}

        <div className="row">
          <button type="button" className="btn btn-sm" disabled={loading} onClick={() => void ask('explain-concept')}>
            Explicar conceito
          </button>
          <button type="button" className="btn btn-sm" disabled={loading} onClick={() => void ask('explain-error')}>
            Explicar erro
          </button>
          <button type="button" className="btn btn-sm" disabled={loading} onClick={() => void ask('next-hint')}>
            Próximo passo
          </button>
          <button type="button" className="btn btn-sm" disabled={loading} onClick={() => void ask('review-code')}>
            Revisar meu código
          </button>
        </div>

        <div className="stack">
          <label className="field-label" htmlFor="mentor-q">Sua pergunta</label>
          <textarea
            id="mentor-q"
            className="input"
            rows={3}
            placeholder="Ex: Por que o compilador disse que o valor foi movido?"
            value={question}
            onChange={(e) => setQuestion(e.target.value)}
          />
          <button
            type="button"
            className="btn btn-primary"
            disabled={loading}
            onClick={() => void ask('custom', question)}
          >
            <Send size={15} /> {loading ? 'Consultando mentor...' : 'Perguntar'}
          </button>
        </div>

        {error ? <div className="banner banner-danger">{error}</div> : null}

        {reply ? (
          <div className="card stack">
            <div className="between">
              <span className="pill pill-info">{reply.source}</span>
              {reply.readOnlyEnforced ? (
                <span className="pill pill-ok">Somente leitura ativo</span>
              ) : null}
            </div>
            <div>
              <SafeRichText text={reply.body} />
            </div>
            {!reply.readOnlyEnforced && reply.proposedCode ? (
              <pre className="code-block">{reply.proposedCode}</pre>
            ) : null}
            <span className="muted" style={{ fontSize: '0.82rem' }}>{reply.probabilisticNotice}</span>
          </div>
        ) : null}
      </div>
    </div>
  );
}
