//! Guarded device diagnostics and read-only Windows mix-format information.
//! USB capabilities are those observed in the validated B5 0.7.1 revision;
//! core clock and internal rate are decoded from the reference protocol.
use crate::protocol::{Result, Transport};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub firmware: &'static str,
    pub usb_capabilities_verified: bool,
    pub usb_capture_channels: Option<u16>,
    pub usb_playback_channels: Option<u16>,
    pub usb_sample_rates_hz: Option<[u32; 2]>,
    pub usb_transport_bits: Option<[u16; 2]>,
    pub usb_speed: Option<&'static str>,
    pub usb_protocol: Option<&'static str>,
    pub usb_capabilities_source: &'static str,
    pub reported_core_clock_mhz: Option<u16>,
    pub internal_sample_rate_hz_inferred: Option<u32>,
    pub internal_frame_samples_inferred: Option<u16>,
    pub windows_capture_sample_rate_hz: Option<u32>,
    pub windows_capture_channels: Option<u16>,
    pub windows_capture_mix_bits: Option<u16>,
    pub windows_playback_sample_rate_hz: Option<u32>,
    pub windows_playback_channels: Option<u16>,
    pub windows_playback_mix_bits: Option<u16>,
    pub headset_mode: Option<String>,
    pub voice_mode_status: &'static str,
}

const RATES: [u32; 13] = [8000,11025,12000,16000,22050,24000,32000,44100,48000,88200,96000,176400,192000];

fn decode_diagnostics(system: &[u8], status: &[u8]) -> Result<(Option<u16>, Option<u32>, Option<u16>)> {
    if system.len() != 17 || system[0] != 255 || status.len() != 12 || status[0] != 255 {
        return Err("Layout das informações internas não validado".into());
    }
    let clock = u16::from_le_bytes([status[6], status[7]]);
    let frame = u16::from_le_bytes([system[15], system[16]]);
    Ok((
        if (1..=1000).contains(&clock) { Some(clock) } else { None },
        RATES.get(usize::from(system[7])).copied(),
        if (1..=4096).contains(&frame) { Some(frame) } else { None },
    ))
}

pub fn read(t: &mut impl Transport) -> Result<DeviceInfo> {
    t.guard()?;
    let (clock, rate, frame) = decode_diagnostics(&t.query(0x01, &[])?, &t.query(0x02, &[])?)?;
    let dac = t.query(0x09, &[])?;
    if dac.len() != 29 || dac[0] != 255 { return Err("Layout do DAC não validado".into()); }
    let mode = u16::from_le_bytes([dac[15], dac[16]]);
    let headset_mode = match mode { 0 => Some("stereo".into()), 2 => Some("mono".into()), _ => Some("other".into()) };
    let (capture, playback) = windows_mix_formats();
    // A matching DSP graph is not proof of identical USB audio descriptors.
    let firmware = t.firmware();
    let usb_capabilities_verified = firmware == "B5 0.7.1";
    Ok(DeviceInfo {
        firmware, usb_capabilities_verified,
        usb_capture_channels: usb_capabilities_verified.then_some(2),
        usb_playback_channels: usb_capabilities_verified.then_some(2),
        usb_sample_rates_hz: usb_capabilities_verified.then_some([44100,48000]),
        usb_transport_bits: usb_capabilities_verified.then_some([16,24]),
        usb_speed: usb_capabilities_verified.then_some("Full Speed"),
        usb_protocol: usb_capabilities_verified.then_some("UAC 1.0"),
        usb_capabilities_source: "Descriptors observed in the validated B5 0.7.1 unit; not a new USB descriptor fetch",
        reported_core_clock_mhz: clock, internal_sample_rate_hz_inferred: rate,
        internal_frame_samples_inferred: frame,
        windows_capture_sample_rate_hz: capture.map(|format| format.0),
        windows_capture_channels: capture.map(|format| format.1),
        windows_capture_mix_bits: capture.map(|format| format.2),
        windows_playback_sample_rate_hz: playback.map(|format| format.0),
        windows_playback_channels: playback.map(|format| format.1),
        windows_playback_mix_bits: playback.map(|format| format.2),
        headset_mode, voice_mode_status: "investigating",
    })
}

type Mix = Option<(u32, u16, u16)>;

#[cfg(not(windows))]
fn windows_mix_formats() -> (Mix, Mix) { (None, None) }

