//! Supervised hardware verification: only the validated DAC mode is changed.
//! Always attempts restoration; reports contain no serial or voice samples.
use am8_lab::{protocol::{self, Result}, session::Session, windows::{HidDevice, InstanceMutex}};
use std::{fs, path::PathBuf, time::Instant};

fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let report_path = PathBuf::from(args.get(1).ok_or("Informe o caminho absoluto do relatório")?);
    if !report_path.is_absolute() || report_path.exists() { return Err("Use um caminho absoluto novo para o relatório".into()); }
    let _instance = InstanceMutex::acquire()?;
    let mut device = HidDevice::open()?;
    let local = PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA ausente")?).join("AM8Lab");
    if local.join("teste-mono-fones-recuperacao.json").exists() { return Err("Recuperação de teste pendente".into()); }
    let journal_path = local.join("efeitos-recuperacao.json");
    let mut session = Session::open(&journal_path)?;
    if session.pending() { return Err("Restaure a sessão existente antes do teste".into()); }
    let baseline = session.inspect(&mut device)?;
    let before_dac = protocol::read_dac_words(&mut device)?;
    if before_dac[7] != 0 { return Err("Este teste exige a referência estéreo original".into()); }
    let info = am8_lab::device_info::read(&mut device)?;
    let started = Instant::now();
    let test: Result<_> = (|| {
        let active = session.apply_headphone_mode(&mut device, "mono")?;
        let active_dac = protocol::read_dac_words(&mut device)?;
        let mut expected = before_dac; expected[7] = 2;
        if active_dac != expected || active.effects != baseline.effects || active.gain_db != baseline.gain_db {
            return Err("Estado ativo diferente do ajuste exclusivo de modo".into());
        }
        let compared = session.compare(&mut device)?;
        if compared.headphone_mode.as_deref() != Some("stereo") || !compared.comparison
            || protocol::read_dac_words(&mut device)? != before_dac || compared.effects != baseline.effects {
            return Err("Comparação não voltou ao modo original".into());
        }
        let resumed = session.compare(&mut device)?;
        if resumed.headphone_mode.as_deref() != Some("mono") || resumed.comparison
            || protocol::read_dac_words(&mut device)? != active_dac || resumed.effects != active.effects {
            return Err("Retomada do modo mono não confirmou".into());
        }
        let persisted = fs::read(&journal_path).map_err(|error| error.to_string())?;
        let journal: serde_json::Value = serde_json::from_slice(&persisted).map_err(|error| error.to_string())?;
        if journal["version"] != 4 || journal["original_headphone_mode"] != 0 {
            return Err("Diário não preservou o modo original".into());
        }
        session = Session::open(&journal_path)?;
        if !session.pending() { return Err("Recuperação não carregou ao reabrir a sessão".into()); }
        Ok(serde_json::json!({"active_dac": active_dac, "comparison_stereo_confirmed": true,
            "resumed_mono_confirmed": true, "journal_version": 4, "restart_recovery_loaded": true}))
    })();
    let restored = session.restore(&mut device);
    let after_dac = protocol::read_dac_words(&mut device);
    let unchanged = restored.as_ref().is_ok_and(|snapshot| snapshot.effects == baseline.effects
        && snapshot.gain_db == baseline.gain_db && snapshot.headphone_mode.as_deref() == Some("stereo")
        && !snapshot.recovery_pending && !snapshot.comparison)
        && after_dac.as_ref().is_ok_and(|dac| *dac == before_dac);
    let report = serde_json::json!({"version": "0.7.0", "test": "integrated_headphone_mode_transaction",
        "firmware": "B5 0.7.1", "before_dac": before_dac, "after_dac": after_dac.as_ref().ok(),
        "test_result": test.as_ref().ok(), "error": test.as_ref().err(), "restore_error": restored.as_ref().err(),
        "final_dac_read_error": after_dac.as_ref().err(), "all_originals_restored": unchanged,
        "recovery_pending": session.pending(), "elapsed_ms": started.elapsed().as_millis(),
        "device_info": info, "writes_scope": "Only DAC0 0x09 selector 7, modes 0 and 2"});
    fs::write(&report_path, serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?).map_err(|error| error.to_string())?;
    restored?; test?; after_dac?;
    if !unchanged { return Err("O estado final não coincide com a referência; relatório salvo".into()); }
    println!("MODOS_VERIFICADOS: mono, comparação, retomada e recuperação após reabrir; tudo restaurado.");
    Ok(())
}

fn main() { if let Err(error) = run() { eprintln!("{error}"); std::process::exit(1); } }
