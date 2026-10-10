//! Measures real USB apply latency, with a recovery journal and restoration per case.
//! `--quick` tests noise/pitch only. `--report PATH` saves a report without serial numbers.
use am8_lab::{protocol::{self, Block, Effect, Result, Transport}, session::Session, windows::{HidDevice, InstanceMutex}};
use std::{cell::Cell, collections::BTreeMap, path::PathBuf, time::{Duration, Instant}};

struct Counted {
    device: HidDevice,
    queries: usize,
    writes: usize,
    waits_ms: Cell<u64>,
    guard_ms: f64,
}
impl Transport for Counted {
    fn firmware(&self) -> &'static str { self.device.firmware() }
    fn serial(&self) -> &str { self.device.serial() }
    fn query(&mut self, op: u8, payload: &[u8]) -> Result<Vec<u8>> { self.queries += 1; self.device.query(op, payload) }
    fn write_word(&mut self, block: Block, index: usize, value: i16) -> Result<()> {
        self.writes += 1; self.device.write_word(block, index, value)
    }
    fn guard(&mut self) -> Result<()> {
        let start = Instant::now(); let result = self.device.guard();
        self.guard_ms += start.elapsed().as_secs_f64() * 1000.0; result
    }
    fn wait(&self, ms: u64) { self.waits_ms.set(self.waits_ms.get() + ms); self.device.wait(ms); }
}
fn main() { if let Err(error) = run() { eprintln!("{error}"); std::process::exit(1); } }
fn run() -> Result<()> {
    let _instance = InstanceMutex::acquire()?;
    let path = PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA ausente")?).join("AM8Lab/efeitos-recuperacao.json");
    let mut session = Session::open(path)?;
    if session.pending() { return Err("Restaure a sessão existente antes de medir".into()); }
    let mut t = Counted { device: HidDevice::open()?, queries: 0, writes: 0, waits_ms: Cell::new(0), guard_ms: 0.0 };
    let before = session.inspect(&mut t)?;
    let raw = [
        (Effect::Noise, vec![("threshold", -45),("ratio",3),("attack",10),("release",250)]),
        (Effect::Pitch, vec![("pitch",-1)]),
        (Effect::Eq, vec![("bass",-1),("mid",0),("treble",0)]),
        (Effect::Playbackeq, vec![("bass",-1),("mid",0),("treble",0)]),
        (Effect::Compressor, vec![("threshold",-18),("ratio",2),("attack",10),("release",200)]),
        (Effect::Pitchpro, vec![("pitch",-1)]),
        (Effect::Voicepro, vec![("pitch",100),("formant",105)]),
        (Effect::Reverb, vec![("mix",30),("room",30),("damping",50),("wet_gain",-12)]),
        (Effect::Plate, vec![("decay",40),("predelay",1000),("damping",50),("wet_gain",-12)]),
    ];
    let quick = std::env::args().any(|a| a == "--quick");
    let mut tests = Vec::new();
    let test_result: Result<()> = (|| {
        for (effect, raw_values) in raw.into_iter().filter(|(e,_)| !quick || matches!(e, Effect::Noise | Effect::Pitch)) {
            let values: BTreeMap<String, i32> = raw_values.into_iter().map(|(k,v)|(k.into(),v)).collect();
            let expected = protocol::target(effect, true, &values, &before.effects[&effect.block()])?;
            t.queries=0; t.writes=0; t.waits_ms.set(0); t.guard_ms=0.0;
            let start=Instant::now();
            let active=session.apply(&mut t,effect,true,values)?;
            let apply_ms=start.elapsed().as_secs_f64()*1000.0;
            let queries=t.queries; let writes=t.writes; let wait_ms=t.waits_ms.get(); let guard_ms=t.guard_ms;
            if active.effects[&effect.block()] != expected { return Err(format!("Snapshot diferente do solicitado: {effect:?}")); }
            std::thread::sleep(Duration::from_millis(400));
            if protocol::read_block(&mut t,effect.block())? != expected { return Err(format!("Estado mudou após a confirmação: {effect:?}")); }
            let restore_start=Instant::now();
            let after=session.restore(&mut t)?;
            let restore_ms=restore_start.elapsed().as_secs_f64()*1000.0;
            if after.effects != before.effects || after.gain_db != before.gain_db { return Err("Restauração diferente do estado inicial".into()); }
            tests.push(serde_json::json!({"effect":effect,"apply_ms":apply_ms,"restore_ms":restore_ms,
                "queries_without_guard":queries,"writes":writes,"explicit_wait_ms":wait_ms,"guard_ms":guard_ms,"confirmed_after_400_ms":true,"restored":true}));
            println!("{effect:?}: aplicar {apply_ms:.0} ms, restaurar {restore_ms:.0} ms; confirmado.");
        }
        Ok(())
    })();
    let cleanup=session.restore(&mut t);
    let baseline_matched=cleanup.as_ref().is_ok_and(|after| after.effects==before.effects && after.gain_db==before.gain_db);
    let report=serde_json::json!({"version":env!("CARGO_PKG_VERSION"),"tests":tests,"error":test_result.as_ref().err(),
        "cleanup_error":cleanup.as_ref().err(),"baseline_matched":baseline_matched,"recovery_pending":session.pending()});
    let arguments: Vec<_>=std::env::args().collect();
    if let Some(index)=arguments.iter().position(|a| a=="--report") {
        let path=arguments.get(index+1).ok_or("Caminho do relatório ausente")?;
        std::fs::write(path,serde_json::to_vec_pretty(&report).unwrap()).map_err(|e|e.to_string())?;
    }
    test_result?; cleanup?;
    if !baseline_matched { return Err("Estado final diferente do estado inicial".into()); }
    println!("Todos os ajustes foram restaurados."); Ok(())
}
