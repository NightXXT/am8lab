<div align="center">

![AM8 Lab — Sua voz. O DSP do seu AM8.](docs/assets/banner.svg)

**Controle o processamento nativo do FIFINE AM8 direto pelo Windows.**

[Instalação](#instalação) · [Recursos](#recursos) · [Compatibilidade](#compatibilidade) · [Segurança](SECURITY.md) · [English](README.en.md)

</div>

---

O **AM8 Lab** é um aplicativo experimental e não oficial que ajusta os efeitos do DSP do **FIFINE AM8 USB**. A interface prepara os controles; o chip do microfone processa a voz.

Construído com **Rust + Tauri**, com interface em português, medidores de áudio e comparação com o som original da sessão.

> [!IMPORTANT]
> Validado inicialmente em **um AM8 normal USB, firmware B5 0.7.1**. Outras revisões podem ser diferentes e são recusadas pelo aplicativo. O projeto não é afiliado nem endossado pela FIFINE.

## Recursos

| Recurso | O que você pode ajustar |
| :--- | :--- |
| 🎙️ **Redução de ruído** | Limiar, intensidade, ataque e liberação para atenuar sons baixos entre as palavras. |
| 🎚️ **Equalizador** | Dez filtros: pico, graves/agudos shelf, passa-altas e passa-baixas; curva estimada e compensação de ganho. |
| 📊 **Compressor** | Uma faixa, com limiar, razão, ataque e liberação. |
| 🎵 **Tom da voz** | Algoritmo padrão e Pro; Pro limitado a −3/+3 semitons nesta versão. |
| 🧬 **Transformação Pro** | Controles independentes de altura e timbre. |
| 🪩 **Reverb Sala/Plate** | Disponível como experimental; há relato de estalos na reprodução. |
| 🔉 **Supressão de microfonia** | Experimental; eficácia acústica ainda não medida. |
| 💾 **Perfis locais** | Salve os controles no computador e carregue quando quiser. |
| ↔️ **Comparar e restaurar** | Alterne entre os efeitos e os valores anteriores à sessão; restaure todos os ajustes afetados. |

**Tom, transformação e reverb funcionam um por vez.** Ruído, EQ e compressor podem ser combinados.

### Uma interface para acompanhar seu áudio

- Barras finas com leituras HID reais de **voz** e **reprodução**.
- Consulta da saída padrão do Windows, sem mudar o roteamento.
- Controles em português, tema escuro e transições discretas.
- Detecção automática do AM8 USB compatível.

Os medidores usam uma **escala visual relativa**, sem calibração em dB. A resposta da reprodução foi observada, mas sua sincronização e escala ainda não foram completamente caracterizadas. A curva do EQ é uma estimativa, não uma medição do áudio.

## Instalação

**Requisitos:** Windows 10/11 x64, WebView2 e AM8 USB compatível.

1. Abra a aba **Releases** deste repositório.
2. Baixe `AM8-Lab-Setup-v0.6.3.exe` e `SHA256SUMS.txt`.
3. Confira o hash conforme [o guia de instalação](docs/INSTALL.md) e execute o setup.
4. Conecte o AM8 por **USB** e abra o programa.

O setup instala para o usuário atual, cria atalhos e inclui desinstalador. Se o WebView2 estiver ausente, o bootstrapper oficial da Microsoft precisará de internet para instalar o runtime. A compatibilidade do microfone é conferida ao abrir o aplicativo; ele não precisa estar conectado durante a instalação.

<details>
<summary><strong>Prefere uma versão portátil?</strong></summary>

Baixe `AM8-Lab-Portable-v0.6.3.zip` nas Releases, confira o hash, extraia a pasta e abra `AM8-Lab.exe`. Mantenha os arquivos de licença que acompanham o programa. WebView2 precisa estar instalado.

</details>

> [!NOTE]
> Os executáveis ainda **não têm assinatura digital Authenticode**. O SHA-256 confere integridade em relação à lista da Release; não substitui uma assinatura nem garante confiança no arquivo.

## Começando

### 1 · Conecte

Abra o aplicativo com o AM8 ligado por USB. Aguarde **“AM8 conectado e verificado”**. O programa confere a identidade, versão, layout HID e fingerprint do fluxo interno.

### 2 · Prepare e aplique

Ajuste os controles e pressione **Aplicar** no efeito desejado. Mover um slider ou carregar um perfil prepara os valores; isso não envia todos os ajustes automaticamente.

### 3 · Ouça e compare

Use **Comparar com original** para alternar com os valores anteriores à sessão. Essa referência preserva o que já existia antes de abrir os efeitos; não é necessariamente um microfone sem processamento.

### 4 · Restaure quando terminar

Use **Restaurar sessão** ou feche normalmente. O programa restaura os parâmetros e canais afetados antes de sair. Se a comunicação falhar, reconecte e restaure.

### Um ponto de partida para voz natural

Experimente **ruído + EQ suave + compressor leve**, com tom, transformação e reverb desligados. Ouça em volume confortável e ajuste conforme sua voz e ambiente. Não existe um preset ideal para todas as pessoas.

## Compatibilidade

| Item | Configuração |
| :--- | :--- |
| Sistema | Windows 10/11 x64 — sistemas alvo |
| Microfone | FIFINE AM8 normal, conectado por USB |
| Firmware | **B5 0.7.1** |
| Biblioteca / engine | **2.43.2 / 2.23.2** |
| USB | VID:PID **3142:A010**, interface **MI_04** |
| Fluxo | Modo **HunXiang** e fingerprint conhecido |

A identidade do aparelho é verificada antes de permitir alterações. **Não remova essa proteção para aceitar outro firmware sem investigação e testes.** Os testes físicos iniciais cobriram apenas uma unidade/revisão.

<details>
<summary><strong>Detalhes da verificação e recuperação</strong></summary>

O fluxo interno é comparado por comprimento e SHA-256. O pacote público não inclui o grafo bruto, firmware ou DLL extraída do aparelho.

Antes de escrever, o aplicativo salva os valores originais em um diário vinculado ao serial. Ele confirma os ajustes por leitura e tenta restaurar os valores afetados em caso de falha.

Diário local: `%LOCALAPPDATA%/AM8Lab/efeitos-recuperacao.json`.

Não edite, apague ou copie esse arquivo para outro microfone quando houver uma sessão pendente. Recuperações sem serial válido são recusadas e preservadas. Reconecte o mesmo aparelho e use a restauração.

</details>

## Limitações conhecidas

> [!WARNING]
> **Reverb é experimental.** Foram relatados estalos ao ouvir áudio nos fones; eles pararam ao desativar o reverb. A causa técnica não foi determinada. Para uso natural, mantenha-o desligado.

- Supressão de microfonia ainda não teve eficácia acústica medida.
- XLR sozinho não transporta os comandos USB.
- Não há controle de RGB nem detecção física de fones no conector P2.
- Afinação automática, echo e compressor multibanda estão fora desta versão.
- Nem todos os valores e combinações receberam avaliação auditiva.
- Ajustes e perfis não são gravados na firmware.

## Segurança e privacidade

A interface permite somente comandos próprios tipados e escuta de eventos. Os valores são validados no Rust; o app não oferece comandos genéricos de shell, rede ou arquivos pela interface. Ele não grava voz, não faz upload de áudio e não inclui telemetria de rede.

Na revisão inicial, **18 testes Rust passaram**, além dos testes de interface, instalação e desinstalação. A consulta ao RustSec encontrou zero alertas classificados como vulnerabilidades e dois avisos informativos de dependências fora do alvo Windows.

Isso **não garante ausência de vulnerabilidades**. Consulte a [revisão e seu escopo](docs/SECURITY-REVIEW.md) e a [política de segurança](SECURITY.md). O instalador pode precisar de internet para instalar WebView2.

## Encontrou um problema?

Abra uma **Issue** com:

- versão do aplicativo, Windows e firmware exibida;
- conexão usada e onde você ouviu o áudio;
- efeitos ativos e passos para reproduzir;
- resultado esperado, resultado observado e se restaurar resolveu.

Não envie serial, caminhos pessoais, gravações privadas ou o diário completo. Para falhas de segurança, siga [SECURITY.md](SECURITY.md).

## Para desenvolvedores

O código foi organizado para separar a interface dos comandos e da comunicação USB:

```text
am8-lab/
├── ui/                    # Interface HTML, CSS e JavaScript
├── src-tauri/
│   ├── src/               # SDK, validação, sessões e integração Windows
│   ├── examples/          # Diagnóstico para desenvolvedores
│   ├── capabilities/      # Permissões da interface
│   └── data/              # Metadados e fixtures dos testes
├── docs/                  # Instalação, compilação e revisão
└── .github/               # CI Windows e atualização de dependências
```

[Compilar no Windows](docs/BUILD.md) · [Contribuir](CONTRIBUTING.md) · [Origem dos dados](docs/PROVENANCE.md) · [Dependências](docs/DEPENDENCIES.json)

O workflow verifica testes e dependências e gera o setup como artefato. Ele precisa ser habilitado no repositório; ainda não foi executado no GitHub nesta entrega local.

---

<div align="center">

**Feito para explorar o AM8, com ajustes temporários e restauráveis.**

[MIT](LICENSE) · [Avisos de terceiros](THIRD-PARTY-NOTICES.md)

FIFINE é marca de seus titulares. Projeto independente, sem afiliação com o fabricante.

</div>
