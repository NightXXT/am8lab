use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::OnceLock, thread, time::Duration};

pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Effect { Noise, Pitch, Pitchpro, Voicepro, Reverb, Plate, Feedback, Eq, Playbackeq, Compressor }

impl Effect {
    pub fn block(self) -> Block {
        match self { Self::Noise => Block::Noise, Self::Pitch => Block::Pitch, Self::Pitchpro => Block::PitchPro,
            Self::Voicepro => Block::VoicePro, Self::Reverb => Block::ReverbNative, Self::Plate => Block::PlateNative,
            Self::Feedback => Block::FeedbackFine, Self::Eq => Block::Eq, Self::Playbackeq => Block::PlaybackEq,
            Self::Compressor => Block::Compressor }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Block { Noise, Pitch, Eq, Compressor, MicGain, Autotune, Voice, MainRoute, VoiceRoute, AutoRoute,
    PitchPro, VoicePro, EchoEq, ReverbNative, PlateNative, WetRoute, FeedbackFine, PlaybackEq }

impl Block {
    pub fn address(self) -> u8 {
        match self {
            Self::Noise => 0x90, Self::Pitch => 0x96, Self::Eq => 0x99, Self::Compressor => 0xAE,
            Self::MicGain => 0x8F, Self::Autotune => 0xA3, Self::Voice => 0xA0,
            Self::MainRoute => 0x95, Self::VoiceRoute => 0x9F, Self::AutoRoute => 0xA2,
            Self::PitchPro => 0x97, Self::VoicePro => 0xA1, Self::EchoEq => 0xA4,
            Self::ReverbNative => 0xA8, Self::PlateNative => 0xA9, Self::WetRoute => 0xAC, Self::FeedbackFine => 0x94,
            Self::PlaybackEq => 0x8D,
        }
    }
    pub fn kind(self) -> &'static str {
        match self {
            Self::Noise => "noise_suppressor_expander", Self::Pitch => "pitch_shifter", Self::Eq => "eq", Self::Compressor => "drc",
            Self::Autotune => "auto_tune", Self::Voice => "voice_changer",
            Self::PitchPro => "pitch_shifter_pro", Self::VoicePro => "voice_changer_pro",
            Self::EchoEq | Self::PlaybackEq => "eq", Self::ReverbNative => "reverb", Self::PlateNative => "reverb_plate",
            Self::FeedbackFine => "howling_suppressor_fine", _ => "gain_control",
        }
    }
    pub fn is_gain(self) -> bool { self.kind() == "gain_control" }
    pub fn legacy(self) -> bool { matches!(self, Self::Autotune | Self::Voice | Self::MainRoute | Self::VoiceRoute | Self::AutoRoute) }
}

#[derive(Deserialize)]
struct Param { min: i16, max: i16 }
#[derive(Deserialize)]
struct Descriptor { params: Vec<Param> }
fn schema() -> &'static BTreeMap<String, Descriptor> {
    static SCHEMA: OnceLock<BTreeMap<String, Descriptor>> = OnceLock::new();
    SCHEMA.get_or_init(|| serde_json::from_str(include_str!("../data/effect-schema.json")).expect("bundled verified schema"))
}

pub fn validate_block(block: Block, words: &[i16]) -> Result<()> {
    let params = &schema()[block.kind()].params;
    if words.len() != params.len() + 1 || !matches!(words[0], 0 | 1) { return Err("Bloco de parâmetros inválido".into()); }
    for (v, p) in words[1..].iter().zip(params) {
        if *v < p.min || *v > p.max { return Err(format!("Parâmetro fora dos limites: {block:?}")); }
    }
    Ok(())
}

pub fn validate_word(block: Block, index: usize, value: i16) -> Result<()> {
    if index == 0 {
        return if matches!(value, 0 | 1) { Ok(()) } else { Err("Estado inválido".into()) };
    }
    let param = schema()[block.kind()].params.get(index - 1).ok_or("Seletor fora do bloco")?;
    if value < param.min || value > param.max { return Err("Parâmetro fora dos limites".into()); }
    Ok(())
}

