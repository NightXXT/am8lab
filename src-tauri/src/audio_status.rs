//! Read-only Windows default-output information. Never changes routing or volume.
use ::windows::Win32::{Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
    Media::Audio::{IMMDeviceEnumerator, MMDeviceEnumerator, eRender, eConsole},
    System::Com::{CoInitializeEx, CoUninitialize, CoCreateInstance, CoTaskMemFree,
        CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ},
    System::Com::StructuredStorage::{PropVariantClear, PropVariantToStringAlloc}};

pub fn default_output() -> Option<String> {
    unsafe {
        if CoInitializeEx(None, COINIT_MULTITHREADED).is_err() { return None; }
        let result = (|| -> ::windows::core::Result<String> {
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
            let store = device.OpenPropertyStore(STGM_READ)?;
            let mut value = store.GetValue(&PKEY_Device_FriendlyName)?;
            let allocated = PropVariantToStringAlloc(&value);
            let _ = PropVariantClear(&mut value);
            let text = allocated?;
            let result = text.to_string();
            CoTaskMemFree(Some(text.0.cast()));
            Ok(result?)
        })().ok();
        CoUninitialize();
        result
    }
}
