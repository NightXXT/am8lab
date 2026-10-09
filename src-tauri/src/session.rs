use crate::protocol::{self, Block, Effect, Result, Transport, read_block, set_block, validate_block};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::{self, File}, io::Read, path::{Path, PathBuf}};

const MAX_JOURNAL_BYTES: usize = 64 * 1024;
const MAX_SERIAL_UNITS: usize = 126;

fn validate_serial(serial: &str) -> Result<()> {
    if serial.trim().is_empty() || serial.trim() != serial
        || serial.encode_utf16().count() > MAX_SERIAL_UNITS
        || serial.chars().any(char::is_control) || serial.contains('\u{fffd}') {
        return Err("Número de série inválido".into());
    }
    Ok(())
}

type Blocks = BTreeMap<Block, Vec<i16>>;
// Preparation reads belong to one guarded operation only. Never retain these
// values across commands: another tool or the device can change its state.
fn prepared_block(t: &mut impl Transport, blocks: &mut Blocks, block: Block) -> Result<Vec<i16>> {
    if let Some(words) = blocks.get(&block) { return Ok(words.clone()); }
    let words = read_block(t, block)?;
    blocks.insert(block, words.clone());
    Ok(words)
}
#[derive(Clone, Deserialize, Serialize)]
struct Journal {
    version: u32,
    originals: Blocks,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    serial: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    original_headphone_mode: Option<u16>,
}

#[derive(Clone)]
struct Comparison { blocks: Blocks, headphone_mode: Option<u16> }

#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub device: &'static str,
    pub firmware: &'static str,
    pub serial: String,
    pub effects: Blocks,
    pub gain_db: f64,
    pub recovery_pending: bool,
    pub comparison: bool,
    pub legacy_pending: bool,
    pub headphone_mode: Option<String>,
}

pub struct Session {
    path: PathBuf,
    journal: Journal,
    comparison: Option<Comparison>,
}

