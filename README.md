# Dictator

General-purpose local voice mode for the focused app.

Hold a hotkey, talk, and the transcribed text streams into whatever window currently holds keyboard focus. Fully local (whisper.cpp via Metal on Apple Silicon), no cloud and no API keys. App-agnostic: it works with any program that accepts typed input.

## Status

Early skeleton. Being built live as the companion project to a blog post on The Gigi Zone.

## Planned pipeline

- Hold-to-talk on **right Option** (`rdev`)
- Mic capture ([`cpal`](https://docs.rs/cpal/) + `hound`)
- Local speech-to-text (`whisper-rs`, `metal` feature, `ggml-large-v3-turbo`)
- Inject into the focused window (`enigo` typing by default, `arboard` + synthetic Cmd+V as fallback)

Design rule: inject the transcribed text but never auto-press Enter. Voice transcription makes
mistakes, so the final keystroke stays with the human.

## Decisions

**Backend: `whisper-rs` + Metal, not MLX.** `mlx-whisper` is Python-only, so using it means shipping a
Python sidecar and giving up the single-binary story. The one Rust port (`mlx-whisper-rs`) is a
5-commit repo that isn't on crates.io. MLX's advantage is throughput on long files; hold-to-talk
clips are seconds long and Whisper pads to a 30s window regardless, so the per-press cost is one
encoder pass either way. The latency that actually matters is model loading — so the
`WhisperContext` is built once at startup and kept resident for the process lifetime, never
reloaded per keypress.

**Model: `ggml-large-v3-turbo` (1.62 GB).** Turbo trades a little accuracy on hard audio for a much
faster decode, which is the right side of that trade for short, clean, close-mic dictation.

**Streaming: segmented by voice activity detection (VAD), not sliding-window.** VAD is just
speech-vs-silence classification over short frames of mic input; here it supplies the phrase
boundaries. Whisper has no incremental decoder, so any "streaming" is re-transcription — and
consecutive passes over a growing window *revise* earlier
words. Since typed keystrokes can't be retracted from another app's text field, we only ever emit
text that is already final: wait for a speech pause, transcribe the completed phrase, type it,
clear the buffer. Text arrives phrase-by-phrase as you pause naturally. The upgrade path, if
mid-phrase output is wanted later, is the LocalAgreement-2 policy from
[Whisper-Streaming](https://arxiv.org/abs/2307.14743) — re-transcribe every ~750ms and commit only
the longest common prefix that two consecutive passes agree on. That swaps the segmentation
strategy without touching the injection layer.

Note that `whisper-rs`'s own VAD support (`FullParams::enable_vad`) is *not* a substitute: it trims
silence inside a `full()` call, so it can't tell you when to make the call in the first place.
Segmentation VAD has to sit upstream, on the capture side. Starting with a plain RMS energy
threshold plus a hangover window — the same approach `whisper.cpp`'s `stream` example takes — and
reaching for [`earshot`](https://lib.rs/crates/earshot) only if that proves flaky.

**Audio transport: lock-free ring buffer, not a channel.** Real-time audio callbacks must not
allocate, lock, or block — the driver gives them a fixed deadline, and missing it produces dropouts.
`std::sync::mpsc` allocates internally and isn't documented as real-time-safe, so captured frames go
through a lock-free single-producer/single-consumer ring buffer instead
([`rtrb`](https://docs.rs/rtrb/)): the cpal callback writes into preallocated memory and never
touches the allocator.

Two consequences worth stating up front. First, audio bypasses the `DictatorEvents` callback trait
entirely. `dictator` creates one ring at startup, sized for the longest stretch the worker can go
without draining (a transcription), and hands the `Producer` to `platform`'s capture adapter and the
`Consumer` to `engine`. The producer needs `&mut self` and isn't `Sync`, so the adapter keeps it in
an `Arc<Mutex<Producer<f32>>>`, and each take's data callback holds an `Arc` clone and pushes under
`try_lock`. `try_lock` never blocks, and nothing else ever locks that mutex, so it always succeeds.
It costs one atomic per callback, which is real-time-safe in a way a blocking `lock` isn't. Dropping
the stream on `stop` drops only the clone, so the same producer serves every take. The trait carries
only control events (start, stop, capture error), which happen a few times a minute and can allocate
freely.

Second, the engine has two inputs and can't block on both, since std has no stable `select`. So it
blocks on the control channel while idle — zero CPU when nobody is dictating — and switches to
draining the ring buffer on a short cadence while recording. Polling only happens during a take.

**Injection: `enigo` default, clipboard fallback.** Per-character typing keeps the user's clipboard
untouched and suits phrase-at-a-time output. The risk is that terminal apps drop characters under a
fast synthetic-event burst — unmeasured so far. `arboard` + Cmd+V (with prior clipboard contents
saved and restored) is the fallback for when typing proves unreliable or too slow.

**Hotkey: hold right Option.** Nothing on macOS binds a lone right-Option, and holding it produces
no character, so it's free real estate. This rules out the `global-hotkey` crate:
`HotKey::new(mods: Option<Modifiers>, key: Code)` requires a non-optional `Code`, so
modifier-only bindings are unrepresentable. `rdev`'s `listen` handles it, and needs no fallback:
under the hood it already *is* a CGEventTap on `kCGEventFlagsChanged`, and its keycode table
distinguishes the two Option keys — 58 is `Key::Alt`, 61 is `Key::AltGr` — so a right-Option hold
surfaces as an ordinary `KeyPress`/`KeyRelease` pair.

The seam to watch is how those edges get synthesized. A flags-changed event carries no up/down bit,
so `rdev` infers direction by comparing the whole modifier bitmask against a process-global previous
value: numerically larger means press, smaller means release. That assumes one modifier changes at a
time, so chording right-Option with Cmd or Shift is where it could misreport.

**Focus: resolved at inject time.** No snapshot of the frontmost app at key-down, no mismatch
check. Each phrase lands wherever keyboard focus is when it's typed. The natural usage is to focus
the target window first and then talk.

**Packaging: bare binary for now.** A `cargo run` binary has no bundle identity, so macOS attaches
Accessibility and Input Monitoring grants to the parent terminal rather than to dictator itself.
Fine during development; a real `.app` bundle (`cargo-bundle` + `Info.plist` usage descriptions)
is deferred until this stops being a demo.

## Architecture

`rdev`'s listener and macOS global input taps both require a CFRunLoop on the main thread, which
fixes the threading shape:

- **Main thread** — owns the input tap. Right-Option down/up drives `on_start` / `on_stop`, which
  start and stop capture *inline*: `build_input_stream` plus `play()` return in milliseconds and
  cpal spawns its own threads, so there's nothing worth handing off. `on_stop` also ships the
  completed take to the worker. Does no work that can block.
- **CoreAudio thread** — cpal's own. Its data callback writes samples into the ring buffer.
- **Worker thread** — owns the resident `WhisperContext` and the injector. Detects pause
  boundaries, transcribes each completed phrase, types it.

Only transcription is worth a thread of its own; capture control isn't. The `cpal::Stream` handle
therefore lives in `platform`'s capture adapter behind a `Mutex<Option<Stream>>`, since the trait
methods take `&self`. That `Option` doubles as the already-capturing guard, which matters because
`rdev` can emit a second `KeyPress` with no release between (see the flags-bitmask seam above) —
`start` and `stop` both have to be idempotent.

Keeping STT off the main thread isn't only about latency. macOS disables an event tap whose callback
overruns, notifying via `kCGEventTapDisabledByTimeout`, and `rdev` 0.5.3 handles neither half of
that: `convert()` has no arm for the notification, and `CGEventTapEnable` is called exactly once at
startup. An overrun doesn't swallow one keypress — it kills the hotkey for the rest of the process,
silently, while `listen()` keeps spinning and never returns an error.

## Testing grounds

pi.dev, Claude Code, Codex, and OpenCode.

## Build

Fetch the model first (gitignored, not committed):

```
mkdir -p models
curl -L -o models/ggml-large-v3-turbo.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin
cargo run
```

## macOS permissions

Three permissions, granted in System Settings → Privacy & Security:

| Permission | Needed for | Used by |
|---|---|---|
| Input Monitoring | reading the global hotkey | `rdev` |
| Accessibility | posting synthetic keystrokes | `enigo` |
| Microphone | capture | `cpal` |

The non-obvious part: **TCC attributes these to the responsible process, not to the binary.** A bare
`cargo run` binary has no bundle identity, so the grant lands on whatever launched it — iTerm for a
terminal run, RustRover for an IDE run. Those are separate grants; granting one does nothing for the
other, and each prompts on its own first run. After granting, quit and reopen the app or the grant
won't take effect.

Two consequences. Denying the prompt once flips TCC from "unknown" to "denied", after which
`rdev::listen` returns `Err(ListenError::EventTapError)` with no further prompting — recovery is
through System Settings, not another dialog. And granting Input Monitoring to a terminal lets *every*
binary run from that terminal read your keystrokes, which is the real argument for shipping a proper
`.app` bundle: the grant would then apply to dictator alone.

To check the current state without triggering a prompt:

```
python3 -c "
import ctypes, ctypes.util
a = ctypes.CDLL(ctypes.util.find_library('ApplicationServices'))
a.AXIsProcessTrusted.restype = ctypes.c_bool
print('Accessibility:', a.AXIsProcessTrusted())
io = ctypes.CDLL(ctypes.util.find_library('IOKit'))
io.IOHIDCheckAccess.restype = ctypes.c_int
io.IOHIDCheckAccess.argtypes = [ctypes.c_int]
print('Input Monitoring:', {0: 'granted', 1: 'denied', 2: 'not yet asked'}[io.IOHIDCheckAccess(1)])
"
```