pub fn frame(op: u8, payload: &[u8]) -> Result<[u8; 257]> {
    if payload.len() > 250 { return Err("Payload excessivo".into()); }
    let mut out = [0; 257];
    out[1..5].copy_from_slice(&[0xA5, 0x5A, op, payload.len() as u8]);
    out[5..5 + payload.len()].copy_from_slice(payload);
    out[5 + payload.len()] = 0x16;
    Ok(out)
}

pub fn decode(raw: &[u8], op: u8) -> Result<&[u8]> {
    if raw.len() < 6 || raw[..3] != [0, 0xA5, 0x5A] || raw[3] != op { return Err("Resposta HID inesperada".into()); }
    let n = raw[4] as usize;
    if n + 5 >= raw.len() || raw[n + 5] != 0x16 { return Err("Resposta HID incompleta".into()); }
    Ok(&raw[5..5 + n])
}

pub trait Transport {
    fn query(&mut self, op: u8, payload: &[u8]) -> Result<Vec<u8>>;
    fn write_word(&mut self, block: Block, index: usize, value: i16) -> Result<()>;
    fn write_dac_mode(&mut self, _mode: u16) -> Result<()> {
        Err("Este transporte não permite ajustar o modo dos fones".into())
    }
    fn guard(&mut self) -> Result<()>;
    fn serial(&self) -> &str;
    /// Label established by the most recent successful full guard.
    fn firmware(&self) -> &'static str;
    fn wait(&self, ms: u64) { thread::sleep(Duration::from_millis(ms)); }
}

pub fn validate_dac_mode(mode: u16) -> Result<()> {
    if matches!(mode, 0 | 2) { Ok(()) } else { Err("Modo dos fones não validado; permitidos estéreo e mono".into()) }
}

/// The B5 0.7.1 DAC response is a tag followed by exactly fourteen u16 words.
/// Only word seven is writable through this SDK; the other fields are observed.
pub fn read_dac_words(t: &mut impl Transport) -> Result<[u16; 14]> {
    let payload = t.query(0x09, &[])?;
    if payload.len() != 29 || payload[0] != 255 { return Err("Layout do DAC diferente do validado".into()); }
    let words = std::array::from_fn(|i| u16::from_le_bytes([payload[1 + i * 2], payload[2 + i * 2]]));
    if words[0] != 3 || words[1] > 8 || words[2] > 3 || words[7] > 3 { return Err("Estado do DAC fora da referência validada".into()); }
    Ok(words)
}

pub fn read_dac_mode(t: &mut impl Transport) -> Result<u16> {
    let mode = read_dac_words(t)?[7];
    validate_dac_mode(mode)?;
    Ok(mode)
}

pub fn set_dac_mode(t: &mut impl Transport, mode: u16) -> Result<()> {
    validate_dac_mode(mode)?;
    let before = read_dac_words(t)?;
    validate_dac_mode(before[7])?;
    if before[7] == mode { return Ok(()); }
    t.write_dac_mode(mode)?;
    for _ in 0..6 {
        t.wait(80);
        let after = read_dac_words(t)?;
        if before.iter().zip(&after).enumerate().any(|(index, (a, b))| index != 7 && a != b) {
            return Err("Outra configuração do DAC mudou durante o ajuste dos fones".into());
        }
        validate_dac_mode(after[7])?;
        if after[7] == mode { return Ok(()); }
    }
    Err("O DAC não confirmou o modo solicitado para os fones".into())
}

pub fn read_meter(t: &mut impl Transport, opcode: u8) -> Result<u16> {
    if !matches!(opcode,0x91 | 0x82) { return Err("Medidor não permitido".into()); }
    let p=t.query(opcode,&[])?;
    if p.len()!=5 || p[..3]!=[255,1,0] { return Err("Resposta do medidor inválida".into()); }
    let level=u16::from_le_bytes([p[3],p[4]]);
    if level>32767 { return Err("Nível fora da escala interna".into()); }
    Ok(level)
}