impl Session {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let journal = if path.exists() {
            let mut bytes = Vec::new();
            File::open(&path).map_err(|e| e.to_string())?
                .take((MAX_JOURNAL_BYTES + 1) as u64)
                .read_to_end(&mut bytes).map_err(|e| e.to_string())?;
            if bytes.len() > MAX_JOURNAL_BYTES { return Err("Arquivo de recuperação excessivo; preservado".into()); }
            let data: Journal = serde_json::from_slice(&bytes).map_err(|e| format!("Recuperação inválida, arquivo preservado: {e}"))?;
            if !(1..=4).contains(&data.version) { return Err("Versão de recuperação desconhecida".into()); }
            for (block, words) in &data.originals { validate_block(*block, words)?; }
            if let Some(mode) = data.original_headphone_mode {
                if data.version != 4 { return Err("Modo dos fones em versão de recuperação incompatível; arquivo preservado".into()); }
                protocol::validate_dac_mode(mode)?;
            }
            if (!data.originals.is_empty() || data.original_headphone_mode.is_some()) && data.serial.is_none() {
                return Err("Recuperação pendente sem número de série; arquivo preservado. Não é seguro associá-la automaticamente ao microfone conectado.".into());
            }
            if let Some(serial) = data.serial.as_deref() {
                validate_serial(serial).map_err(|e| format!("{e} na recuperação; arquivo preservado"))?;
            }
            data
        } else { Journal { version: 1, originals: Blocks::new(), serial: None, original_headphone_mode: None } };
        Ok(Self { path, journal, comparison: None })
    }

    pub fn pending(&self) -> bool { !self.journal.originals.is_empty() || self.journal.original_headphone_mode.is_some() }
    fn legacy_pending(&self) -> bool { self.journal.originals.contains_key(&Block::Autotune) || self.journal.originals.contains_key(&Block::Voice) || self.journal.originals.contains_key(&Block::AutoRoute) }
    fn guard(&self, t: &mut impl Transport) -> Result<()> {
        t.guard()?;
        validate_serial(t.serial())?;
        if self.pending() {
            let serial = self.journal.serial.as_deref().ok_or("Recuperação sem identidade; operação recusada")?;
            validate_serial(serial)?;
            if serial != t.serial() {
                return Err("A recuperação pertence a outro microfone; operação recusada".into());
            }
        }
        Ok(())
    }
    fn save(&self) -> Result<()> {
        let parent = self.path.parent().ok_or("Caminho de recuperação inválido")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let temp = self.path.with_extension("rust.tmp");
        let mut file = File::create(&temp).map_err(|e| e.to_string())?;
        let empty = Journal { version: 1, originals: Blocks::new(), serial: None, original_headphone_mode: None };
        let persisted = if !self.pending() { &empty } else { &self.journal };
        serde_json::to_writer_pretty(&mut file, persisted).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        replace_file(&temp, &self.path)
    }

    pub fn inspect(&self, t: &mut impl Transport) -> Result<Snapshot> {
        self.guard(t)?;
        self.snapshot(t)
    }
    fn snapshot(&self, t: &mut impl Transport) -> Result<Snapshot> {
        let mut effects = Blocks::new();
        for block in [Block::Noise, Block::Pitch, Block::Eq, Block::PlaybackEq, Block::Compressor, Block::PitchPro, Block::VoicePro,
            Block::ReverbNative, Block::PlateNative, Block::FeedbackFine, Block::WetRoute, Block::MainRoute, Block::VoiceRoute,
            Block::EchoEq, Block::Autotune, Block::Voice, Block::AutoRoute] { effects.insert(block, read_block(t, block)?); }
        let gain = read_block(t, Block::MicGain)?;
        let legacy_active = effects[&Block::Autotune][0] != 0 || effects[&Block::Voice][0] != 0 || effects[&Block::AutoRoute][1] != 1;
        let headphone_mode = protocol::read_dac_mode(t).ok().map(|mode| if mode == 0 { "stereo" } else { "mono" }.into());
        Ok(Snapshot { device: "FIFINE AM8 USB • 3142:A010", firmware: "B5 0.7.1", serial: t.serial().into(), effects,
            gain_db: f64::from(gain[2]) / 100.0, recovery_pending: self.pending(), comparison: self.comparison.is_some(),
            legacy_pending: self.legacy_pending() || legacy_active, headphone_mode })
    }

    fn ready(&self, t: &mut impl Transport) -> Result<Blocks> {
        self.guard(t)?;
        if self.comparison.is_some() { return Err("Volte aos efeitos antes de aplicar ajustes".into()); }
        if self.legacy_pending() { return Err("Restaure os ajustes da versão antiga antes de continuar".into()); }
        let mut prepared = Blocks::new();
        if prepared_block(t, &mut prepared, Block::Autotune)?[0] != 0 || prepared_block(t, &mut prepared, Block::Voice)?[0] != 0 || prepared_block(t, &mut prepared, Block::AutoRoute)?[1] != 1 {
            return Err("O caminho de afinação antigo está ativo. Restaure pela versão anterior antes de continuar.".into());
        }
        for block in [Block::PitchPro,Block::VoicePro,Block::ReverbNative,Block::PlateNative,Block::FeedbackFine] {
            if prepared_block(t, &mut prepared, block)?[0] != 0 && !self.journal.originals.contains_key(&block) {
                return Err("Um processador externo à sessão está ativo; restaure com a ferramenta que o ativou".into());
            }
        }
        for (block,expected) in [(Block::MainRoute,vec![1,0,600]),(Block::VoiceRoute,vec![1,1,0]),(Block::WetRoute,vec![1,1,0])] {
            if !self.journal.originals.contains_key(&block) && prepared_block(t, &mut prepared, block)?!=expected {
                return Err("Um canal de áudio fora da sessão está alterado; operações recusadas".into());
            }
        }
        Ok(prepared)
    }

    fn commit(t: &mut impl Transport, targets: &Blocks) -> Result<()> {
        let routes: Vec<_> = targets.keys().copied().filter(|b| matches!(b, Block::MainRoute | Block::VoiceRoute | Block::AutoRoute | Block::WetRoute)).collect();
        for block in &routes {
            let current = read_block(t, *block)?;
            set_block(t, *block, &[1, 1, current[2]])?;
        }
        // Disable serial/alternative processors before reconciling their parameters.
        // This also prevents both pitch algorithms or both reverbs running briefly.
        for block in [Block::Pitch,Block::PitchPro,Block::VoicePro,Block::ReverbNative,Block::PlateNative] {
            if targets.contains_key(&block) {
                let current = read_block(t, block)?;
                if current[0] != 0 {
                    // Disabling may replace the runtime context with preset defaults.
                    // Confirm only the disabled state here; reconcile final parameters below.
                    t.write_word(block,0,0)?;
                    let mut disabled=false;
                    for _ in 0..8 {
                        t.wait(250);
                        if read_block(t,block)?[0]==0 { disabled=true;break; }
                    }
                    if !disabled { return Err(format!("O processador não confirmou a desativação: {block:?}")); }
                }
            }
        }
        for (block, target) in targets {
            if !routes.contains(block) { set_block(t, *block, target)?; }
        }
        for block in routes { set_block(t, block, &targets[&block])?; }
        Ok(())
    }

    fn transaction(&mut self, t: &mut impl Transport, targets: Blocks) -> Result<()> {
        self.prepared_transaction(t, targets, Blocks::new())
    }

    fn prepared_transaction(&mut self, t: &mut impl Transport, targets: Blocks, mut prepared: Blocks) -> Result<()> {
        self.prepared_transaction_with_mode(t, targets, &mut prepared, None).map(|_| ())
    }

    fn prepared_transaction_with_mode(&mut self, t: &mut impl Transport, targets: Blocks, prepared: &mut Blocks, headphone_mode: Option<u16>) -> Result<Option<[u16; 14]>> {
        for (block, target) in &targets { validate_block(*block, target)?; }
        let mut before = Blocks::new();
        for block in targets.keys() { before.insert(*block, prepared_block(t, prepared, *block)?); }
        let before_dac = if let Some(mode) = headphone_mode {
            protocol::validate_dac_mode(mode)?;
            let words = protocol::read_dac_words(t)?;
            protocol::validate_dac_mode(words[7])?;
            Some(words)
        } else { None };
        let saved = self.journal.clone();
        for (block, words) in &before { self.journal.originals.entry(*block).or_insert_with(|| words.clone()); }
        if let Some(words) = before_dac { self.journal.original_headphone_mode.get_or_insert(words[7]); }
        // Older executables reject v4 instead of silently dropping DAC recovery.
        self.journal.version = if self.journal.original_headphone_mode.is_some() { 4 } else { 3 };
        self.journal.serial = Some(t.serial().into());
        if let Err(error) = self.save() { self.journal = saved; return Err(error); }
        // All originals are durable before changing either effects or the DAC.
        let result = Self::commit(t, &targets).and_then(|()| {
            if let Some(mode) = headphone_mode {
                protocol::set_dac_mode(t, mode)?;
                let mut expected = before_dac.ok_or("Referência do DAC ausente")?;
                expected[7] = mode;
                if protocol::read_dac_words(t)? != expected { return Err("Outros campos do DAC mudaram durante a operação".into()); }
            }
            Ok(())
        });
        if let Err(error) = result {
            let recovery = Self::commit(t, &before);
            let dac_recovery = if let Some(words) = before_dac {
                protocol::set_dac_mode(t, words[7]).and_then(|()| {
                    if protocol::read_dac_words(t)? != words { return Err("Outros campos do DAC diferem do estado anterior".into()); }
                    Ok(())
                })
            } else { Ok(()) };
            if let Err(recovery) = recovery.and(dac_recovery) { return Err(format!("{error}. Restauração pendente: {recovery}")); }
            return Err(error);
        }
        Ok(before_dac)
    }

    pub fn apply_headphone_mode(&mut self, t: &mut impl Transport, mode: &str) -> Result<Snapshot> {
        let target = match mode {
            "stereo" => 0, "mono" => 2,
            _ => return Err("Modo inválido; escolha estéreo ou mono para os fones".into()),
        };
        let mut prepared = self.ready(t)?;
        let before = protocol::read_dac_words(t)?;
        protocol::validate_dac_mode(before[7])?;
        if before[7] == target { return self.snapshot(t); }
        let before = self.prepared_transaction_with_mode(t, Blocks::new(), &mut prepared, Some(target))?
            .ok_or("Referência do DAC ausente")?;
        match self.snapshot(t) {
            Ok(snapshot) if snapshot.headphone_mode.as_deref() == Some(mode) => Ok(snapshot),
            result => {
                let error = result.err().unwrap_or_else(|| "O modo dos fones não confirmou a leitura final".into());
                if let Err(recovery) = protocol::set_dac_mode(t, before[7]) {
                    return Err(format!("{error}. Restauração pendente: {recovery}"));
                }
                if protocol::read_dac_words(t)? != before { return Err(format!("{error}. Outros campos do DAC mudaram; recuperação preservada")); }
                Err(error)
            }
        }
    }

    pub fn apply(&mut self, t: &mut impl Transport, effect: Effect, enabled: bool, values: BTreeMap<String, i32>) -> Result<Snapshot> {
        let mut prepared = self.ready(t)?;
        if enabled && matches!(effect,Effect::Pitch | Effect::Pitchpro | Effect::Voicepro | Effect::Reverb | Effect::Plate) {
            let allowed_alternatives=match effect {
                Effect::Pitch=>vec![Block::PitchPro], Effect::Pitchpro=>vec![Block::Pitch],
                Effect::Voicepro=>vec![Block::Pitch,Block::PitchPro],
                Effect::Reverb=>vec![Block::PlateNative],Effect::Plate=>vec![Block::ReverbNative],_=>vec![]
            };
            for block in [Block::Pitch,Block::PitchPro,Block::VoicePro,Block::ReverbNative,Block::PlateNative] {
                if block!=effect.block() && !allowed_alternatives.contains(&block) && prepared_block(t, &mut prepared, block)?[0]!=0 {
                    return Err("Use apenas um entre Tom da voz, Transformação Pro e Reverberação. Desative o atual antes de aplicar outro.".into());
                }
            }
        }
        let before = prepared_block(t, &mut prepared, effect.block())?;
        let target = protocol::target(effect, enabled, &values, &before)?;
        let mut targets = Blocks::from([(effect.block(), target)]);
        if effect==Effect::Feedback && enabled {
            let basic=t.query(0x93,&[])?;
            if basic.len()!=5 || basic[..3]!=[255,0,0] { return Err("Outro supressor de microfonia está ativo ou não respondeu".into()); }
        }
        if matches!(effect,Effect::Pitch | Effect::Pitchpro) {
            if enabled && prepared_block(t, &mut prepared, Block::VoicePro)?[0]!=0 { return Err("Desative a transformação de voz antes de ajustar o tom".into()); }
            let other = if effect==Effect::Pitch { Block::PitchPro } else { Block::Pitch };
            let mut words=prepared_block(t, &mut prepared, other)?; words[0]=0; targets.insert(other,words);
        }
        if effect==Effect::Voicepro {
            for block in [Block::Pitch,Block::PitchPro] { let mut words=prepared_block(t, &mut prepared, block)?; words[0]=0; targets.insert(block,words); }
            let original_main=match self.journal.originals.get(&Block::MainRoute) {
                Some(words)=>words.clone(), None=>prepared_block(t, &mut prepared, Block::MainRoute)?
            };
            let original_voice=match self.journal.originals.get(&Block::VoiceRoute) {
                Some(words)=>words.clone(), None=>prepared_block(t, &mut prepared, Block::VoiceRoute)?
            };
            if original_main[..2] != [1,0] || original_voice[..2] != [1,1] { return Err("Caminho original de voz inesperado".into()); }
            let mut main=original_main.clone(); let mut voice=original_voice;
            if enabled { main[1]=1; voice[1]=0; voice[2]=original_main[2]; }
            targets.insert(Block::MainRoute,main); targets.insert(Block::VoiceRoute,voice);
        }
        if matches!(effect,Effect::Reverb | Effect::Plate) {
            let other=if effect==Effect::Reverb { Block::PlateNative } else { Block::ReverbNative };
            let mut words=prepared_block(t, &mut prepared, other)?; words[0]=0; targets.insert(other,words);
            let original_route=match self.journal.originals.get(&Block::WetRoute) {
                Some(words)=>words.clone(), None=>prepared_block(t, &mut prepared, Block::WetRoute)?
            };
            let original_eq=match self.journal.originals.get(&Block::EchoEq) {
                Some(words)=>words.clone(), None=>prepared_block(t, &mut prepared, Block::EchoEq)?
            };
            if original_route[..2]!=[1,1] { return Err("Mistura original diferente da configuração testada".into()); }
            let mut eq=original_eq.clone();
            let route=if enabled {
                eq[..3].copy_from_slice(&[1,-24576,0]); for i in (3..53).step_by(5) { eq[i]=0; }
                vec![1,0,(values["wet_gain"]*100) as i16]
            } else { eq=original_eq; original_route };
            targets.insert(Block::EchoEq,eq); targets.insert(Block::WetRoute,route);
        }
        self.prepared_transaction(t, targets, prepared)?;
        self.snapshot(t)
    }

    pub fn compare(&mut self, t: &mut impl Transport) -> Result<Snapshot> {
        self.guard(t)?;
        if !self.pending() { return Err("Aplique um efeito antes de comparar".into()); }
        if self.legacy_pending() { return Err("Restaure a sessão antiga antes de comparar".into()); }
        if let Some(targets) = self.comparison.clone() {
            self.prepared_transaction_with_mode(t, targets.blocks, &mut Blocks::new(), targets.headphone_mode)?;
            self.comparison = None;
        } else {
            let mut current = Blocks::new();
            for block in self.journal.originals.keys() { current.insert(*block, read_block(t, *block)?); }
            let current_mode = if self.journal.original_headphone_mode.is_some() { Some(protocol::read_dac_mode(t)?) } else { None };
            self.prepared_transaction_with_mode(t, self.journal.originals.clone(), &mut Blocks::new(), self.journal.original_headphone_mode)?;
            self.comparison = Some(Comparison { blocks: current, headphone_mode: current_mode });
        }
        self.snapshot(t)
    }

    pub fn restore(&mut self, t: &mut impl Transport) -> Result<Snapshot> {
        self.guard(t)?;
        Self::commit(t, &self.journal.originals)?;
        if let Some(mode) = self.journal.original_headphone_mode { protocol::set_dac_mode(t, mode)?; }
        // Preserve the full journal on any failure, including a failed clear.
        let saved = self.journal.clone();
        self.journal = Journal { version: 1, originals: Blocks::new(), serial: None, original_headphone_mode: None };
        if let Err(error) = self.save() { self.journal = saved; return Err(error); }
        self.comparison = None;
        self.snapshot(t)
    }

    pub fn test_gain(&mut self, t: &mut impl Transport, db: i32) -> Result<Snapshot> {
        if !(1..=12).contains(&db) { return Err("Redução permitida: 1 a 12 dB".into()); }
        self.ready(t)?;
        if self.journal.originals.contains_key(&Block::MicGain) { return Err("Há recuperação de ganho pendente; restaure primeiro".into()); }
        let before = read_block(t, Block::MicGain)?;
        let value = i32::from(before[2]) - db * 100;
        if before[..2] != [1, 0] || before[2] > 0 || value < -9000 { return Err("Ganho fora do intervalo validado".into()); }
        self.transaction(t, Blocks::from([(Block::MicGain, vec![1, 0, value as i16])]))?;
        t.wait(5000);
        set_block(t, Block::MicGain, &before)?;
        let baseline = self.journal.originals.remove(&Block::MicGain);
        if let Err(error) = self.save() {
            if let Some(words) = baseline { self.journal.originals.insert(Block::MicGain, words); }
            return Err(error);
        }
        self.snapshot(t)
    }
}

