// lib/sounds.ts — Toggle-able sound effects.
//
// Sound files should be placed in frontend/static/sounds/.
// The AudioContext is created lazily on first user interaction (browser requirement).

let enabled = $state(
	typeof localStorage !== 'undefined' ? localStorage.getItem('sounds') !== 'off' : true
);

export function getSoundsEnabled() {
	return enabled;
}

export function toggleSounds() {
	enabled = !enabled;
	if (typeof localStorage !== 'undefined') {
		localStorage.setItem('sounds', enabled ? 'on' : 'off');
	}
}

let audioContext: AudioContext | null = null;

function getContext(): AudioContext {
	if (!audioContext) {
		audioContext = new AudioContext();
	}
	return audioContext;
}

// Simple beep using Web Audio API (no external files needed)
export function playVictory() {
	if (!enabled) return;
	try {
		const ctx = getContext();
		const osc = ctx.createOscillator();
		const gain = ctx.createGain();
		osc.connect(gain);
		gain.connect(ctx.destination);
		gain.gain.value = 0.1;

		// Victory fanfare: three ascending notes
		osc.frequency.setValueAtTime(523, ctx.currentTime); // C5
		osc.frequency.setValueAtTime(659, ctx.currentTime + 0.15); // E5
		osc.frequency.setValueAtTime(784, ctx.currentTime + 0.3); // G5
		gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.6);

		osc.start(ctx.currentTime);
		osc.stop(ctx.currentTime + 0.6);
	} catch {
		// Audio not available
	}
}

export function playSadTrombone() {
	if (!enabled) return;
	try {
		const ctx = getContext();
		const osc = ctx.createOscillator();
		const gain = ctx.createGain();
		osc.connect(gain);
		gain.connect(ctx.destination);
		gain.gain.value = 0.08;
		osc.type = 'sawtooth';

		// Sad trombone: descending notes
		osc.frequency.setValueAtTime(311, ctx.currentTime); // Eb4
		osc.frequency.setValueAtTime(293, ctx.currentTime + 0.3); // D4
		osc.frequency.setValueAtTime(277, ctx.currentTime + 0.6); // Db4
		osc.frequency.setValueAtTime(261, ctx.currentTime + 0.9); // C4
		gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 1.3);

		osc.start(ctx.currentTime);
		osc.stop(ctx.currentTime + 1.3);
	} catch {
		// Audio not available
	}
}
