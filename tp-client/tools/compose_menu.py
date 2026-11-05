from pathlib import Path
import subprocess
import wave
import numpy as np

root = Path(__file__).resolve().parents[1]
rate = 22050
beat = 60 / 66
length = round(32 * beat * rate)
track = np.zeros(length, dtype=np.float64)
rng = np.random.default_rng(20261102)

def mix(signal, start):
    indices = (np.arange(len(signal)) + round(start * rate)) % length
    np.add.at(track, indices, signal)

def note(pitch, start, seconds, gain, piano=True):
    t = np.arange(round(seconds * rate)) / rate
    frequency = 440 * 2 ** ((pitch - 69) / 12)
    attack = np.minimum(1, t / (.012 if piano else .7))
    release = np.minimum(1, np.maximum(0, seconds - t) / (.4 if piano else 1.1))
    sound = np.zeros(len(t))
    for harmonic in range(1, 5 if piano else 3):
        decay = np.exp(-t / (3.1 / harmonic ** .5)) if piano else np.ones(len(t))
        sound += np.sin(2 * np.pi * frequency * harmonic * t + .03 * np.sin(t * 1.8)) * decay / harmonic ** 2.1
    mix(sound * attack * release * gain, start)

chords = ((55, 60, 63, 67), (52, 59, 62, 66), (50, 57, 60, 65), (54, 58, 61, 66))
for index, chord in enumerate(chords):
    start = index * 8 * beat
    for pitch in chord:
        note(pitch - 12, start, 9 * beat, .021, False)
    for position, pitch in zip((0, 2.75, 4.5, 6.25), chord):
        note(pitch, start + position * beat, 5.3, .09)

melody = ((0.5, 79), (3, 75), (5.25, 72), (7, 74), (9.5, 78), (12, 74), (15, 71), (17, 77), (19.5, 72), (22.25, 69), (25.5, 73), (28, 78), (30.75, 70))
for position, pitch in melody:
    note(pitch, position * beat, 5.6, .057)

for index in range(16):
    t = np.arange(round(.16 * rate)) / rate
    noise = rng.normal(0, 1, len(t))
    brushed = np.diff(noise, prepend=0) * np.exp(-t * 36) * .0022
    mix(brushed, (index * 2 + 1.5) * beat)

dry = track.copy()
for seconds, gain in ((.23, .17), (.47, .12), (.91, .08), (1.63, .05)):
    track += np.roll(dry, round(seconds * rate)) * gain
left = np.tanh(track * 1.3)
right = np.tanh((track * .94 + np.roll(track, round(.018 * rate)) * .06) * 1.3)
stereo = np.stack((left, right), axis=1)
stereo *= .66 / max(.66, np.max(np.abs(stereo)))
wav = root / 'build' / 'original-audio' / 'endless_dusk.wav'
wav.parent.mkdir(parents=True, exist_ok=True)
with wave.open(str(wav), 'wb') as output:
    output.setnchannels(2)
    output.setsampwidth(2)
    output.setframerate(rate)
    output.writeframes((stereo * 32767).astype('<i2').tobytes())
ogg = root / 'src/main/resources/assets/tp_client/sounds/menu/endless_dusk.ogg'
ogg.parent.mkdir(parents=True, exist_ok=True)
subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-y', '-i', str(wav), '-c:a', 'libvorbis', '-q:a', '5', '-metadata', 'title=Endless Dusk', '-metadata', 'artist=TP LABS', '-metadata', 'comment=Original procedural composition; source: tp-client/tools/compose_menu.py; GPL-3.0-only', str(ogg)], check=True)
print(f'Original TP Client ambience created: {ogg}')
