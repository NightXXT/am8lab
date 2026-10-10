//! Read-only identification, without serial, audio samples or parameter setters.
use am8_lab::{protocol::{Result,Transport},windows::{HidDevice,InstanceMutex,ACCEPTED_FIRMWARE,GRAPH_LENGTH,GRAPH_SHA256,accepted_firmware,validate_mode_and_name,validate_flow}};
use sha2::{Digest,Sha256};
fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ") }
fn run() -> Result<()> {
    let args:Vec<_>=std::env::args().skip(1).collect();
    if args.iter().any(|a| a=="--help") { println!("identify [--completo]: identidade; --completo também verifica modo, nome e comprimento/hash do fluxo. Somente leitura, sem serial. Feche o AM8 Lab antes.");return Ok(()); }
    if args.iter().any(|a| a!="--completo") || args.len()>1 { return Err("Use identify [--completo]".into()); }
    let _instance=InstanceMutex::acquire()?;let mut device=HidDevice::open()?;
    let id=device.query(0,&[])?;
    println!("Identidade: {}",hex(&id));
    for (known,label) in ACCEPTED_FIRMWARE { println!("Aceita: {} ({label})",hex(&known)); }
    let label=accepted_firmware(&id);
    println!("Identidade aceita: {}",label.unwrap_or("não"));
    if args.is_empty() { return label.map(|_|()).ok_or_else(||"Identidade não suportada".into()); }
    let mode=device.query(0x80,&[6])?;let name=device.query(0xFC,&[])?;
    println!("Modo: {}",hex(&mode));println!("Fluxo: {:?}",String::from_utf8_lossy(&name));
    let (total,graph,complete)=device.read_flow()?;
    println!("Fluxo interno: declarado {total}; lido {}; completo {complete}",graph.len());
    println!("SHA-256: {}",hex(&Sha256::digest(&graph)).replace(' ',""));
    println!("Esperado: {GRAPH_LENGTH} bytes, {}",hex(&GRAPH_SHA256).replace(' ',""));
    validate_mode_and_name(&mode,&name)?;validate_flow(total,&graph,complete)?;
    let label=label.ok_or("Identidade não suportada")?;
    println!("Compatibilidade completa conferida: {label}. Efeitos e áudio não foram testados por este diagnóstico.");
    Ok(())
}
fn main() { if let Err(error)=run() { eprintln!("{error}");std::process::exit(1); } }
