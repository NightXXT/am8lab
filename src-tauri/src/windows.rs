use crate::protocol::{self, Block, Result, Transport};
use std::{ffi::c_void, path::Path, ptr, thread, time::Duration};
use sha2::{Digest, Sha256};

type Handle = *mut c_void;
#[repr(C)]
struct Guid { a: u32, b: u16, c: u16, d: [u8; 8] }
const HID_GUID: Guid = Guid { a: 0x4d1e55b2, b: 0xf16f, c: 0x11cf, d: [0x88, 0xcb, 0, 0x11, 0x11, 0, 0, 0x30] };

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateFileW(name: *const u16, access: u32, share: u32, security: *const c_void, creation: u32, flags: u32, template: Handle) -> Handle;
    fn CloseHandle(handle: Handle) -> i32;
    fn GetLastError() -> u32;
    fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    fn CreateMutexW(security: *const c_void, owner: i32, name: *const u16) -> Handle;
}
#[link(name = "cfgmgr32")]
unsafe extern "system" {
    fn CM_Get_Device_Interface_List_SizeW(size: *mut u32, guid: *const Guid, device: *const u16, flags: u32) -> u32;
    fn CM_Get_Device_Interface_ListW(guid: *const Guid, device: *const u16, buffer: *mut u16, length: u32, flags: u32) -> u32;
}
#[link(name = "hid")]
unsafe extern "system" {
    fn HidD_GetPreparsedData(handle: Handle, data: *mut *mut c_void) -> u8;
    fn HidD_FreePreparsedData(data: *mut c_void) -> u8;
    fn HidP_GetCaps(data: *mut c_void, caps: *mut u16) -> i32;
    fn HidD_SetOutputReport(handle: Handle, buffer: *const c_void, length: u32) -> u8;
    fn HidD_GetInputReport(handle: Handle, buffer: *mut c_void, length: u32) -> u8;
    fn HidD_GetSerialNumberString(handle: Handle, buffer: *mut c_void, length: u32) -> u8;
}

fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(Some(0)).collect() }
fn win_error(context: &str) -> String { format!("{context}: Windows {}", unsafe { GetLastError() }) }
#[link(name = "user32")]
unsafe extern "system" { fn MessageBoxW(window: Handle, text: *const u16, caption: *const u16, kind: u32) -> i32; }
pub fn show_error(error: &str) {
    let message = wide(error); let caption = wide("AM8 Lab");
    unsafe { MessageBoxW(ptr::null_mut(), message.as_ptr(), caption.as_ptr(), 0x10); }
}

pub fn atomic_replace(from: &Path, to: &Path) -> Result<()> {
    let f = wide(&from.to_string_lossy()); let t = wide(&to.to_string_lossy());
    // MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH. Same-volume temporary file.
    if unsafe { MoveFileExW(f.as_ptr(), t.as_ptr(), 1 | 8) } == 0 { return Err(win_error("Não foi possível salvar a recuperação")); }
    Ok(())
}

pub struct InstanceMutex(Vec<Handle>);
impl InstanceMutex {
    pub fn acquire() -> Result<Self> {
        let mut lock=Self(Vec::new());
        // The local lock also coordinates with older versions in this session.
        for id in ["Global\\AM8LabEffectsSession", "Local\\AM8LabEffectsSession"] {
            let name = wide(id);
            let h = unsafe { CreateMutexW(ptr::null(), 1, name.as_ptr()) };
            let error = unsafe { GetLastError() };
            if h.is_null() { return Err(format!("Falha no bloqueio de instância: {error}")); }
            lock.0.push(h);
            if error == 183 {
                return Err("Outra versão do AM8 Lab está aberta. Feche-a antes de continuar.".into());
            }
        }
        Ok(lock)
    }
}
impl Drop for InstanceMutex { fn drop(&mut self) { for h in &self.0 { unsafe { CloseHandle(*h); } } } }

pub struct HidDevice { handle: Handle, serial: String }
impl HidDevice {
    pub fn open() -> Result<Self> {
        let mut count = 0;
        if unsafe { CM_Get_Device_Interface_List_SizeW(&mut count, &HID_GUID, ptr::null(), 0) } != 0 || count == 0 || count > 1_000_000 {
            return Err("Não foi possível listar as interfaces HID".into());
        }
        let mut paths = vec![0u16; count as usize];
        if unsafe { CM_Get_Device_Interface_ListW(&HID_GUID, ptr::null(), paths.as_mut_ptr(), count, 0) } != 0 {
            return Err("Não foi possível ler as interfaces HID".into());
        }
        let candidates: Vec<String> = paths.split(|v| *v == 0).filter(|p| !p.is_empty()).map(String::from_utf16_lossy)
            .filter(|p| p.to_lowercase().contains("vid_3142&pid_a010&mi_04")).collect();
        if candidates.len() != 1 { return Err(format!("Conecte um único AM8 USB compatível. Interfaces encontradas: {}", candidates.len())); }
        let path = wide(&candidates[0]);
        let handle = unsafe { CreateFileW(path.as_ptr(), 0xC0000000, 3, ptr::null(), 3, 0, ptr::null_mut()) };
        if handle as isize == -1 { return Err(win_error("Não foi possível abrir o AM8")); }
        let mut device = Self { handle, serial: String::new() };
        let mut data = ptr::null_mut();
        if unsafe { HidD_GetPreparsedData(handle, &mut data) } == 0 { return Err(win_error("Não foi possível verificar o HID")); }
        let mut caps = [0u16; 32];
        let status = unsafe { HidP_GetCaps(data, caps.as_mut_ptr()) };
        unsafe { HidD_FreePreparsedData(data); }
        if status != 0x110000 || caps[..5] != [0x55AA, 0xFF00, 257, 257, 9] {
            return Err("Layout HID não validado; operações recusadas".into());
        }
        let mut serial = [0u16; 126];
        if unsafe { HidD_GetSerialNumberString(handle, serial.as_mut_ptr().cast(), (serial.len() * 2) as u32) } == 0 {
            return Err(win_error("Não foi possível identificar o número de série"));
        }
        let n = serial.iter().position(|v| *v == 0).unwrap_or(serial.len());
        device.serial = String::from_utf16_lossy(&serial[..n]);
        if device.serial.is_empty() { return Err("Número de série vazio".into()); }
        Ok(device)
    }