pub fn read_block(t: &mut impl Transport, block: Block) -> Result<Vec<i16>> {
    let p = t.query(block.address(), &[])?;
    if p.first() != Some(&255) || (p.len() - 1) % 2 != 0 { return Err("Resposta de efeito inválida".into()); }
    let words: Vec<_> = p[1..].chunks_exact(2).map(|x| i16::from_le_bytes([x[0], x[1]])).collect();
    validate_block(block, &words)?;
    Ok(words)
}

pub fn set_block(t: &mut impl Transport, block: Block, target: &[i16]) -> Result<()> {
    validate_block(block, target)?;
    for _ in 0..3 {
        let mut before = read_block(t, block)?;
        if before == target { return Ok(()); }
        if before[0] == 1 && before[1..] != target[1..] && !block.is_gain() {
            t.write_word(block, 0, 0)?;
            t.wait(250);
            before = read_block(t, block)?;
        }
        for i in 1..target.len() {
            if before[i] != target[i] { t.write_word(block, i, target[i])?; }
        }
        if block == Block::Compressor && target[0] == 0 && before[1..] != target[1..] {
            t.write_word(block, 0, 1)?;
            t.wait(250);
            if read_block(t, block)?[1..] != target[1..] { return Err("Compressor não carregou os valores restaurados".into()); }
            t.write_word(block, 0, 0)?;
        } else if target[0] != before[0] { t.write_word(block, 0, target[0])?; }
        t.wait(if matches!(block,Block::VoicePro | Block::PlateNative) { 1000 } else { 250 });
        if read_block(t, block)? == target { return Ok(()); }
    }
    Err(format!("Confirmação do ajuste falhou: {block:?}; esperado {target:?}; lido {:?}",read_block(t,block)?))
}

