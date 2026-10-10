# Compilar no Windows

Requisitos: Windows x64, Rust stable com target MSVC e Visual Studio Build Tools com Desktop development with C++. WebView2 para executar. O frontend compilado já está no repositório; Node não é necessário para a compilação Rust.

```powershell
cargo install tauri-cli --version 2.12.1 --locked
cd src-tauri
cargo test --locked --lib
cargo tauri build --bundles nsis --config '{"bundle":{"createUpdaterArtifacts":false}}' -- --locked
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

Diagnóstico de novas revisões, sem serial ou escritas: `cargo run --release --locked --no-default-features --example identify -- --completo`. Consulte [compatibilidade](FIRMWARE-COMPATIBILITY.md).

## Publicar uma atualização assinada

Gere uma chave com `cargo tauri signer generate --ci --write-keys CAMINHO_FORA_DO_REPOSITORIO`. Guarde a chave privada fora do projeto e faça backup seguro. Não substitua a chave pública embarcada: clientes antigos confiam nela. Defina `TAURI_SIGNING_PRIVATE_KEY` com o caminho da chave e `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` no ambiente local, execute `cargo tauri build --bundles nsis -- --locked` e remova essas variáveis ao terminar. A chave nunca deve entrar no Git, ZIP ou logs.

Publique o setup, seu arquivo `.sig` e os hashes na Release. Só depois atualize `update.json` na raiz com a versão, URL exata do setup e conteúdo textual do `.sig`. A CLI 2.12.1 vincula a versão no comentário assinado; `requireSignedVersion` fica habilitado. O workflow gera artefatos não assinados para revisão e não publica atualizações. Não use esses artefatos no manifesto.

Para conferir o pacote publicado pelo atualizador nativo sem instalar nem acessar USB: `cargo run --locked --example signed-update-check`. Esse exemplo aceita a mesma versão somente para diagnóstico; o app normal continua bloqueando versões iguais e anteriores.