#[cfg(windows)] fn replace_file(from: &Path, to: &Path) -> Result<()> { crate::windows::atomic_replace(from, to) }
#[cfg(not(windows))] fn replace_file(from: &Path, to: &Path) -> Result<()> { fs::rename(from, to).map_err(|e| e.to_string()) }

#[cfg(test)]
mod tests {
    use super::*;
    struct Fake { words: Blocks, preset: Vec<i16>, fail: Option<(Block, usize, i16)>, journal: PathBuf, serial: String, lose: bool, writes: usize,
        written_blocks: Vec<Block>, queried_blocks: Vec<Block>, external_on_write: Option<(Block, Vec<i16>)>,
        dac: [u16; 14], dac_writes: Vec<u16>, dac_fail_once: bool, dac_ignore_once: bool, dac_corrupt_once: bool, guard_calls: usize }
    impl Transport for Fake {
        fn guard(&mut self) -> Result<()> { self.guard_calls += 1; if self.lose { Err("disconnected".into()) } else { Ok(()) } }
        fn serial(&self) -> &str { &self.serial }
        fn wait(&self, _: u64) {}
        fn query(&mut self, op: u8, _: &[u8]) -> Result<Vec<u8>> {
            if self.lose { return Err("disconnected".into()); }
            if op == 0x09 { return Ok([vec![255], self.dac.iter().flat_map(|v| v.to_le_bytes()).collect()].concat()); }
            let block = *self.words.keys().find(|block| block.address() == op).unwrap();
            self.queried_blocks.push(block);
            let words = &self.words[&block];
            Ok([vec![255], words.iter().flat_map(|v| v.to_le_bytes()).collect()].concat())
        }
        fn write_word(&mut self, block: Block, index: usize, value: i16) -> Result<()> {
            self.writes += 1;
            self.written_blocks.push(block);
            assert!(self.journal.exists(), "recovery precedes mutation");
            if self.lose { return Err("disconnected".into()); }
            if self.fail == Some((block, index, value)) { self.fail = None; return Err("USB failure".into()); }
            let words = self.words.get_mut(&block).unwrap();
            if block == Block::Compressor && index != 0 && words[0] == 0 { self.preset[index] = value; }
            else {
                if block == Block::Compressor && index == 0 && value == 1 { words[1..].copy_from_slice(&self.preset[1..]); }
                words[index] = value;
                if block==Block::VoicePro && index==0 && value==0 { words[1..].copy_from_slice(&[200,130]); }
                if block == Block::Compressor && index != 0 { self.preset[index] = value; }
            }
            if let Some((external_block, words)) = self.external_on_write.take() { self.words.insert(external_block, words); }
            Ok(())
        }
        fn write_dac_mode(&mut self, mode: u16) -> Result<()> {
            protocol::validate_dac_mode(mode)?;
            assert!(self.guard_calls > 0, "full guard precedes DAC mutation");
            let persisted = Session::open(&self.journal).expect("durable recovery precedes DAC mutation");
            assert_eq!(persisted.journal.version, 4);
            assert!(persisted.journal.original_headphone_mode.is_some());
            assert_eq!(persisted.journal.serial.as_deref(), Some(self.serial.as_str()));
            self.dac_writes.push(mode);
            if self.lose { return Err("disconnected".into()); }
            if self.dac_fail_once { self.dac_fail_once = false; return Err("DAC failure".into()); }
            if self.dac_ignore_once { self.dac_ignore_once = false; return Ok(()); }
            self.dac[7] = mode;
            if self.dac_corrupt_once { self.dac_corrupt_once = false; self.dac[3] -= 1; }
            Ok(())
        }
    }
    fn setup() -> (Session, Fake) {
        static ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let path = std::env::temp_dir().join(format!("am8-rust-test-{}-{}.json", std::process::id(), ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
        let original: serde_json::Value = serde_json::from_str(include_str!("../data/test-baseline.json")).unwrap();
        let mut words: Blocks = serde_json::from_value(original["effects"].clone()).unwrap();
        words.insert(Block::MicGain, vec![1, 0, -300]);
        for (block,values) in [(Block::PitchPro,vec![0,0]),(Block::VoicePro,vec![0,200,130]),
            (Block::ReverbNative,vec![0,50,0,100,0,50,0]),(Block::PlateNative,vec![0,12000,1,2500,60,65,5000,60]),
            (Block::FeedbackFine,vec![0,512,5120]),(Block::WetRoute,vec![1,1,0])] { words.insert(block,values); }
        words.insert(Block::EchoEq,words[&Block::Eq].clone());
        let preset = words[&Block::Compressor].clone();
        let session = Session::open(&path).unwrap();
        (session, Fake { words, preset, fail: None, journal: path, serial: "TEST-AM8-001".into(), lose: false, writes: 0, written_blocks: vec![],
            queried_blocks: vec![], external_on_write: None,
            dac: [3,7,0,4095,4095,0,3,0,0,0,0,5,0,0], dac_writes: vec![],
            dac_fail_once: false, dac_ignore_once: false, dac_corrupt_once: false, guard_calls: 0 })
    }
    impl Drop for Fake { fn drop(&mut self) { let _ = fs::remove_file(&self.journal); } }
    fn vals(items: &[(&str, i32)]) -> BTreeMap<String, i32> { items.iter().map(|(k,v)| ((*k).into(), *v)).collect() }
    #[test] fn headphone_mode_only_changes_the_validated_dac_field_and_recovers_after_restart() {
        let (mut s, mut t) = setup(); let original = t.words.clone(); let dac = t.dac;
        let snapshot = s.apply_headphone_mode(&mut t, "mono").unwrap();
        assert_eq!(snapshot.headphone_mode.as_deref(), Some("mono"));
        assert!(snapshot.recovery_pending); assert!(s.journal.originals.is_empty());
        assert_eq!(s.journal.original_headphone_mode, Some(0));
        assert_eq!(t.words, original); assert!(t.written_blocks.is_empty());
        for i in 0..14 { if i != 7 { assert_eq!(t.dac[i], dac[i]); } }
        let mut restarted = Session::open(&t.journal).unwrap();
        assert!(restarted.pending());
        let restored = restarted.restore(&mut t).unwrap();
        assert_eq!(restored.headphone_mode.as_deref(), Some("stereo"));
        assert!(!restarted.pending()); assert_eq!(t.dac, dac); assert_eq!(t.words, original);
        assert_eq!(t.dac_writes, vec![2, 0]);
        assert_eq!(serde_json::from_slice::<Journal>(&fs::read(&t.journal).unwrap()).unwrap().version, 1);
    }
    #[test] fn headphone_mode_preserves_an_original_mono_baseline_across_reapplication() {
        let (mut s, mut t) = setup(); t.dac[7] = 2; let original = t.dac;
        s.apply_headphone_mode(&mut t, "stereo").unwrap();
        s.apply_headphone_mode(&mut t, "mono").unwrap();
        s.apply_headphone_mode(&mut t, "stereo").unwrap();
        assert_eq!(s.journal.original_headphone_mode, Some(2));
        Session::open(&t.journal).unwrap().restore(&mut t).unwrap();
        assert_eq!(t.dac, original); assert!(t.dac_writes.iter().all(|mode| matches!(mode, 0 | 2)));
    }
    #[test] fn headphone_and_voice_comparison_restore_both_paths_without_changing_unrelated_blocks() {
        let (mut s, mut t) = setup(); let original = t.words.clone(); let original_dac = t.dac;
        s.apply_headphone_mode(&mut t, "mono").unwrap();
        s.apply(&mut t, Effect::Pitch, true, vals(&[("pitch", -3)])).unwrap();
        assert_eq!(s.journal.version, 4, "an effect write must retain DAC recovery version");
        let active = t.words.clone(); let active_dac = t.dac;
        let compared = s.compare(&mut t).unwrap();
        assert!(compared.comparison); assert_eq!(t.words, original); assert_eq!(t.dac, original_dac);
        assert!(s.apply_headphone_mode(&mut t, "mono").is_err());
        s.compare(&mut t).unwrap(); assert_eq!(t.words, active); assert_eq!(t.dac, active_dac);
        Session::open(&t.journal).unwrap().restore(&mut t).unwrap();
        assert_eq!(t.words, original); assert_eq!(t.dac, original_dac);
        assert!(t.written_blocks.iter().all(|block| matches!(block, Block::Pitch | Block::PitchPro)));
    }
    #[test] fn headphone_only_comparison_retains_pending_recovery() {
        let (mut s, mut t) = setup();
        s.apply_headphone_mode(&mut t, "mono").unwrap();
        s.compare(&mut t).unwrap(); assert_eq!(t.dac[7], 0); assert!(s.pending());
        s.compare(&mut t).unwrap(); assert_eq!(t.dac[7], 2);
        s.restore(&mut t).unwrap(); assert_eq!(t.dac[7], 0); assert!(t.written_blocks.is_empty());
    }
    #[test] fn headphone_mode_rejects_bad_requests_unknown_modes_and_failed_guard_before_mutation() {
        let (mut s, mut t) = setup(); let original = t.words.clone(); let dac = t.dac;
        for mode in ["", "Mono", "swap", "phase", "1", "3"] { assert!(s.apply_headphone_mode(&mut t, mode).is_err()); }
        for mode in [1, 3, u16::MAX] { t.dac[7] = mode; assert!(s.apply_headphone_mode(&mut t, "mono").is_err()); }
        t.dac = dac; t.lose = true; assert!(s.apply_headphone_mode(&mut t, "mono").is_err());
        assert!(!t.journal.exists()); assert!(t.dac_writes.is_empty()); assert_eq!(t.words, original);
        t.lose = false;
        assert_eq!(s.apply_headphone_mode(&mut t, "stereo").unwrap().headphone_mode.as_deref(), Some("stereo"));
        assert!(!t.journal.exists()); assert!(!s.pending());
    }
    #[test] fn headphone_write_failures_or_unconfirmed_modes_roll_back_and_keep_recovery() {
        for ignore in [false, true] {
            let (mut s, mut t) = setup(); let original = t.dac;
            t.dac_fail_once = !ignore; t.dac_ignore_once = ignore;
            assert!(s.apply_headphone_mode(&mut t, "mono").is_err());
            assert_eq!(t.dac, original); assert!(s.pending());
            assert!(Session::open(&t.journal).unwrap().pending());
            s.restore(&mut t).unwrap(); assert!(!s.pending()); assert!(t.written_blocks.is_empty());
        }
    }
    #[test] fn headphone_readback_detects_unrelated_dac_changes_and_never_rewrites_them() {
        let (mut s, mut t) = setup(); let original = t.dac;
        t.dac_corrupt_once = true;
        let error = s.apply_headphone_mode(&mut t, "mono").unwrap_err();
        assert!(error.contains("Restauração pendente")); assert_eq!(t.dac[7], original[7]);
        assert_eq!(t.dac[3], original[3] - 1); assert!(s.pending());
        // Only the remembered mode is restored; externally changed volume survives.
        s.restore(&mut t).unwrap(); assert_eq!(t.dac[3], original[3] - 1);
        assert_eq!(t.dac_writes, vec![2, 0]); assert!(t.written_blocks.is_empty());
    }
    #[test] fn headphone_recovery_refuses_another_serial_and_survives_disconnect() {
        let (mut s, mut t) = setup(); let original = t.dac;
        s.apply_headphone_mode(&mut t, "mono").unwrap(); let writes = t.dac_writes.len();
        t.serial = "TEST-AM8-OTHER".into();
        assert!(Session::open(&t.journal).unwrap().restore(&mut t).is_err());
        assert_eq!(t.dac_writes.len(), writes);
        t.serial = "TEST-AM8-001".into(); t.lose = true;
        assert!(Session::open(&t.journal).unwrap().restore(&mut t).is_err());
        assert!(Session::open(&t.journal).unwrap().pending());
        t.lose = false; Session::open(&t.journal).unwrap().restore(&mut t).unwrap(); assert_eq!(t.dac, original);
    }
    #[test] fn headphone_comparison_failure_rolls_back_the_effects_already_restored() {
        let (mut s, mut t) = setup();
        s.apply(&mut t, Effect::Pitch, true, vals(&[("pitch", 2)])).unwrap();
        s.apply_headphone_mode(&mut t, "mono").unwrap(); let active = t.words.clone(); let dac = t.dac;
        t.dac_fail_once = true;
        assert!(s.compare(&mut t).is_err());
        assert_eq!(t.words, active); assert_eq!(t.dac, dac); assert!(s.comparison.is_none()); assert!(s.pending());
        s.restore(&mut t).unwrap();
    }
    #[test] fn headphone_journal_failure_prevents_all_hardware_writes() {
        let (mut s, mut t) = setup(); let original = t.dac;
        fs::write(&t.journal, b"blocking parent file").unwrap();
        s.path = t.journal.join("cannot-create.json");
        assert!(s.apply_headphone_mode(&mut t, "mono").is_err());
        assert_eq!(t.dac, original); assert!(t.dac_writes.is_empty()); assert!(!s.pending());
        assert!(t.written_blocks.is_empty());
    }
    #[test] fn headphone_journals_reject_unvalidated_modes_missing_serial_and_wrong_versions() {
        let (_s, t) = setup();
        for data in [
            serde_json::json!({"version":4,"originals":{},"original_headphone_mode":0}),
            serde_json::json!({"version":4,"originals":{},"original_headphone_mode":0,"serial":" "}),
            serde_json::json!({"version":4,"originals":{},"original_headphone_mode":1,"serial":"TEST-AM8-001"}),
            serde_json::json!({"version":4,"originals":{},"original_headphone_mode":3,"serial":"TEST-AM8-001"}),
            serde_json::json!({"version":3,"originals":{},"original_headphone_mode":0,"serial":"TEST-AM8-001"}),
            serde_json::json!({"version":5,"originals":{},"original_headphone_mode":0,"serial":"TEST-AM8-001"}),
        ] {
            let bytes = serde_json::to_vec(&data).unwrap(); fs::write(&t.journal, &bytes).unwrap();
            assert!(Session::open(&t.journal).is_err()); assert_eq!(fs::read(&t.journal).unwrap(), bytes);
        }
        assert!(t.dac_writes.is_empty());
    }
    #[test] fn snapshot_reads_each_block_once_including_legacy_state() {
        let (s, mut t) = setup();
        let snapshot = s.inspect(&mut t).unwrap();
        assert_eq!(snapshot.effects.len(), 17);
        assert_eq!(t.queried_blocks.len(), 18, "one read per effect and microphone gain");
        for block in t.words.keys() {
            assert_eq!(t.queried_blocks.iter().filter(|read| *read == block).count(), 1, "duplicate read: {block:?}");
        }
        t.words.get_mut(&Block::AutoRoute).unwrap()[1] = 0;
        assert!(s.inspect(&mut t).unwrap().legacy_pending, "a new command must see external changes");
    }
    #[test] fn pitch_apply_reduces_queries_without_reusing_previous_command_state() {
        let (mut s, mut t) = setup();
        let original = t.words.clone();
        let snapshot = s.apply(&mut t, Effect::Pitch, true, vals(&[("pitch", -3)])).unwrap();
        // The previous path used 45 effect reads for this immediate-response
        // device. Keep full post-write inspection while removing redundant
        // preparation reads; a simulated 20 ms query costs <=700 ms instead of 900 ms.
        assert!(t.queried_blocks.len() <= 35, "{} queries", t.queried_blocks.len());
        assert_eq!(snapshot.effects[&Block::Pitch], t.words[&Block::Pitch]);
        let writes = t.writes;
        t.words.get_mut(&Block::VoicePro).unwrap()[0] = 1;
        assert!(s.apply(&mut t, Effect::Pitch, true, vals(&[("pitch", 1)])).is_err());
        assert_eq!(t.writes, writes, "externally enabled processor must prevent writes");
        t.words.get_mut(&Block::VoicePro).unwrap()[0] = 0;
        s.restore(&mut t).unwrap();
        assert_eq!(t.words, original);
    }
    #[test] fn apply_snapshot_observes_external_changes_to_unaffected_blocks_during_write() {
        let (mut s, mut t) = setup();
        let mut playback = t.words[&Block::PlaybackEq].clone();
        playback[1] = -5 * 256;
        t.external_on_write = Some((Block::PlaybackEq, playback.clone()));
        let snapshot = s.apply(&mut t, Effect::Noise, true, vals(&[("threshold", -45)])).unwrap();
        assert_eq!(snapshot.effects[&Block::PlaybackEq], playback);
        assert!(!s.journal.originals.contains_key(&Block::PlaybackEq));
        s.restore(&mut t).unwrap();
        assert_eq!(t.words[&Block::PlaybackEq], playback, "unaffected external state must survive restore");
    }
    #[test] fn apply_compare_restart_and_restore() {
        let (mut s, mut t) = setup(); let before = t.words.clone();
        s.apply(&mut t, Effect::Pitch, true, vals(&[("pitch", -3)])).unwrap();
        s.apply(&mut t, Effect::Eq, true, vals(&[("bass", 3), ("mid", -1), ("treble", 2)])).unwrap();
        let changed = t.words.clone();
        s.compare(&mut t).unwrap(); assert_eq!(t.words, before);
        assert!(s.apply(&mut t, Effect::Pitch, true, vals(&[("pitch", 1)])).is_err());
        s.compare(&mut t).unwrap(); assert_eq!(t.words, changed);
        Session::open(&t.journal).unwrap().restore(&mut t).unwrap(); assert_eq!(t.words, before);
    }
    #[test] fn failed_enable_rolls_back_and_keeps_recovery() {
        let (mut s, mut t) = setup(); let before = t.words.clone();
        t.fail = Some((Block::Pitch, 0, 1));
        assert!(s.apply(&mut t, Effect::Pitch, true, vals(&[("pitch", 3)])).is_err());
        assert_eq!(t.words, before); assert!(s.pending());
        s.restore(&mut t).unwrap();
    }
    #[test] fn compressor_runtime_refresh_and_restore() {
        let (mut s, mut t) = setup(); let before = t.words.clone();
        s.apply(&mut t, Effect::Compressor, true, vals(&[("threshold", -18), ("ratio", 3), ("attack", 10), ("release", 200)])).unwrap();
        s.compare(&mut t).unwrap(); assert_eq!(t.words, before);
        s.compare(&mut t).unwrap(); assert_eq!(t.words[&Block::Compressor][10], -1800);
        s.restore(&mut t).unwrap(); assert_eq!(t.words, before);
    }
    #[test] fn disconnect_keeps_journal_for_recovery() {
        let (mut s, mut t) = setup(); let before = t.words.clone();
        s.apply(&mut t, Effect::Noise, true, vals(&[("threshold", -45)])).unwrap();
        t.lose = true; assert!(s.restore(&mut t).is_err());
        assert!(Session::open(&t.journal).unwrap().pending());
        t.lose = false; Session::open(&t.journal).unwrap().restore(&mut t).unwrap(); assert_eq!(t.words, before);
    }
    #[test] fn refuse_other_serial_and_invalid_values() {
        let (mut s, mut t) = setup();
        assert!(s.apply(&mut t, Effect::Pitch, true, vals(&[("pitch", 13)])).is_err()); assert!(!t.journal.exists());
        s.apply(&mut t, Effect::Pitch, true, vals(&[("pitch", 2)])).unwrap();
        let writes = t.writes; let active = t.words.clone();
        t.serial = "other".into(); assert!(s.restore(&mut t).is_err());
        assert_eq!(t.writes, writes); assert_eq!(t.words, active);
        t.serial = "TEST-AM8-001".into(); s.restore(&mut t).unwrap();
    }
    #[test] fn pending_journals_require_serial_in_every_supported_version() {
        let (_s, t) = setup();
        for version in 1..=3 {
            for serial in [None, Some(serde_json::Value::Null), Some(serde_json::json!("")), Some(serde_json::json!("   "))] {
                let mut data = serde_json::json!({"version": version, "originals": {"pitch": [0, 0]}});
                if let Some(value) = serial { data["serial"] = value; }
                let bytes = serde_json::to_vec(&data).unwrap();
                fs::write(&t.journal, &bytes).unwrap();
                assert!(Session::open(&t.journal).is_err());
                assert_eq!(fs::read(&t.journal).unwrap(), bytes, "rejected recovery must be preserved");
            }
        }
        assert_eq!(t.writes, 0);
    }
    #[test] fn malformed_serials_are_preserved_and_device_serials_refused_before_writes() {
        let (mut s, mut t) = setup();
        for serial in [" TEST-AM8-001".to_string(), "AM8\0other".to_string(), "AM8\nother".to_string(), "\u{fffd}".to_string(), "a".repeat(MAX_SERIAL_UNITS + 1)] {
            let data = serde_json::json!({"version": 3, "originals": {"pitch": [0, 0]}, "serial": serial});
            let bytes = serde_json::to_vec(&data).unwrap();
            fs::write(&t.journal, &bytes).unwrap();
            assert!(Session::open(&t.journal).is_err());
            assert_eq!(fs::read(&t.journal).unwrap(), bytes);
            t.serial = serial;
            assert!(s.apply(&mut t, Effect::Pitch, true, vals(&[("pitch", 1)])).is_err());
        }
        assert_eq!(t.writes, 0);
        assert!(validate_serial("TEST-AM8-001").is_ok());
        assert!(validate_serial("AM8-Serial_01").is_ok());
        assert!(validate_serial(&"a".repeat(MAX_SERIAL_UNITS)).is_ok());
    }
    #[test] fn journal_size_is_bounded_before_parse_and_oversized_file_is_preserved() {
        let (_s, t) = setup();
        let oversized = (MAX_JOURNAL_BYTES as u64) * 256;
        let file = File::create(&t.journal).unwrap();
        file.set_len(oversized).unwrap();
        drop(file);
        let error = match Session::open(&t.journal) {
            Ok(_) => panic!("oversized journal was accepted"),
            Err(error) => error,
        };
        assert!(error.contains("excessivo"));
        assert_eq!(fs::metadata(&t.journal).unwrap().len(), oversized);
        assert_eq!(t.writes, 0);
        let mut bytes = serde_json::to_vec(&Journal { version: 1, originals: Blocks::new(), serial: None, original_headphone_mode: None }).unwrap();
        bytes.resize(MAX_JOURNAL_BYTES, b' ');
        fs::write(&t.journal, &bytes).unwrap();
        assert!(!Session::open(&t.journal).unwrap().pending());
    }
    #[test] fn old_autotune_session_is_restore_only() {
        let (s, mut t) = setup(); let baseline = t.words.clone();
        let journal = Journal { version: 2, originals: Blocks::from([
            (Block::Autotune, t.words[&Block::Autotune].clone()), (Block::MainRoute, t.words[&Block::MainRoute].clone()),
            (Block::AutoRoute, t.words[&Block::AutoRoute].clone())]), serial: Some(t.serial.clone()), original_headphone_mode: None };
        fs::write(&t.journal, serde_json::to_vec(&journal).unwrap()).unwrap(); drop(s);
        t.words.get_mut(&Block::Autotune).unwrap()[0] = 1;
        t.words.get_mut(&Block::MainRoute).unwrap()[1] = 1;
        t.words.get_mut(&Block::AutoRoute).unwrap()[1] = 0;
        let mut s = Session::open(&t.journal).unwrap();
        assert!(s.apply(&mut t, Effect::Pitch, true, vals(&[("pitch", 1)])).is_err());
        s.restore(&mut t).unwrap(); assert_eq!(t.words, baseline);
    }
    #[test] fn gain_test_restores_only_gain_keeps_voice_effects() {
        let (mut s, mut t) = setup();
        s.apply(&mut t, Effect::Pitch, true, vals(&[("pitch", 2)])).unwrap();
        let before = t.words.clone(); s.test_gain(&mut t, 1).unwrap(); assert_eq!(t.words, before);
        assert!(s.pending()); s.restore(&mut t).unwrap();
    }
    #[test] fn creative_paths_compare_and_restart_restore() {
        let (mut s, mut t)=setup(); let original=t.words.clone();
        s.apply(&mut t,Effect::Pitchpro,true,vals(&[("pitch",-3)])).unwrap();
        assert_eq!(t.words[&Block::Pitch][0],0);
        s.apply(&mut t,Effect::Voicepro,true,vals(&[("pitch",100),("formant",110)])).unwrap();
        assert_eq!(t.words[&Block::PitchPro][0],0);
        assert_eq!(t.words[&Block::MainRoute][1],1);
        assert_eq!(t.words[&Block::VoiceRoute],vec![1,0,600]);
        assert!(s.apply(&mut t,Effect::Reverb,true,vals(&[("mix",60),("room",60),("damping",50),("wet_gain",-6)])).is_err());
        s.apply(&mut t,Effect::Voicepro,false,vals(&[("pitch",100),("formant",110)])).unwrap();
        s.apply(&mut t,Effect::Reverb,true,vals(&[("mix",60),("room",60),("damping",50),("wet_gain",-6)])).unwrap();
        s.apply(&mut t,Effect::Plate,true,vals(&[("decay",40),("predelay",2500),("damping",50),("wet_gain",-6)])).unwrap();
        assert_eq!(t.words[&Block::ReverbNative][0],0);
        assert_eq!(t.words[&Block::EchoEq][1],-24576);
        let active=t.words.clone();
        s.compare(&mut t).unwrap();assert_eq!(t.words,original);
        s.compare(&mut t).unwrap();assert_eq!(t.words,active);
        Session::open(&t.journal).unwrap().restore(&mut t).unwrap();assert_eq!(t.words,original);
    }
    #[test] fn route_failure_restores_all_affected_blocks() {
        let (mut s,mut t)=setup();let original=t.words.clone();
        t.fail=Some((Block::WetRoute,1,0));
        assert!(s.apply(&mut t,Effect::Reverb,true,vals(&[("mix",60),("room",60),("damping",50),("wet_gain",-6)])).is_err());
        assert_eq!(t.words,original);assert!(s.pending());
        Session::open(&t.journal).unwrap().restore(&mut t).unwrap();assert_eq!(t.words,original);
    }
    #[test] fn playback_eq_does_not_write_mic_eq_or_routes_and_recovers_after_restart() {
        let (mut s, mut t) = setup(); let original = t.words.clone();
        let snapshot = s.apply(&mut t, Effect::Playbackeq, true, vals(&[("bass", -3), ("mid", 2), ("treble", -1)])).unwrap();
        assert_eq!(snapshot.effects[&Block::PlaybackEq][0], 1);
        assert_eq!(snapshot.effects[&Block::PlaybackEq][1], -2 * 256);
        assert_eq!(snapshot.effects[&Block::Eq], original[&Block::Eq]);
        assert_eq!(s.journal.originals.len(), 1);
        assert_eq!(s.journal.originals[&Block::PlaybackEq], original[&Block::PlaybackEq]);
        let active = t.words.clone();
        for (block, words) in &original {
            if *block != Block::PlaybackEq { assert_eq!(t.words[block], *words); }
        }
        s.compare(&mut t).unwrap(); assert_eq!(t.words, original);
        s.compare(&mut t).unwrap(); assert_eq!(t.words, active);
        drop(s);
        let mut restarted = Session::open(&t.journal).unwrap();
        assert!(restarted.pending());
        restarted.restore(&mut t).unwrap(); assert_eq!(t.words, original);
        assert!(!restarted.pending());
        assert!(!t.written_blocks.is_empty());
        assert!(t.written_blocks.iter().all(|block| *block == Block::PlaybackEq));
    }
    #[test] fn playback_eq_failure_rolls_back_and_refuses_another_device() {
        let (mut s, mut t) = setup(); let original = t.words.clone();
        t.fail = Some((Block::PlaybackEq, 0, 1));
        assert!(s.apply(&mut t, Effect::Playbackeq, true, vals(&[("bass", 3), ("mid", -2), ("treble", 1)])).is_err());
        assert_eq!(t.words, original); assert!(s.pending());
        let writes = t.writes;
        t.serial = "TEST-AM8-OTHER".into();
        assert!(Session::open(&t.journal).unwrap().restore(&mut t).is_err());
        assert_eq!(t.writes, writes);
        t.serial = "TEST-AM8-001".into();
        Session::open(&t.journal).unwrap().restore(&mut t).unwrap();
        assert_eq!(t.words, original);
        assert!(t.written_blocks.iter().all(|block| *block == Block::PlaybackEq));
    }
    #[test] fn microphone_and_playback_eq_keep_independent_journal_originals() {
        let (mut s, mut t) = setup(); let original = t.words.clone();
        s.apply(&mut t, Effect::Eq, true, vals(&[("bass", -1), ("mid", 0), ("treble", 1)])).unwrap();
        let microphone = t.words[&Block::Eq].clone();
        s.apply(&mut t, Effect::Playbackeq, true, vals(&[("bass", 2), ("mid", -1), ("treble", 0)])).unwrap();
        assert_eq!(t.words[&Block::Eq], microphone);
        assert_eq!(s.journal.originals.len(), 2);
        assert_eq!(s.journal.originals[&Block::Eq], original[&Block::Eq]);
        assert_eq!(s.journal.originals[&Block::PlaybackEq], original[&Block::PlaybackEq]);
        let active = t.words.clone();
        s.compare(&mut t).unwrap(); assert_eq!(t.words, original);
        s.compare(&mut t).unwrap(); assert_eq!(t.words, active);
        Session::open(&t.journal).unwrap().restore(&mut t).unwrap();
        assert_eq!(t.words, original);
        assert!(t.written_blocks.iter().all(|block| matches!(block, Block::Eq | Block::PlaybackEq)));
    }
    #[test] fn playback_eq_invalid_parameters_are_rejected_before_journaling_or_writes() {
        let (mut s, mut t) = setup(); let original = t.words.clone();
        assert!(s.apply(&mut t, Effect::Playbackeq, true, vals(&[("bass", 7), ("mid", 0), ("treble", 0)])).is_err());
        assert_eq!(t.writes, 0); assert!(!t.journal.exists()); assert_eq!(t.words, original);
    }
    #[test] fn headphone_gain_reapply_compare_restart_and_restore_keep_original_pregain() {
        let (mut s, mut t) = setup();
        // A device can start with a nonzero EQ pre-gain; restore that exact value.
        t.words.get_mut(&Block::PlaybackEq).unwrap()[1] = -4 * 256;
        let original = t.words.clone();
        s.apply(&mut t, Effect::Eq, true, vals(&[("bass", -1), ("mid", 0), ("treble", 1)])).unwrap();
        let microphone = t.words[&Block::Eq].clone();
        s.apply(&mut t, Effect::Playbackeq, true, vals(&[("bass", 0), ("mid", 0), ("treble", 0), ("output_gain", 3)])).unwrap();
        assert_eq!(t.words[&Block::PlaybackEq][1], 3 * 256);
        s.apply(&mut t, Effect::Playbackeq, true, vals(&[("bass", 6), ("mid", -2), ("treble", 3), ("output_gain", 18)])).unwrap();
        assert_eq!(t.words[&Block::PlaybackEq][1], 9 * 256);
        assert_eq!(t.words[&Block::Eq], microphone);
        assert_eq!(s.journal.originals[&Block::PlaybackEq], original[&Block::PlaybackEq]);
        let active = t.words.clone();
        for (block, words) in &original {
            if !matches!(block, Block::Eq | Block::PlaybackEq) { assert_eq!(t.words[block], *words); }
        }
        s.compare(&mut t).unwrap(); assert_eq!(t.words, original);
        s.compare(&mut t).unwrap(); assert_eq!(t.words, active);
        drop(s);
        let mut resumed = Session::open(&t.journal).unwrap();
        assert_eq!(resumed.journal.originals[&Block::PlaybackEq][1], -4 * 256);
        resumed.restore(&mut t).unwrap();
        assert_eq!(t.words, original); assert!(!resumed.pending());
        assert!(t.written_blocks.iter().all(|block| matches!(block, Block::Eq | Block::PlaybackEq)));
    }
    #[test] fn headphone_gain_failed_reapply_rolls_back_to_the_previous_active_setting() {
        let (mut s, mut t) = setup(); let original = t.words.clone();
        s.apply(&mut t, Effect::Playbackeq, true, vals(&[("bass", 0), ("mid", 0), ("treble", 0), ("output_gain", 3)])).unwrap();
        let previous = t.words.clone();
        t.fail = Some((Block::PlaybackEq, 1, 8 * 256));
        assert!(s.apply(&mut t, Effect::Playbackeq, true, vals(&[("bass", 0), ("mid", 0), ("treble", 0), ("output_gain", 8)])).is_err());
        assert_eq!(t.words, previous);
        assert_eq!(s.journal.originals[&Block::PlaybackEq], original[&Block::PlaybackEq]);
        assert_eq!(Session::open(&t.journal).unwrap().journal.originals[&Block::PlaybackEq], original[&Block::PlaybackEq]);
        // The failed write can be retried without losing the first session's baseline.
        s.apply(&mut t, Effect::Playbackeq, true, vals(&[("bass", 0), ("mid", 0), ("treble", 0), ("output_gain", 8)])).unwrap();
        assert_eq!(t.words[&Block::PlaybackEq][1], 8 * 256);
        Session::open(&t.journal).unwrap().restore(&mut t).unwrap();
        assert_eq!(t.words, original);
        assert!(t.written_blocks.iter().all(|block| *block == Block::PlaybackEq));
    }
    #[test] fn headphone_gain_invalid_changes_never_write_or_create_recovery() {
        let (mut s, mut t) = setup(); let original = t.words.clone();
        for gain in [-1, 19, i32::MIN, i32::MAX] {
            for enabled in [false, true] {
                assert!(s.apply(&mut t, Effect::Playbackeq, enabled, vals(&[("bass", 0), ("mid", 0), ("treble", 0), ("output_gain", gain)])).is_err());
            }
        }
        assert!(s.apply(&mut t, Effect::Eq, true, vals(&[("bass", 0), ("mid", 0), ("treble", 0), ("output_gain", 0)])).is_err());
        assert!(s.apply(&mut t, Effect::Playbackeq, true, vals(&[("bass", 0), ("mid", 0), ("treble", 0), ("output_gain", 18), ("hidden", 0)])).is_err());
        let mut full = vals(&[("output_gain", 18)]);
        for i in 0..10 {
            for (key, value) in [("enabled", 0), ("type", 0), ("frequency", 1000), ("q", 724), ("gain", 0)] {
                full.insert(format!("f{i}_{key}"), value);
            }
        }
        full.insert("output_gain".into(), 19);
        assert!(s.apply(&mut t, Effect::Playbackeq, true, full.clone()).is_err());
        full.insert("output_gain".into(), 18);
        full.remove("f9_q"); full.insert("hidden".into(), 0);
        assert!(s.apply(&mut t, Effect::Playbackeq, true, full).is_err());
        assert_eq!(t.writes, 0); assert!(!t.journal.exists()); assert_eq!(t.words, original); assert!(!s.pending());
    }
}
