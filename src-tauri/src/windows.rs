use crate::protocol::{self, Block, Result, Transport};
use std::{ffi::c_void, path::Path, ptr, thread, time::{Duration, Instant}};
use sha2::{Digest, Sha256};

type Handle = *mut c_void;
#[repr(C)]
struct Guid { a: u32, b: u16, c: u16, d: [u8; 8] }
const HID_GUID: Guid = Guid { a: 0x4d1e55b2, b: 0xf16f, c: 0x11cf, d: [0x88, 0xcb, 0, 0x11, 0x11, 0, 0, 0x30] };

/// Exact identities only; DSP mode, flow name and complete graph remain mandatory.
/// The 0.7.3 identity and matching flow come from a user-supplied read-only report.
pub const ACCEPTED_FIRMWARE: [([u8; 14], &str); 2] = [
    ([0x42,0,7,1,2,43,2,2,23,2,2,b'B',b'5',1], "B5 0.7.1"),
    ([0x42,0,7,3,2,43,2,2,23,2,2,b'B',b'5',1], "B5 0.7.3"),
];
pub const GRAPH_LENGTH: usize = 2931;
pub const GRAPH_SHA256: [u8;32] = [0xbd,0x9d,0xbe,0x74,0x99,0x4c,0xbb,0x96,0xcb,0x29,0x81,0xc6,0xbb,0x34,0x0a,0xae,0x5b,0x89,0x44,0x09,0xcf,0x29,0x03,0x78,0xc6,0xe0,0x20,0x1f,0xfc,0x2f,0x13,0x1d];
pub fn accepted_firmware(id: &[u8]) -> Option<&'static str> {
    ACCEPTED_FIRMWARE.iter().find(|(known,_)| id == known).map(|(_,label)| *label)
}
fn validate_identity(id: &[u8]) -> Result<&'static str> {
    accepted_firmware(id).ok_or_else(|| {
        let raw = id.iter().take(14).map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ");
        let extra = if id.len()>14 { " …" } else { "" };
        format!("Firmware não validada. Esperado B5 0.7.1 ou 0.7.3 / biblioteca 2.43.2 / engine 2.23.2. Identidade lida ({} bytes): {raw}{extra}.",id.len())
    })
}
pub fn validate_mode_and_name(mode: &[u8], name: &[u8]) -> Result<()> {
    if mode.len()!=7 || mode[..4]!=[0,255,0,6] || mode[5]!=0 || mode[6]!=1 || name!=b"HunXiang" {
        return Err("Modo ou fluxo interno não validado".into());
    }
    Ok(())
}
pub fn validate_flow(total: usize, graph: &[u8], complete: bool) -> Result<()> {
    if !complete || graph.len()!=total || total!=GRAPH_LENGTH || Sha256::digest(graph)[..]!=GRAPH_SHA256 {
        return Err("O fluxo interno difere do aparelho investigado".into());
    }
    Ok(())
}
#[derive(Default)]
struct FirmwareState { identity: Option<[u8;14]>, label: &'static str }
impl FirmwareState {
    fn accepted(id: &[u8]) -> Result<Self> {
        let label=validate_identity(id)?;
        Ok(Self { identity:Some(id.try_into().map_err(|_| "Identidade incompleta")?),label })
    }
    fn check(&mut self, current: Result<Vec<u8>>) -> Result<()> {
        let result=current.and_then(|id| {
            if self.identity.as_ref().is_some_and(|known| id==*known) { Ok(()) }
            else { Err("Identidade alterada; reconexão e validação completa necessárias".into()) }
        });
        if result.is_err() { *self=Self::default(); }
        result
    }
}

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

pub struct HidDevice { handle: Handle, serial: String, query_interval_ms: u64, firmware: FirmwareState }
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
        let mut device = Self { handle, serial: String::new(), query_interval_ms: 8, firmware: FirmwareState::default() };
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
        self.exchange_with_wait(op, request, sequence, self.query_interval_ms)
    }

    /// Select the polling interval for control responses; the response budget stays 800 ms.
    /// This changes read polling only, never the pacing between parameter writes.
    pub fn set_query_interval(&mut self, interval_ms: u64) -> Result<()> {
        if !(8..=80).contains(&interval_ms) { return Err("Intervalo de consulta permitido: 8 a 80 ms".into()); }
        self.query_interval_ms = interval_ms;
        Ok(())
    }

    pub fn meter_with_wait(&mut self, op: u8, wait_ms: u64) -> Result<u16> {
        if !matches!(op, 0x91 | 0x82) || !(4..=80).contains(&wait_ms) { return Err("Consulta de medidor inválida".into()); }
        let p = self.exchange_with_wait(op, &[], None, wait_ms)?;
        if p.len() != 5 || p[..3] != [255, 1, 0] { return Err("Resposta de medidor inválida".into()); }
        let value = u16::from_le_bytes([p[3], p[4]]);
        if value > 32767 { return Err("Amplitude de medidor inválida".into()); }
        Ok(value)
    }

    /// Check the exact identity accepted for this connection, not merely membership.
    pub fn check_identity(&mut self) -> Result<()> {
        let current=self.query(0,&[]);
        self.firmware.check(current)
    }
    /// Read-only internal flow: declared length, bytes and complete end marker.
    pub fn read_flow(&mut self) -> Result<(usize,Vec<u8>,bool)> {
        let mut graph=Vec::new(); let mut total=0;
        for seq in 0..=255u8 {
            let p=self.exchange(0x80,&[2],Some(seq))?;
            if seq==0 {
                if p.len()<7 || p[1..4]!=[255,0,2] { return Err("Cabeçalho do fluxo inválido".into()); }
                total=u16::from_le_bytes([p[4],p[5]]) as usize;
                graph.extend_from_slice(&p[6..p.len()-1]);
            } else {
                if p.len()<2 { return Err("Pacote do fluxo incompleto".into()); }
                graph.extend_from_slice(&p[1..p.len()-1]);
            }
            if p.last()==Some(&1) { return Ok((total,graph,true)); }
            if p.last()!=Some(&0) { return Err("Marcador do fluxo inválido".into()); }
        }
        Ok((total,graph,false))
    }

    fn exchange_with_wait(&mut self, op: u8, request: &[u8], sequence: Option<u8>, wait_ms: u64) -> Result<Vec<u8>> {
        if op == 0x80 && sequence.is_none() && request.is_empty() { return Err("Consulta de modo sem seletor".into()); }
        self.send(op, request)?;
        let response_budget = Duration::from_millis(800);
        let started = Instant::now();
        for _ in 0..(800 / wait_ms) {
            if started.elapsed() >= response_budget { break; }
            thread::sleep(Duration::from_millis(wait_ms));
            if started.elapsed() >= response_budget { break; }
            let mut raw = [0u8; 257];
            if unsafe { HidD_GetInputReport(self.handle, raw.as_mut_ptr().cast(), 257) } == 0 { return Err(win_error("Leitura HID falhou")); }
            if started.elapsed() >= response_budget { break; }
            if let Ok(p) = protocol::decode(&raw, op) {
                if !matches_reply(op, request, sequence, p) { continue; }
                return Ok(p.to_vec());
            }
        }
        Err("O AM8 não confirmou a resposta esperada".into())
    }
}

