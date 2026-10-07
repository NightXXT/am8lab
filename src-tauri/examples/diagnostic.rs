use am8_lab::{protocol::{Block, Effect, Result, Transport, read_meter, read_block}, session::Session, windows::{HidDevice, InstanceMutex}};
use std::{collections::BTreeMap, path::PathBuf};

fn main() {
    if let Err(error) = run() { eprintln!("{error}"); std::process::exit(1); }
}
fn run() -> Result<()> {
    let _instance = InstanceMutex::acquire()?;
    let path = PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA ausente")?).join("AM8Lab/efeitos-recuperacao.json");
    let mut s = Session::open(path)?;
    let mut t = HidDevice::open()?;
    if std::env::args().any(|a| a == "--control-timing") {
        t.set_query_interval(80)?;
        let blocks = [Block::Noise, Block::Pitch, Block::Eq, Block::PlaybackEq, Block::Compressor,
            Block::PitchPro, Block::VoicePro, Block::ReverbNative, Block::PlateNative, Block::FeedbackFine,
            Block::WetRoute, Block::MainRoute, Block::VoiceRoute, Block::EchoEq, Block::Autotune,
            Block::Voice, Block::AutoRoute, Block::MicGain];
        t.guard()?;
        let mut reference = BTreeMap::new();
        for block in blocks { reference.insert(block, read_block(&mut t, block)?); }
        let mut runs = Vec::new();
        for interval_ms in [80, 16, 8] {
            t.set_query_interval(interval_ms)?;
            let guard_start = std::time::Instant::now();
            t.guard()?;
            let guard_ms = guard_start.elapsed().as_secs_f64() * 1000.0;
            let mut samples = Vec::new();
            for round in 0..3 {
                let order: Vec<_> = if round % 2 == 0 { blocks.to_vec() } else { blocks.iter().rev().copied().collect() };
                for block in order {
                    let start = std::time::Instant::now();
                    let words = read_block(&mut t, block)?;
                    if words != reference[&block] { return Err(format!("O bloco {block:?} mudou durante o diagnóstico de leitura")); }
                    samples.push(serde_json::json!({"block":block,"ms":start.elapsed().as_secs_f64()*1000.0}));
                }
            }
            runs.push(serde_json::json!({"interval_ms":interval_ms,"guard_ms":guard_ms,"samples":samples}));
        }
        let report = serde_json::json!({"test":"read_only_control_timing","blocks":blocks.len(),"runs":runs,"all_matched":true,"writes":0});
        let arguments: Vec<_> = std::env::args().collect();
        if let Some(index) = arguments.iter().position(|a| a == "--report") {
            let path = arguments.get(index + 1).ok_or("Caminho do relatório ausente")?;
            std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).map_err(|e| e.to_string())?;
        }
        println!("{}", report);
        return Ok(());
    }
    if std::env::args().any(|a| a == "--meter-timing") {
        t.guard()?;
        let mut runs = Vec::new();
        for wait_ms in [80, 8] {
            let mut samples = Vec::new();
            for _ in 0..60 {
                let start = std::time::Instant::now();
                let voice = t.meter_with_wait(0x91, wait_ms)?;
                let voice_ms = start.elapsed().as_secs_f64() * 1000.0;
                let playback = t.meter_with_wait(0x82, wait_ms)?;
                samples.push(serde_json::json!({"voice":voice,"playback":playback,"voice_ms":voice_ms,"pair_ms":start.elapsed().as_secs_f64()*1000.0}));
            }
            runs.push(serde_json::json!({"wait_ms":wait_ms,"samples":samples}));
        }
        println!("{}",serde_json::json!({"runs":runs}));
        return Ok(());
    }
    let before = s.inspect(&mut t)?;
    if std::env::args().any(|a| a == "--playback-eq-test") {
        if s.pending() { return Err("Restaure a sessão existente antes de testar os fones".into()); }
        let mut values = BTreeMap::new();
        for i in 0..10 {
            for (key, value) in [("enabled", i32::from(i == 0)), ("type", if i == 0 { 3 } else { 0 }),
                ("frequency", if i == 0 { 2500 } else { 200 }), ("q", 724), ("gain", 0)] {
                values.insert(format!("f{i}_{key}"), value);
            }
        }
        let result: Result<_> = (|| {
            let active = s.apply(&mut t, Effect::Playbackeq, true, values)?;
            for (block, words) in &before.effects {
                if *block != Block::PlaybackEq && active.effects.get(block) != Some(words) {
                    return Err(format!("Outro bloco mudou durante o teste: {block:?}"));
                }
            }
            if s.compare(&mut t)?.effects != before.effects { return Err("Comparação dos fones difere do original".into()); }
            if s.compare(&mut t)?.effects != active.effects { return Err("Retomada do EQ dos fones difere".into()); }
            println!("FONES_TESTE_ATIVO: passa-baixas de 2500 Hz por 60 segundos; sem aumento de volume.");
            for remaining in [50, 40, 30, 20, 10, 0] {
                std::thread::sleep(std::time::Duration::from_secs(10));
                println!("Tempo restante: {remaining}s");
            }
            Ok(active)
        })();
        let restoration = s.restore(&mut t);
        let arguments: Vec<_> = std::env::args().collect();
        if let Some(index) = arguments.iter().position(|a| a == "--report") {
            if let Some(path) = arguments.get(index + 1) {
                let report = serde_json::json!({"test":"playback_eq_lowpass_2500_60s","before":before,
                    "active":result.as_ref().ok(),"error":result.as_ref().err(),"after":restoration.as_ref().ok(),
                    "restoration_error":restoration.as_ref().err(),"audible_validation":"pending_user"});
                std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).map_err(|e| e.to_string())?;
            }
        }
        result?;
        if restoration?.effects != before.effects { return Err("Estado final dos fones diferente do original".into()); }
        println!("FONES_TESTE_RESTAURADO: todos os blocos conferem com o estado original.");
        return Ok(());
    }
    if std::env::args().any(|a| a == "--studio") {
        if s.pending() { return Err("Restaure a sessão existente antes do preset".into()); }
        if [Block::Pitch, Block::PitchPro, Block::VoicePro, Block::PlateNative].iter().any(|key| before.effects.get(key).is_some_and(|w| w[0] != 0)) {
            return Err("Desative tom, transformação e plate antes do preset de estúdio".into());
        }
        let mut eq = BTreeMap::new();
        let filters = [(4,80,724,0),(0,250,1024,-2),(0,3200,724,2),(2,8000,724,1)];
        for i in 0..10 {
            let (kind,hz,q,gain)=filters.get(i).copied().unwrap_or((0,200,724,0));
            for (key,v) in [("enabled",i32::from(i<filters.len())),("type",kind),("frequency",hz),("q",q),("gain",gain)] { eq.insert(format!("f{i}_{key}"),v); }
        }
        let values = |v: &[(&str,i32)]| v.iter().map(|(k,n)|((*k).to_string(),*n)).collect::<BTreeMap<_,_>>();
        let cases = [
            (Effect::Noise,values(&[("threshold",-45),("ratio",3),("attack",10),("release",250)])),
            (Effect::Eq,eq),
            (Effect::Compressor,values(&[("threshold",-18),("ratio",2),("attack",10),("release",200)])),
        ];
        let result: Result<_> = (|| {
            for (effect,values) in &cases {
                println!("Aplicando estúdio: {effect:?}");
                s.apply(&mut t,*effect,true,values.clone())?;
            }
            let active = s.inspect(&mut t)?;
            if s.compare(&mut t)?.effects != before.effects { return Err("Comparação difere do original".into()); }
            if s.compare(&mut t)?.effects != active.effects { return Err("Retomada do preset difere".into()); }
            Ok(active)
        })();
        let args: Vec<_> = std::env::args().collect();
        if let Some(i)=args.iter().position(|a| a=="--report") {
            if let Some(path)=args.get(i+1) {
                let report=serde_json::json!({"preset":"Estúdio natural","settings":cases,"before":before,"active":result.as_ref().ok(),"error":result.as_ref().err()});
                if let Err(e)=std::fs::write(path,serde_json::to_vec_pretty(&report).unwrap()) {
                    let recovery=s.restore(&mut t);
                    return Err(format!("Não foi possível salvar relatório: {e}. Restauração: {:?}",recovery.as_ref().err()));
                }
            }
        }
        match result {
            Ok(active) => { println!("{}",serde_json::to_string_pretty(&active).unwrap()); return Ok(()); }
            Err(e) => { let recovery=s.restore(&mut t);return Err(format!("{e}. Restauração: {:?}",recovery.as_ref().err())); }
        }
    }
    if std::env::args().any(|a| a=="--meters") {
        let mut values=Vec::new();
        for _ in 0..12 { values.push(serde_json::json!({"voice":read_meter(&mut t,0x91)?,"playback":read_meter(&mut t,0x82)?})); }
        println!("{}",serde_json::json!({"meters":values,"default_output":am8_lab::audio_status::default_output()}));return Ok(());
    }
    if std::env::args().any(|a| a == "--restore") {
        println!("{}", serde_json::to_string_pretty(&s.restore(&mut t)?).unwrap());
        return Ok(());
    }
    if !std::env::args().any(|a| a == "--validate-effects") {
        println!("{}", serde_json::to_string_pretty(&before).unwrap()); return Ok(());
    }
    if s.pending() { return Err("Restaure a sessão existente antes de validar".into()); }
    let raw_cases = [
        (Effect::Noise, vec![("threshold", -40),("ratio",4),("attack",10),("release",200)]), (Effect::Pitch, vec![("pitch", -1)]),
        (Effect::Eq, vec![("bass", -1), ("mid", 0), ("treble", 0)]),
        (Effect::Playbackeq, vec![("bass", -2), ("mid", 0), ("treble", 0)]),
        (Effect::Compressor, vec![("threshold", -18), ("ratio", 2), ("attack", 10), ("release", 200)]),
        (Effect::Pitchpro,vec![("pitch",-3)]),
        (Effect::Voicepro,vec![("pitch",100),("formant",110)]),
        (Effect::Reverb,vec![("mix",60),("room",60),("damping",50),("wet_gain",-6)]),
        (Effect::Plate,vec![("decay",40),("predelay",2500),("damping",50),("wet_gain",-6)]),
        (Effect::Feedback,vec![]),
    ];
    let mut cases: Vec<(Effect,BTreeMap<String,i32>)> = raw_cases.into_iter().map(|(effect,values)| (effect,values.into_iter().map(|(k,v)|(k.into(),v)).collect())).collect();
    if std::env::args().any(|a| a == "--dual-eq") {
        cases.retain(|(effect,_)| matches!(effect, Effect::Eq | Effect::Playbackeq));
    }
    if std::env::args().any(|a| a == "--playback-gain-test") {
        cases = vec![(Effect::Playbackeq, BTreeMap::from([
            ("bass".into(), 0), ("mid".into(), 0), ("treble".into(), 0), ("output_gain".into(), 1),
        ]))];
    }
    if std::env::args().any(|a| a=="--full-eq") {
        let mut values=BTreeMap::new();
        for i in 0..10 {
            for (key,v) in [("enabled",if i==0 {1} else {0}),("type",if i==0 {4} else {0}),("frequency",if i==0 {400} else {200}),("q",724),("gain",0)] { values.insert(format!("f{i}_{key}"),v); }
        }
        cases=vec![(Effect::Eq,values)];
    }
    let arguments: Vec<_>=std::env::args().collect();
    if let Some(index)=arguments.iter().position(|a| a=="--only") {
        let name=arguments.get(index+1).ok_or("Missing effect name")?;
        let effect: Effect=serde_json::from_value(serde_json::Value::String(name.clone())).map_err(|e| e.to_string())?;
        cases.retain(|(candidate,_)| *candidate==effect);
    }
    let mut tests = Vec::new();
    let test_result: Result<()> = (|| {
        for (effect, values) in cases.iter().filter(|_| !std::env::args().any(|a|a=="--combined-only")) {
            println!("Validating {effect:?}");
            let values=values.clone();
            let during = s.apply(&mut t, *effect, true, values)?;
            println!("Applied {effect:?}");
            let comparison = s.compare(&mut t)?;
            println!("Compared {effect:?}");
            if comparison.effects != before.effects { return Err("Comparação não restaurou o estado original".into()); }
            let resumed = s.compare(&mut t)?;
            println!("Resumed {effect:?}");
            if resumed.effects != during.effects { return Err("Retomada dos efeitos não confere".into()); }
            let after = s.restore(&mut t)?;
            if after.effects != before.effects { return Err("Restauração não confere".into()); }
            tests.push(serde_json::json!({"effect": effect, "during": during, "restored": true}));
            println!("Confirmed and restored {effect:?}");
        }
        if cases.len()>1 {
        println!("Validating supported combination");
        for (effect, values) in cases.iter().filter(|(e,_)| !matches!(e,Effect::Pitch | Effect::Pitchpro | Effect::Voicepro | Effect::Reverb)) {
            println!("Combining {effect:?}");
            s.apply(&mut t, *effect, true, values.clone())?;
        }
        let combined = s.inspect(&mut t)?;
        if s.compare(&mut t)?.effects != before.effects { return Err("Combined comparison differs".into()); }
        if s.compare(&mut t)?.effects != combined.effects { return Err("Combined resumption differs".into()); }
        tests.push(serde_json::json!({"combined": combined}));
        }
        Ok(())
    })();
    let restoration = s.restore(&mut t);
    let args: Vec<_> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--report") {
        if let Some(path) = args.get(i + 1) {
            let report = serde_json::json!({"before":before,"tests":tests,"error":test_result.as_ref().err(),"after":restoration.as_ref().ok(),"restoration_error":restoration.as_ref().err()});
            std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).map_err(|e| e.to_string())?;
        }
    }
    test_result?;
    let after = restoration?;
    if after.effects != before.effects { return Err("Estado final diferente do original".into()); }
    println!("All originals restored");
    Ok(())
}
