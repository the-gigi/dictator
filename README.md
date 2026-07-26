# dictator

General-purpose local voice mode for the focused app.

Hold a hotkey, talk, and the transcribed text is injected into whatever window currently holds keyboard focus. Fully local (whisper.cpp via Metal on Apple Silicon), no cloud and no API keys. App-agnostic: it works with any program that accepts typed input.

## Status

Early skeleton. Being built live as the companion project to a blog post on The Gigi Zone.

## Planned pipeline

- Global push-to-talk hotkey (`global-hotkey`)
- Mic capture (`cpal` + `hound`)
- Local speech-to-text (`whisper-rs`, Metal feature)
- Inject into the focused window (`enigo`, or `arboard` + synthetic Cmd+V)

Design rule: inject the transcribed text but never auto-press Enter. Voice transcription makes mistakes, so the final keystroke stays with the human.

## Testing grounds

pi.dev, Claude Code, Codex, and OpenCode.

## Build

```
cargo run
```

## macOS permissions

First run needs three permissions granted in System Settings: Microphone, Accessibility (synthetic input events), and Input Monitoring (the global hotkey).
