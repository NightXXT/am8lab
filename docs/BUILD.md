# Compilar no Windows

Requisitos: Windows x64, Rust stable com target MSVC e Visual Studio Build Tools com Desktop development with C++. WebView2 para executar. O frontend compilado já está no repositório; Node não é necessário para a compilação Rust.

```powershell
cargo install tauri-cli --version 2.12.1 --locked
cd src-tauri
cargo test --locked --lib
cargo tauri build --bundles nsis -- --locked
```

Saída: `src-tauri/target/release/bundle/nsis/*-setup.exe`. Executável portátil: `src-tauri/target/release/am8-lab.exe`. Nunca inclua `target` no Git.

```powershell
cargo install cargo-audit --version 0.22.2 --locked
cargo audit --file Cargo.lock
```

Auditoria usa o catálogo atual RustSec e pode apontar novas informações desde a revisão desta versão. Não ignore avisos automaticamente. `--smoke-test` no executável verifica interface/perfis sem comunicação USB; `--design-preview` também não acessa o aparelho. Testes Rust usam transportes falsos. O diagnóstico é um exemplo para desenvolvedores; não é instalado pelo setup. Compilar opcionalmente: `cargo build --locked --example am8-diagnostic` (saída target/debug/examples).

Para recompilar CSS após mudanças de classes, use o Tailwind standalone 3.4.17 oficial:

```powershell
.\tailwindcss.exe -i input.css -o ui/styles.css -c tailwind.config.cjs --minify
```

O CI verifica Rust e dependências no Windows e gera o setup como artefato. Seu workflow precisa ser ativado no GitHub depois do upload; ainda não foi executado no GitHub nesta entrega local.

O exemplo `am8-diagnostic --validate-effects --playback-gain-test --report arquivo.json` verifica ganho dos fones de +1 dB, comparação e restauração. Feche o app antes de usar o diagnóstico. O relatório contém identidade do aparelho e deve permanecer privado; não o envie ao GitHub.

Para verificar a consulta HTTPS real sem acessar o microfone: `cargo run --locked --no-default-features --example updates-check`. Esse exemplo consulta o GitHub sob demanda e não abre o navegador nem instala arquivos. Os testes de seleção das Releases já fazem parte de `cargo test --locked --lib`.