fn matches_reply(op: u8, request: &[u8], sequence: Option<u8>, payload: &[u8]) -> bool {
    if let Some(seq) = sequence {
        if payload.first() != Some(&seq) || payload.len() < 2 || !matches!(payload.last(), Some(0 | 1)) { return false; }
        // Mode and flow share opcode 0x80 and both can start with zero. A slow
        // mode reply must not be consumed as the first packet of the flow.
        return seq != 0 || (payload.len() >= 7 && payload[1..3] == [255, 0] && payload.get(3) == request.first());
    }
    op != 0x80 || (payload.len() >= 5 && payload[..3] == [0, 255, 0] && payload.get(3) == request.first() && payload.last() == Some(&1))
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
    fn write_dac_mode(&mut self, mode: u16) -> Result<()> {
        protocol::validate_dac_mode(mode)?;
        let bytes = mode.to_le_bytes();
        self.send(0x09, &[7, bytes[0], bytes[1]])?;
        thread::sleep(Duration::from_millis(120));
        Ok(())
    }
    fn guard(&mut self) -> Result<()> {
        self.firmware=FirmwareState::default();
        let id=self.query(0,&[])?;
        validate_identity(&id)?;
        let mode=self.query(0x80,&[6])?;
        let name=self.query(0xFC,&[])?;
        validate_mode_and_name(&mode,&name)?;
        let (total,graph,complete)=self.read_flow()?;
        validate_flow(total,&graph,complete)?;
        self.firmware=FirmwareState::accepted(&id)?;
        Ok(())
    }
    fn firmware(&self) -> &'static str { self.firmware.label }
}

