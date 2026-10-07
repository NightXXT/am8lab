# AM8 Lab

[Português](README.md)

Experimental, unofficial Windows USB controller for the FIFINE AM8's native DSP. Built with Rust + Tauri; the UI is in Portuguese. Voice effects and headphone playback EQ run on the AM8, without PC audio processing, a virtual microphone or firmware flashing.

Download `AM8-Lab-Setup-v0.6.7.exe` and `SHA256SUMS.txt` from this repository's Releases. Check the hash, install, connect the microphone and apply controls. Setup installs for the current user and checks WebView2; if missing, Microsoft's embedded bootstrapper needs internet to install the runtime. Binaries are not Authenticode-signed; a hash is not a trust guarantee.

Validated on Windows x64, USB 3142:A010 / MI_04, B5 0.7.1 / library2.43.2 / engine2.23.2 / HunXiang and one known graph fingerprint. Other revisions are refused. Hardware testing covered one physical unit; broad compatibility is not established.

Noise, ten-filter microphone EQ, independent ten-filter headphone EQ, single-band compressor, standard/Pro pitch and Pro voice transformation are available. Pitch, transformation and reverberation are mutually exclusive. **Room/plate reverb is experimental: playback crackles were reported and stopped when disabled.** Feedback suppression is also experimental. RGB, autotune, echo and multiband compressor are excluded.

Choose **Voz** or **Fones** in the equalizer. Headphone EQ applies to Windows audio played through the AM8 and heard from its P2 headphone output; select the AM8 as the playback device. One 60-second native 2,500 Hz low-pass test was audibly confirmed and its original values restored exactly. Other filters and combinations have not all been listening-tested. See [headphone EQ evidence and limits](docs/HEADPHONE-EQ.md).

Meters use real HID values, with a relative visual scale rather than calibrated dB. Playback response has not been fully characterized. Settings are temporary: comparison restores the pre-session reference and normal app exit restores all affected parameters/routes. Reconnect and restore if communication fails. A serial-bound local recovery journal is saved before hardware writes; pending unbound journals are preserved and refused.

Profiles are local and load controls without automatically applying them. New profiles may include both EQs; older profiles without headphone EQ leave its current controls unchanged. Version 0.6.6 passed 42 Rust tests and 17 update UI checks with a mock transport. This change does not add new hardware listening validation. The dependency audit from 0.6.3 is carried forward with unchanged dependency versions; it was not rerun for 0.6.6. See [build instructions](docs/BUILD.md), [security policy](SECURITY.md), [review](docs/SECURITY-REVIEW.md) and [data provenance](docs/PROVENANCE.md). Do not share personal serials or recovery files in Issues.

MIT for original code; third-party notices apply. Not affiliated with FIFINE.

Headphone **general gain** is adjustable from 0 to +18 dB, the verified native descriptor limit. +20 dB exceeds that limit and is rejected. Positive gain can clip; the app has no automatic limiter. EQ boost compensation remains active. Older headphone profiles without the gain field load with 0 dB. See [gain evidence and limits](docs/HEADPHONE-GAIN.md).

## Manual updates in 0.6.6

The **Atualizações** footer button opens a panel with the current version and **Verificar atualizações**. Checks run only on request, work without a connected AM8, and read public Releases from [NightXXT/am8lab](https://github.com/NightXXT/am8lab/releases) over HTTPS. When a newer installer is available, **Baixar instalador** opens it in the default browser. Close the app normally to restore the session before manually running the downloaded installer. There is no automatic installation or firmware update.

The app examines up to 30 published Releases and selects the highest canonical `X.Y.Z` version from an exact `AM8-Lab-Setup-vX.Y.Z.exe` asset. Published experimental prereleases can be offered and are identified. Code commits alone are not downloadable app updates. An older published installer is not offered as a downgrade. No voice, device serial or recovery data is sent; GitHub receives an ordinary network request and the app version in its User-Agent. See [update behavior and verification](docs/UPDATES.md).

## Studio redesign in 0.6.7

The interface now follows the supplied Google Stitch voice-screen export and design guide: graphite surfaces, restrained violet accents, thin meters and navigation for **Estúdio**, **Voz**, **Fones**, **Perfis** and **Capacidades**. Each voice module retains its own Apply action. Headphone and microphone EQ remain independent. Capabilities are grouped by listening evidence, communication/recovery checks and experimental status.

Interface fonts, icons and styles are local or provided by the system, without font/CDN requests at startup. Native DSP commands, recovery behavior and manual GitHub updates are preserved. Mock audio numbers from the design are not presented as real measurements. See [design mapping and limits](docs/REDESIGN.md).