    fn send(&mut self, op: u8, payload: &[u8]) -> Result<()> {
        let out = protocol::frame(op, payload)?;
        if unsafe { HidD_SetOutputReport(self.handle, out.as_ptr().cast(), 257) } == 0 { return Err(win_error("Envio HID falhou")); }
        Ok(())
    }

    fn exchange(&mut self, op: u8, request: &[u8], sequence: Option<u8>) -> Result<Vec<u8>> {
        self.exchange_with_wait(op, request, sequence, 80)
    }

    pub fn meter_with_wait(&mut self, op: u8, wait_ms: u64) -> Result<u16> {
        if !matches!(op, 0x91 | 0x82) || !(4..=80).contains(&wait_ms) { return Err("Consulta de medidor inválida".into()); }
        let p = self.exchange_with_wait(op, &[], None, wait_ms)?;
        if p.len() != 5 || p[..3] != [255, 1, 0] { return Err("Resposta de medidor inválida".into()); }
        let value = u16::from_le_bytes([p[3], p[4]]);
        if value > 32767 { return Err("Amplitude de medidor inválida".into()); }
        Ok(value)
    }

    fn exchange_with_wait(&mut self, op: u8, request: &[u8], sequence: Option<u8>, wait_ms: u64) -> Result<Vec<u8>> {
        if op == 0x80 && sequence.is_none() && request.is_empty() { return Err("Consulta de modo sem seletor".into()); }
        self.send(op, request)?;
        for _ in 0..(800 / wait_ms) {
            thread::sleep(Duration::from_millis(wait_ms));
            let mut raw = [0u8; 257];
            if unsafe { HidD_GetInputReport(self.handle, raw.as_mut_ptr().cast(), 257) } == 0 { return Err(win_error("Leitura HID falhou")); }
            if let Ok(p) = protocol::decode(&raw, op) {
                if let Some(seq) = sequence {
                    if p.first() != Some(&seq) { continue; }
                } else if op == 0x80 {
                    if p.len() < 5 || p[..3] != [0, 255, 0] || p[3] != request[0] || p.last() != Some(&1) { continue; }
                }
                return Ok(p.to_vec());
            }
        }
        Err("O AM8 não confirmou a resposta esperada".into())
    }
}

impl Transport for HidDevice {
    fn serial(&self) -> &str { &self.serial }
    fn query(&mut self, op: u8, payload: &[u8]) -> Result<Vec<u8>> { self.exchange(op, payload, None) }
    fn write_word(&mut self, block: Block, index: usize, value: i16) -> Result<()> {
        protocol::validate_word(block, index, value)?;
        let bytes = value.to_le_bytes();
        self.send(block.address(), &[index as u8, bytes[0], bytes[1]])?;
        thread::sleep(Duration::from_millis(120));
        Ok(())
    }
    fn guard(&mut self) -> Result<()> {
        let id = self.query(0, &[])?;
        if id != [0x42, 0, 7, 1, 2, 43, 2, 2, 23, 2, 2, b'B', b'5', 1] {
            return Err("Firmware não validada. Esperado B5 0.7.1 / biblioteca 2.43.2 / engine 2.23.2.".into());
        }
        let mode = self.query(0x80, &[6])?;
        if mode.len() != 7 || mode[5] != 0 || self.query(0xFC, &[])? != b"HunXiang" {
            return Err("Modo ou fluxo interno não validado".into());
        }
        let mut graph = Vec::new();
        let mut total = 0usize;
        let mut complete = false;
        for seq in 0..=255u8 {
            let p = self.exchange(0x80, &[2], Some(seq))?;
            if seq == 0 {
                if p.len() < 7 || p[1..4] != [255, 0, 2] { return Err("Cabeçalho do fluxo inválido".into()); }
                total = u16::from_le_bytes([p[4], p[5]]) as usize;
                graph.extend_from_slice(&p[6..p.len() - 1]);
            } else {
                if p.len() < 2 { return Err("Pacote do fluxo incompleto".into()); }
                graph.extend_from_slice(&p[1..p.len() - 1]);
            }
            if p.last() == Some(&1) { complete = true; break; }
            if p.last() != Some(&0) { return Err("Marcador do fluxo inválido".into()); }
        }
        const GRAPH_LENGTH: usize = 2931;
        const GRAPH_SHA256: [u8; 32] = [0xbd,0x9d,0xbe,0x74,0x99,0x4c,0xbb,0x96,0xcb,0x29,0x81,0xc6,0xbb,0x34,0x0a,0xae,0x5b,0x89,0x44,0x09,0xcf,0x29,0x03,0x78,0xc6,0xe0,0x20,0x1f,0xfc,0x2f,0x13,0x1d];
        if !complete || graph.len() != total || total != GRAPH_LENGTH || Sha256::digest(&graph)[..] != GRAPH_SHA256 {
            return Err("O fluxo interno difere do aparelho investigado".into());
        }
        Ok(())
    }
}

impl Drop for HidDevice { fn drop(&mut self) { unsafe { CloseHandle(self.handle); } } }