#[cfg(windows)]
fn windows_mix_formats() -> (Mix, Mix) {
    use ::windows::Win32::{
        Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
        Media::Audio::{DEVICE_STATE_ACTIVE, EDataFlow, IAudioClient, IMMDeviceEnumerator, MMDeviceEnumerator, eCapture, eRender},
        System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ, CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
            StructuredStorage::{PropVariantClear, PropVariantToStringAlloc}},
    };
    struct Apartment;
    impl Drop for Apartment { fn drop(&mut self) { unsafe { CoUninitialize(); } } }
    unsafe fn endpoint_format(enumerator: &IMMDeviceEnumerator, flow: EDataFlow) -> Mix {
        unsafe {
            let endpoints = enumerator.EnumAudioEndpoints(flow, DEVICE_STATE_ACTIVE).ok()?;
            let mut selected = None;
            for index in 0..endpoints.GetCount().ok()? {
                let device = endpoints.Item(index).ok()?;
                let store = device.OpenPropertyStore(STGM_READ).ok()?;
                let mut value = store.GetValue(&PKEY_Device_FriendlyName).ok()?;
                let allocated = PropVariantToStringAlloc(&value);
                let _ = PropVariantClear(&mut value);
                let name = allocated.ok()?;
                let text = name.to_string();
                CoTaskMemFree(Some(name.0.cast()));
                if text.ok()?.to_lowercase().contains("fifine") {
                    if selected.is_some() { return None; }
                    selected = Some(device);
                }
            }
            let client: IAudioClient = selected?.Activate(CLSCTX_ALL, None).ok()?;
            let format = client.GetMixFormat().ok()?;
            if format.is_null() { return None; }
            let wave = std::ptr::read_unaligned(format);
            CoTaskMemFree(Some(format.cast()));
            let rate = wave.nSamplesPerSec;
            let channels = wave.nChannels;
            let bits = wave.wBitsPerSample;
            if !(8000..=384000).contains(&rate) || !(1..=32).contains(&channels)
                || !(1..=64).contains(&bits) { return None; }
            Some((rate, channels, bits))
        }
    }
    unsafe {
        if CoInitializeEx(None, COINIT_MULTITHREADED).is_err() { return (None, None); }
        let _apartment = Apartment;
        let Ok(enumerator) = CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL) else { return (None, None); };
        (endpoint_format(&enumerator, eCapture), endpoint_format(&enumerator, eRender))
    }
}

#[cfg(test)]
mod tests {
    use super::decode_diagnostics;
    use crate::protocol::{Block, Result, Transport};
    struct Fake { firmware: &'static str, rejected: bool, queries: usize }
    impl Transport for Fake {
        fn serial(&self) -> &str { "FAKE_DIAGNOSTIC" }
        fn firmware(&self) -> &'static str { self.firmware }
        fn guard(&mut self) -> Result<()> { if self.rejected { Err("Unverified".into()) } else { Ok(()) } }
        fn query(&mut self, op: u8, _: &[u8]) -> Result<Vec<u8>> {
            self.queries+=1;
            match op {
                0x01 => Ok(vec![255,0,0,0,0,0,1,7,0,1,0,0,0,0,0,0,1]),
                0x02 => Ok(vec![255,120,0,160,0,0,240,0,0,1,0,0]),
                0x09 => { let mut dac=vec![0;29];dac[0]=255;Ok(dac) },
                _ => Err("Unexpected query".into()),
            }
        }
        fn write_word(&mut self, _: Block, _: usize, _: i16) -> Result<()> { panic!("Diagnostics must not write") }
    }
    #[test] fn revision_073_has_actual_label_without_inheriting_usb_descriptor_claims() {
        let mut t=Fake { firmware:"B5 0.7.3",rejected:false,queries:0 };
        let info=super::read(&mut t).unwrap();
        assert_eq!(info.firmware,"B5 0.7.3");assert!(!info.usb_capabilities_verified);
        assert_eq!(info.usb_sample_rates_hz,None);assert_eq!(info.usb_transport_bits,None);
        assert_eq!(info.usb_capture_channels,None);assert_eq!(info.usb_playback_channels,None);
        assert_eq!(info.usb_speed,None);assert_eq!(info.usb_protocol,None);
        assert_eq!(info.reported_core_clock_mhz,Some(240));
        t.firmware="B5 0.7.1";let reference=super::read(&mut t).unwrap();
        assert!(reference.usb_capabilities_verified);assert_eq!(reference.usb_sample_rates_hz,Some([44100,48000]));
    }
    #[test] fn rejected_device_is_not_queried_for_diagnostics() {
        let mut t=Fake { firmware:"",rejected:true,queries:0 };
        assert!(super::read(&mut t).is_err());assert_eq!(t.queries,0);
    }
    #[test]
    fn decodes_observed_unit_without_equating_clock_and_sample_rate() {
        let system = [255,0,0,0,0,0,1,7,0,1,0,0,0,0,0,0,1];
        let status = [255,120,0,160,0,0,240,0,0,1,0,0];
        assert_eq!(decode_diagnostics(&system, &status).unwrap(), (Some(240),Some(44100),Some(256)));
    }
    #[test]
    fn unknown_values_remain_unavailable_and_wrong_layout_is_rejected() {
        let mut system = [255;17];
        let status = [255;12];
        system[7] = 255;
        assert_eq!(decode_diagnostics(&system, &status).unwrap(), (None,None,None));
        assert!(decode_diagnostics(&system[..16], &status).is_err());
        system[0] = 0;
        assert!(decode_diagnostics(&system, &status).is_err());
    }
}