pub fn target(effect: Effect, enabled: bool, values: &BTreeMap<String, i32>, before: &[i16]) -> Result<Vec<i16>> {
    validate_block(effect.block(), before)?;
    let output_gain = if let Some(gain) = values.get("output_gain") {
        if effect != Effect::Playbackeq || !(0..=18).contains(gain) {
            return Err("Ganho dos fones permitido: 0 a 18 dB, apenas no EQ de reprodução".into());
        }
        *gain
    } else { 0 };
    let gain_field_count = usize::from(values.contains_key("output_gain"));
    let mut out = before.to_vec();
    out[0] = i16::from(enabled);
    if matches!(effect, Effect::Eq | Effect::Playbackeq) && values.contains_key("f0_enabled") {
        if values.len() != 50 + gain_field_count { return Err("Esperados dez filtros completos".into()); }
        out[2] = 0;
        let mut boost = 0;
        for i in 0..10 {
            let mut fields = Vec::new();
            for (key, lo, hi) in [("enabled",0,1),("type",0,4),("frequency",20,16000),("q",256,8192),("gain",-6,6)] {
                let v = *values.get(&format!("f{i}_{key}")).ok_or("Filtro incompleto")?;
                if !(lo..=hi).contains(&v) { return Err("Filtro fora dos limites".into()); }
                fields.push(v);
            }
            let gain = if fields[1] >= 3 { 0 } else { fields[4] };
            if fields[0] == 1 { boost += gain.max(0); }
            let start = 3 + i * 5;
            out[start..start + 5].copy_from_slice(&[fields[0] as i16, fields[1] as i16, fields[2] as i16, fields[3] as i16, (gain * 256) as i16]);
        }
        // The device's EQ pre-gain is Q8.8; keep headroom compensation for band boosts.
        out[1] = ((output_gain - boost) * 256) as i16;
        validate_block(effect.block(), &out)?;
        return Ok(out);
    }
    let expected: &[(&str, i32, i32)] = match effect {
        Effect::Noise if values.len() == 1 => &[("threshold", -70, -20)],
        Effect::Noise => &[("threshold", -70, -20),("ratio",1,10),("attack",1,100),("release",50,1000)],
        Effect::Pitch => &[("pitch", -12, 12)], Effect::Pitchpro => &[("pitch",-3,3)],
        Effect::Voicepro => &[("pitch",80,120),("formant",90,110)],
        Effect::Reverb => &[("mix",0,60),("room",0,60),("damping",0,100),("wet_gain",-18,-6)],
        Effect::Plate => &[("decay",20,60),("predelay",0,2500),("damping",0,100),("wet_gain",-18,-6)],
        Effect::Feedback => &[],
        Effect::Eq | Effect::Playbackeq => &[("bass", -6, 6), ("mid", -6, 6), ("treble", -6, 6)],
        Effect::Compressor => &[("threshold", -40, -6), ("ratio", 1, 6), ("attack", 1, 100), ("release", 50, 1000)],
    };
    if values.len() != expected.len() + gain_field_count || expected.iter().any(|(k, lo, hi)| values.get(*k).is_none_or(|v| v < lo || v > hi)) {
        return Err("Controles inválidos ou fora dos limites".into());
    }
    match effect {
        Effect::Noise => { out[1] = (values["threshold"] * 100) as i16;
            if values.len() > 1 { out[2] = values["ratio"] as i16; out[3] = values["attack"] as i16; out[4] = values["release"] as i16; }
        }
        Effect::Pitch | Effect::Pitchpro => out[1] = (values["pitch"] * 10) as i16,
        Effect::Voicepro => { out[1] = values["pitch"] as i16; out[2] = values["formant"] as i16; }
        Effect::Reverb => { out[1..].copy_from_slice(&[0,values["mix"] as i16,100,values["room"] as i16,values["damping"] as i16,1]); }
        Effect::Plate => { out[1..].copy_from_slice(&[8000,1,values["predelay"] as i16,60,values["decay"] as i16,(values["damping"] * 100) as i16,100]); }
        Effect::Feedback => { if out[1] > out[2] { return Err("Q mínimo maior que o máximo".into()); } }
        Effect::Eq | Effect::Playbackeq => {
            let boost = ["bass", "mid", "treble"].iter().map(|key| values[*key].max(0)).sum::<i32>();
            out[1] = ((output_gain - boost) * 256) as i16;
            out[2] = 0;
            for (start, key, kind, hz) in [(3, "bass", 1, 150), (8, "mid", 0, 1200), (13, "treble", 2, 5000)] {
                out[start..start + 5].copy_from_slice(&[1, kind, hz, 724, (values[key] * 256) as i16]);
            }
            for start in (18..53).step_by(5) { out[start] = 0; }
        }
        Effect::Compressor => {
            out[1] = 0;
            out[10] = (values["threshold"] * 100) as i16;
            out[14] = (values["ratio"] * 100) as i16;
            out[18] = values["attack"] as i16;
            out[22] = values["release"] as i16;
            out[26] = 0;
        }
    }
    validate_block(effect.block(), &out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    struct DacFake { response: Vec<u8>, written: Vec<u16> }
    impl Transport for DacFake {
        fn firmware(&self) -> &'static str { "B5 0.7.1" }
        fn query(&mut self, op: u8, payload: &[u8]) -> Result<Vec<u8>> { assert_eq!(op, 0x09); assert!(payload.is_empty()); Ok(self.response.clone()) }
        fn write_word(&mut self, _: Block, _: usize, _: i16) -> Result<()> { panic!("effect write forbidden in DAC test") }
        fn write_dac_mode(&mut self, mode: u16) -> Result<()> { self.written.push(mode); self.response[15..17].copy_from_slice(&mode.to_le_bytes()); Ok(()) }
        fn guard(&mut self) -> Result<()> { Ok(()) }
        fn serial(&self) -> &str { "TEST" }
        fn wait(&self, _: u64) {}
    }
    fn dac_fake() -> DacFake {
        let words: [u16; 14] = [3,7,0,4095,4095,0,3,0,0,0,0,5,0,0];
        DacFake { response: [vec![255], words.iter().flat_map(|v| v.to_le_bytes()).collect()].concat(), written: vec![] }
    }
    #[test] fn dac_parser_requires_the_validated_full_layout() {
        let mut t = dac_fake();
        assert_eq!(read_dac_words(&mut t).unwrap()[7], 0);
        for response in [vec![], vec![255], vec![255; 27], vec![255; 31], vec![0; 29]] {
            t.response = response; assert!(read_dac_words(&mut t).is_err());
        }
        for (index, value) in [(0usize, 2u16), (1, 9), (2, 4), (7, 4)] {
            let mut t = dac_fake(); t.response[1 + index * 2..3 + index * 2].copy_from_slice(&value.to_le_bytes());
            assert!(read_dac_words(&mut t).is_err());
        }
    }
    #[test] fn dac_setter_accepts_only_stereo_and_mono_and_confirms_all_other_fields() {
        let mut t = dac_fake(); let original = read_dac_words(&mut t).unwrap();
        for mode in [1, 3, u16::MAX] { assert!(set_dac_mode(&mut t, mode).is_err()); }
        assert!(t.written.is_empty());
        set_dac_mode(&mut t, 2).unwrap(); assert_eq!(read_dac_mode(&mut t).unwrap(), 2);
        let changed = read_dac_words(&mut t).unwrap();
        for index in 0..14 { if index != 7 { assert_eq!(original[index], changed[index]); } }
        set_dac_mode(&mut t, 0).unwrap(); assert_eq!(read_dac_words(&mut t).unwrap(), original);
        assert_eq!(t.written, vec![2, 0]);
        t.response[15..17].copy_from_slice(&1u16.to_le_bytes());
        assert!(set_dac_mode(&mut t, 0).is_err()); assert_eq!(t.written, vec![2, 0]);
    }
    #[test] fn individual_writes_reject_invalid_selectors_and_values() {
        assert!(validate_word(Block::Noise,0,2).is_err());
        assert!(validate_word(Block::Pitch,2,0).is_err());
        assert!(validate_word(Block::Noise,usize::MAX,0).is_err());
        assert!(validate_word(Block::Noise,1,32767).is_err());
        assert!(validate_word(Block::Noise,1,-4500).is_ok());
        assert!(validate_word(Block::Pitch,0,1).is_ok());
    }
    use super::*;
    #[test] fn frame_roundtrip_and_signed_values() {
        let f = frame(0x96, &[1, 0xF6, 0xFF]).unwrap();
        assert_eq!(decode(&f, 0x96).unwrap(), &[1, 0xF6, 0xFF]);
        assert!(decode(&f, 0x90).is_err());
        assert!(decode(&f[..7], 0x96).is_err());
    }
    #[test] fn autotune_is_not_an_effect() {
        assert!(serde_json::from_str::<Effect>("\"autotune\"").is_err());
        assert!(serde_json::from_str::<Effect>("\"voice\"").is_err());
    }
    #[test] fn controls_reject_extra_fields_and_bad_ranges() {
        let values = BTreeMap::from([("pitch".into(), 13)]);
        assert!(target(Effect::Pitch, true, &values, &[0, 0]).is_err());
        let values = BTreeMap::from([("pitch".into(), 1), ("hidden".into(), 0)]);
        assert!(target(Effect::Pitch, true, &values, &[0, 0]).is_err());
    }
    #[test] fn eq_compensates_combined_boost_and_preserves_disabled_filters() {
        let mut words = vec![0; 53];
        for start in (3..53).step_by(5) { words[start + 2] = 200; words[start + 3] = 724; }
        let vals = BTreeMap::from([("bass".into(), 4), ("mid".into(), -2), ("treble".into(), 3)]);
        let out = target(Effect::Eq, true, &vals, &words).unwrap();
        assert_eq!(out[1], -7 * 256); assert_eq!(out[7], 4 * 256); assert_eq!(out[12], -2 * 256);
        assert_eq!(out[18], 0);
    }
    #[test] fn complete_eq_checks_each_filter_before_mutation() {
        let mut words=vec![0;53];for start in (3..53).step_by(5) { words[start+2]=200;words[start+3]=724; }
        let mut values=BTreeMap::new();
        for i in 0..10 { for (key,v) in [("enabled",1),("type",0),("frequency",200),("q",724),("gain",6)] { values.insert(format!("f{i}_{key}"),v); } }
        let out=target(Effect::Eq,true,&values,&words).unwrap();assert_eq!(out[1],-60*256);
        values.insert("f9_frequency".into(),16001);assert!(target(Effect::Eq,true,&values,&words).is_err());
        values.insert("f9_frequency".into(),1000);values.insert("f9_type".into(),4);
        let out=target(Effect::Eq,true,&values,&words).unwrap();assert_eq!(out[52],0);
    }
    #[test] fn playback_eq_uses_its_own_address_and_shared_eq_validation() {
        assert_eq!(Effect::Playbackeq.block(), Block::PlaybackEq);
        assert_eq!(Block::PlaybackEq.address(), 0x8D);
        assert_eq!(serde_json::to_string(&Effect::Playbackeq).unwrap(), "\"playbackeq\"");
        assert_eq!(serde_json::to_string(&Block::PlaybackEq).unwrap(), "\"playback_eq\"");
        let mut words = vec![0; 53];
        for start in (3..53).step_by(5) { words[start + 2] = 200; words[start + 3] = 724; }
        let values = BTreeMap::from([("bass".into(), 6), ("mid".into(), -6), ("treble".into(), 2)]);
        assert_eq!(target(Effect::Playbackeq, true, &values, &words).unwrap(), target(Effect::Eq, true, &values, &words).unwrap());
        for (key, invalid) in [("bass", 7), ("mid", -7), ("treble", i32::MAX)] {
            let mut invalid_values = values.clone(); invalid_values.insert(key.into(), invalid);
            assert!(target(Effect::Playbackeq, true, &invalid_values, &words).is_err());
        }
        assert!(validate_word(Block::PlaybackEq, 53, 0).is_err());
        assert!(validate_word(Block::PlaybackEq, 0, 2).is_err());
    }
    #[test] fn playback_full_eq_checks_every_field_and_compensates_boost() {
        let mut words = vec![0; 53];
        for start in (3..53).step_by(5) { words[start + 2] = 200; words[start + 3] = 724; }
        let mut values = BTreeMap::new();
        for i in 0..10 {
            for (key, value) in [("enabled", 1), ("type", 0), ("frequency", 1000), ("q", 724), ("gain", 6)] {
                values.insert(format!("f{i}_{key}"), value);
            }
        }
        let out = target(Effect::Playbackeq, true, &values, &words).unwrap();
        assert_eq!(out[1], -60 * 256);
        for (key, invalid) in [("enabled", 2), ("type", 5), ("frequency", 19), ("frequency", 16001),
            ("q", 255), ("q", 8193), ("gain", -7), ("gain", 7)] {
            let mut invalid_values = values.clone(); invalid_values.insert(format!("f9_{key}"), invalid);
            assert!(target(Effect::Playbackeq, true, &invalid_values, &words).is_err());
        }
        values.insert("f9_type".into(), 4);
        let out = target(Effect::Playbackeq, true, &values, &words).unwrap();
        assert_eq!(out[52], 0);
        assert_eq!(out[1], -54 * 256);
        values.remove("f9_q");
        assert!(target(Effect::Playbackeq, true, &values, &words).is_err());
    }
    #[test] fn headphone_gain_uses_native_q8_8_and_keeps_quick_eq_compensation() {
        let mut words = vec![0; 53];
        for start in (3..53).step_by(5) { words[start + 2] = 200; words[start + 3] = 724; }
        let mut values = BTreeMap::from([("bass".into(), 0), ("mid".into(), 0), ("treble".into(), 0)]);
        let original_api = target(Effect::Playbackeq, true, &values, &words).unwrap();
        values.insert("output_gain".into(), 0);
        assert_eq!(target(Effect::Playbackeq, true, &values, &words).unwrap(), original_api);
        values.insert("output_gain".into(), 18);
        let flat = target(Effect::Playbackeq, true, &values, &words).unwrap();
        assert_eq!(flat[1], 4608);
        assert!(validate_word(Block::PlaybackEq, 1, flat[1]).is_ok());
        assert!(validate_word(Block::PlaybackEq, 1, 19 * 256).is_err());
        values.insert("bass".into(), 6); values.insert("mid".into(), -6); values.insert("treble".into(), 3);
        let shaped = target(Effect::Playbackeq, true, &values, &words).unwrap();
        assert_eq!(shaped[1], 9 * 256);
        assert_eq!(shaped[7], 6 * 256); assert_eq!(shaped[12], -6 * 256); assert_eq!(shaped[17], 3 * 256);
    }
    #[test] fn headphone_gain_compensates_only_active_full_eq_boosts() {
        let mut words = vec![0; 53];
        for start in (3..53).step_by(5) { words[start + 2] = 200; words[start + 3] = 724; }
        let mut values = BTreeMap::new();
        for i in 0..10 {
            for (key, value) in [("enabled", 1), ("type", 0), ("frequency", 1000), ("q", 724), ("gain", 6)] {
                values.insert(format!("f{i}_{key}"), value);
            }
        }
        let old_api = target(Effect::Playbackeq, true, &values, &words).unwrap();
        values.insert("output_gain".into(), 0);
        assert_eq!(target(Effect::Playbackeq, true, &values, &words).unwrap(), old_api);
        values.insert("output_gain".into(), 18);
        assert_eq!(target(Effect::Playbackeq, true, &values, &words).unwrap()[1], -42 * 256);
        values.insert("f8_enabled".into(), 0);
        values.insert("f9_type".into(), 4);
        let out = target(Effect::Playbackeq, true, &values, &words).unwrap();
        assert_eq!(out[1], -30 * 256);
        assert_eq!(out[43], 0); assert_eq!(out[52], 0);
        // Cuts never increase the manual gain through negative compensation.
        for i in 0..10 { values.insert(format!("f{i}_gain"), -6); }
        assert_eq!(target(Effect::Playbackeq, true, &values, &words).unwrap()[1], 18 * 256);
    }
    #[test] fn headphone_gain_rejects_out_of_range_foreign_and_incomplete_parameters() {
        let mut words = vec![0; 53];
        for start in (3..53).step_by(5) { words[start + 2] = 200; words[start + 3] = 724; }
        let mut values = BTreeMap::from([("bass".into(), 0), ("mid".into(), 0), ("treble".into(), 0), ("output_gain".into(), 0)]);
        for gain in [-1, 19, i32::MIN, i32::MAX] {
            values.insert("output_gain".into(), gain);
            for enabled in [false, true] { assert!(target(Effect::Playbackeq, enabled, &values, &words).is_err()); }
        }
        values.insert("output_gain".into(), 0);
        assert!(target(Effect::Eq, true, &values, &words).is_err());
        values.insert("hidden".into(), 1);
        assert!(target(Effect::Playbackeq, true, &values, &words).is_err());
        values.remove("treble");
        assert!(target(Effect::Playbackeq, true, &values, &words).is_err());
        let mut full = BTreeMap::from([("output_gain".into(), 18)]);
        for i in 0..10 {
            for (key, value) in [("enabled", 0), ("type", 0), ("frequency", 1000), ("q", 724), ("gain", 0)] {
                full.insert(format!("f{i}_{key}"), value);
            }
        }
        assert!(target(Effect::Eq, true, &full, &words).is_err());
        full.insert("hidden".into(), 0);
        assert!(target(Effect::Playbackeq, true, &full, &words).is_err());
        full.remove("f9_q");
        assert!(target(Effect::Playbackeq, true, &full, &words).is_err());
    }
    #[test] fn gain_ipc_values_require_finite_i32_integers() {
        // Commands deserialize the whole parameter map before Session::apply can touch USB.
        for invalid in ["0.5", "18.0", "NaN", "null", "true", "\"18\"", "2147483648", "-2147483649", "1e300"] {
            let json = format!("{{\"bass\":0,\"mid\":0,\"treble\":0,\"output_gain\":{invalid}}}");
            assert!(serde_json::from_str::<BTreeMap<String, i32>>(&json).is_err(), "accepted {invalid}");
        }
        let valid: BTreeMap<String, i32> = serde_json::from_str("{\"bass\":0,\"mid\":0,\"treble\":0,\"output_gain\":18}").unwrap();
        assert_eq!(valid["output_gain"], 18);
    }
}