impl Drop for HidDevice { fn drop(&mut self) { unsafe { CloseHandle(self.handle); } } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn supported_identity_is_exact_including_library_and_engine() {
        for (id,label) in ACCEPTED_FIRMWARE {
            assert_eq!(accepted_firmware(&id),Some(label));
            for n in 0..14 { assert!(accepted_firmware(&id[..n]).is_none()); }
            let mut extra=id.to_vec();extra.push(0);assert!(accepted_firmware(&extra).is_none());
            for i in (0..14).filter(|i| *i!=3) { let mut altered=id;altered[i]^=1;assert!(accepted_firmware(&altered).is_none()); }
            for p in [0,2,4,255] { let mut altered=id;altered[3]=p;assert!(accepted_firmware(&altered).is_none()); }
        }
    }
    #[test] fn unsupported_identity_error_is_bounded_and_shows_received_bytes() {
        assert!(validate_identity(&[0x42,7]).unwrap_err().contains("2 bytes): 42 07"));
        let error=validate_identity(&vec![0xab;4096]).unwrap_err();
        assert_eq!(error.matches("ab").count(),14);assert!(error.len()<300);
    }
    #[test] fn periodic_check_rejects_switch_between_both_supported_revisions() {
        for (id,label) in ACCEPTED_FIRMWARE {
            let mut state=FirmwareState::accepted(&id).unwrap();assert_eq!(state.label,label);
            state.check(Ok(id.to_vec())).unwrap();
            let other=ACCEPTED_FIRMWARE.iter().find(|(known,_)| *known!=id).unwrap().0;
            assert!(state.check(Ok(other.to_vec())).is_err());assert!(state.label.is_empty());
            assert!(state.check(Ok(id.to_vec())).is_err());
        }
    }
    #[test] fn periodic_failure_clears_identity() {
        let mut state=FirmwareState::accepted(&ACCEPTED_FIRMWARE[1].0).unwrap();
        assert!(state.check(Err("USB failure".into())).is_err());assert!(state.identity.is_none());assert!(state.label.is_empty());
    }
    #[test] fn mode_name_complete_length_and_hash_are_still_required() {
        let mode=[0,255,0,6,0,0,1];assert!(validate_mode_and_name(&mode,b"HunXiang").is_ok());
        for i in [0,1,2,3,5,6] { let mut altered=mode;altered[i]^=1;assert!(validate_mode_and_name(&altered,b"HunXiang").is_err()); }
        assert!(validate_mode_and_name(&mode[..6],b"HunXiang").is_err());assert!(validate_mode_and_name(&mode,b"HunXiang\0").is_err());
        let altered=vec![0;GRAPH_LENGTH];
        assert!(validate_flow(GRAPH_LENGTH,&altered,false).is_err());
        assert!(validate_flow(GRAPH_LENGTH+1,&altered,true).is_err());
        assert!(validate_flow(GRAPH_LENGTH,&altered[..GRAPH_LENGTH-1],true).is_err());
        assert!(validate_flow(GRAPH_LENGTH,&altered,true).is_err());
    }

    #[test]
    fn flow_polling_rejects_previous_mode_response() {
        let mode = [0, 255, 0, 6, 0, 0, 1];
        assert!(matches_reply(0x80, &[6], None, &mode));
        assert!(!matches_reply(0x80, &[2], Some(0), &mode));
        assert!(!matches_reply(0x80, &[2], Some(0), &[0, 255, 0, 2]));
        assert!(matches_reply(0x80, &[2], Some(0), &[0, 255, 0, 2, 115, 11, 0]));
    }
    #[test]
    fn flow_polling_requires_sequence_and_valid_marker() {
        assert!(!matches_reply(0x80, &[2], Some(2), &[1, 42, 0]));
        assert!(!matches_reply(0x80, &[2], Some(2), &[2, 42, 2]));
        assert!(matches_reply(0x80, &[2], Some(2), &[2, 42, 1]));
    }
}
