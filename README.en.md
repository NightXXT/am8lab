# AM8 Lab · FIFINE AM8 software for Windows

[Português](README.md)

![AM8 Lab — Native DSP controls for the FIFINE AM8](docs/assets/banner.svg)

![Windows 10/11 x64](https://img.shields.io/badge/Windows-10%2F11%20x64-0078D4?style=flat-square)
![Experimental 0.7.0](https://img.shields.io/badge/version-0.7.0%20experimental-8B5CF6?style=flat-square)
[![MIT license](https://img.shields.io/badge/license-MIT-22C55E?style=flat-square)](LICENSE)

**[Download Windows installer](https://github.com/NightXXT/am8lab/releases/download/v0.7.0/AM8-Lab-Setup-v0.7.0.exe)** · [Portable ZIP](https://github.com/NightXXT/am8lab/releases/download/v0.7.0/AM8-Lab-Portable-v0.7.0.zip) · [SHA-256 hashes](https://github.com/NightXXT/am8lab/releases/download/v0.7.0/SHA256SUMS.txt) · [Release 0.7.0](https://github.com/NightXXT/am8lab/releases/tag/v0.7.0)

Experimental, unofficial Windows USB controller for the FIFINE AM8's native DSP. Built with Rust + Tauri; the UI is in Portuguese. Voice effects and headphone playback EQ run on the AM8, without PC audio processing, a virtual microphone or firmware flashing.

> [!IMPORTANT]
> Initially validated on **one AM8 USB unit with B5 0.7.1 firmware**. Other revisions are refused. This project is not affiliated with or endorsed by FIFINE.

![AM8 Lab studio interface: voice and headphone controls](docs/assets/studio-preview.png)

*Interface preview with simulated data and no USB communication. This is not a new hardware measurement or test.*

## Quick start

1. Download the installer and verify the published SHA-256 hash. Binaries are not Authenticode-signed.
2. Connect the AM8 by **USB**, open the app and wait for compatibility verification.
3. Prepare an effect, click **Aplicar** (Apply) and use **Comparar com original** (Compare with original). Exit normally to restore the session.

For headphone EQ, select the AM8 playback device in Windows and connect headphones to its P2 output. Other Windows playback devices are not affected.

### New in 0.7.0 · Headphone modes and AM8 information

The headphone **Mono/Stereo** selector has its own Apply button, original comparison and restoration. Mono summing happens in the AM8 DAC. A 60-second test confirmed both tones in both ears and the return to stereo.

The **Capacidades** page shows USB channels/rates, the Windows mix format, calculated PCM data rate and reported core clock. A voice measurement found a mono signal duplicated in both USB channels in the current path. Voice stereo remains under investigation. See [test methods and limitations](docs/NATIVE-MODES.md).

### Faster Apply in 0.6.9

The **Aplicar** (Apply) button remains manual. USB replies are polled sooner, and preparation reads are reused within one command. On one B5 0.7.1 AM8, the noise command fell from **5.22 s to 1.32 s**, and standard pitch from **5.82 s to 1.33 s**. Each measurement covers one application, including compatibility verification, readback and the complete post-write snapshot. Journal persistence and restoration remain in place. More complex effects still take longer. See [methodology and limits](docs/PERFORMANCE.md).

### Visual update in 0.6.8

The microphone illustration has been removed from the Studio panel. USB, filter and firmware details appear on the right in wide windows and below the text in smaller windows.

## Features and compatibility

Download `AM8-Lab-Setup-v0.7.0.exe` and `SHA256SUMS.txt` from this repository's Releases. Check the hash, install, connect the microphone and apply controls. Setup installs for the current user and checks WebView2; if missing, Microsoft's embedded bootstrapper needs internet to install the runtime. Binaries are not Authenticode-signed; a hash is not a trust guarantee.

Validated on Windows x64, USB 3142:A010 / MI_04, B5 0.7.1 / library2.43.2 / engine2.23.2 / HunXiang and one known graph fingerprint. Other revisions are refused. Hardware testing covered one physical unit; broad compatibility is not established.

Noise, ten-filter microphone EQ, independent ten-filter headphone EQ, single-band compressor, standard/Pro pitch and Pro voice transformation are available. Pitch, transformation and reverberation are mutually exclusive. **Room/plate reverb is experimental: playback crackles were reported and stopped when disabled.** Feedback suppression is also experimental. RGB, autotune, echo and multiband compressor are excluded.

Choose **Voz** or **Fones** in the equalizer. Headphone EQ applies to Windows audio played through the AM8 and heard from its P2 headphone output; select the AM8 as the playback device. One 60-second native 2,500 Hz low-pass test was audibly confirmed and its original values restored exactly. Other filters and combinations have not all been listening-tested. See [headphone EQ evidence and limits](docs/HEADPHONE-EQ.md).

Meters use real HID values, with a relative visual scale rather than calibrated dB. Playback response has not been fully characterized. Settings are temporary: comparison restores the pre-session reference and normal app exit restores all affected parameters/routes. Reconnect and restore if communication fails. A serial-bound local recovery journal is saved before hardware writes; pending unbound journals are preserved and refused.

Profiles are local and load controls without automatically applying them. New profiles may include both EQs; older profiles without headphone EQ leave its current controls unchanged. Version 0.6.7 passed 42 Rust tests and 17 update UI checks with a mock transport, alongside the redesign checks documented in [VERIFICACAO.md](VERIFICACAO.md). This change does not add new hardware listening validation. The dependency audit from 0.6.3 is carried forward with unchanged dependency versions; it was not rerun for 0.6.7. See [build instructions](docs/BUILD.md), [security policy](SECURITY.md), [review](docs/SECURITY-REVIEW.md) and [data provenance](docs/PROVENANCE.md). Do not share personal serials or recovery files in Issues.

MIT for original code; third-party notices apply. Not affiliated with FIFINE.

Headphone **general gain** is adjustable from 0 to +18 dB, the verified native descriptor limit. +20 dB exceeds that limit and is rejected. Positive gain can clip; the app has no automatic limiter. EQ boost compensation remains active. Older headphone profiles without the gain field load with 0 dB. See [gain evidence and limits](docs/HEADPHONE-GAIN.md).

## Manual updates in 0.6.6

The **Atualizações** footer button opens a panel with the current version and **Verificar atualizações**. Checks run only on request, work without a connected AM8, and read public Releases from [NightXXT/am8lab](https://github.com/NightXXT/am8lab/releases) over HTTPS. When a newer installer is available, **Baixar instalador** opens it in the default browser. Close the app normally to restore the session before manually running the downloaded installer. There is no automatic installation or firmware update.

The app examines up to 30 published Releases and selects the highest canonical `X.Y.Z` version from an exact `AM8-Lab-Setup-vX.Y.Z.exe` asset. Published experimental prereleases can be offered and are identified. Code commits alone are not downloadable app updates. An older published installer is not offered as a downgrade. No voice, device serial or recovery data is sent; GitHub receives an ordinary network request and the app version in its User-Agent. See [update behavior and verification](docs/UPDATES.md).

## Studio redesign in 0.6.7

The interface now follows the supplied Google Stitch voice-screen export and design guide: graphite surfaces, restrained violet accents, thin meters and navigation for **Estúdio**, **Voz**, **Fones**, **Perfis** and **Capacidades**. Each voice module retains its own Apply action. Headphone and microphone EQ remain independent. Capabilities are grouped by listening evidence, communication/recovery checks and experimental status.

Interface fonts, icons and styles are local or provided by the system, without font/CDN requests at startup. Native DSP commands, recovery behavior and manual GitHub updates are preserved. Mock audio numbers from the design are not presented as real measurements. See [design mapping and limits](docs/REDESIGN.md).

## FAQ

**Is this official FIFINE software?** No. AM8 Lab is an independent, experimental application for one identified AM8 USB revision.

**Does every AM8 work?** Compatibility checks require B5 0.7.1 and the known internal layout. Testing has covered one unit. Other revisions are refused; there is no firmware flashing.

**Can I use XLR alone?** USB is required for the control commands. Documented listening checks concern USB audio and the AM8 headphone path, not XLR output.

**Does headphone EQ affect games and Discord?** It affects Windows audio played through the AM8 and heard from its P2 output. It does not affect another sound card or USB headset.

**Does it control RGB or autotune?** No. These features are excluded. Reverb and feedback suppression remain experimental.

## Help improve the project

Have a compatible AM8? [Report your results](https://github.com/NightXXT/am8lab/issues/new/choose) with the app version, displayed firmware, effects used and expected/observed behavior. Do not include serial numbers, recovery journals or private recordings. If this project is useful, consider starring it or sharing [the repository](https://github.com/NightXXT/am8lab).

The Windows workflow is published, but its [first GitHub run](https://github.com/NightXXT/am8lab/actions/runs/37579658381) was blocked by an account billing issue before a runner started. The reported checks were performed locally.
