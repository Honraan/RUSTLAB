// Reproduz sinal sonoro curto (110ms @ 660Hz->880Hz) via Web Audio API sem depender de arquivos externos.
export function playXpCue(enabled: boolean, volumePercent: number, focusMode: boolean): void {
  if (!enabled || focusMode || volumePercent <= 0) {
    return;
  }
  try {
    const AudioCtx =
      window.AudioContext ||
      (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!AudioCtx) {
      return;
    }
    const ctx = new AudioCtx();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    const vol = Math.min(1, Math.max(0.02, (volumePercent / 100) * 0.18));

    osc.type = 'sine';
    osc.frequency.setValueAtTime(660, ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(880, ctx.currentTime + 0.09);

    gain.gain.setValueAtTime(vol, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.11);

    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start();
    osc.stop(ctx.currentTime + 0.115);
    osc.onended = () => {
      void ctx.close().catch(() => {});
    };
  } catch {
    // Hardware de áudio ausente ou bloqueado no ambiente: degrada silenciosamente.
  }
}
