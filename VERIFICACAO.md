# Verificação da entrega 0.7.0

08/10/2026. Controle mono/estéreo dos fones e área de informações do AM8 adicionados. A seleção continua preparada até pressionar Aplicar.

- **62 testes Rust passaram**, incluindo limites/layout do DAC, exclusão de outros campos, diário antes das escritas, serial, rollback, recuperação após reabrir e comparação coordenada com efeitos.
- Teste físico anterior de 60 segundos: mono e retorno ao estéreo confirmados pelo usuário no P2. Integração nova: aplicação, comparação, retomada e recuperação após reabrir verificadas no aparelho; os 14 campos do DAC retornaram ao estado inicial, sem recuperação pendente.
- Captura RAW de 60 segundos: 2.576.382 quadros relevantes acima de −60 dBFS; 100% dos canais L/R iguais bit a bit, diferença RMS zero e correlação 1. Formato compartilhado 48 kHz/2 canais/float32. Uma indicação de descontinuidade, nenhum erro de timestamp ou captura. Apenas estatísticas foram salvas.
- Informações conferidas no backend e na interface nativa: núcleo reportado 240 MHz, taxa interna inferida 44,1 kHz, quadro inferido 256 amostras, mixer Windows 48 kHz/2 canais/32 bits nos dois sentidos. Capacidades USB previamente identificadas: 44,1/48 kHz, dois canais e transporte 16/24 bits.
- Compilação release x64 e instalador NSIS **0.7.0** concluídos com saída 0. ProductVersion 0.7.0; **--smoke-test terminou com saída 0** e preservou o arquivo de recuperação existente.
- Conferência real pela janela: Mono preparado sem aplicar automaticamente, confirmação após Aplicar e restauração para Estéreo; informações exibidas após consulta sob demanda. Layout conferido na janela de 1440 px.

Relatórios sem serial ou caminhos pessoais em [docs/native-modes](docs/native-modes). Método e limites em [NATIVE-MODES.md](docs/NATIVE-MODES.md). Os resultados valem para a unidade B5 0.7.1 testada. RAW não equivale a capturar os pacotes USB; 32 bits no mixer não medem resolução física do ADC. O estéreo da voz continua em investigação.

O instalador não foi executado para atualizar a instalação anterior nesta verificação. Os resultados locais acima precedem a publicação no GitHub. Não houve nova auditoria RustSec ou execução aprovada de CI. Dependências do aplicativo permanecem as da 0.6.9. Binários sem assinatura Authenticode.

---

# Histórico — verificação da entrega 0.6.9

07/10/2026. O botão Aplicar foi mantido; a consulta das respostas USB foi acelerada e leituras de preparação foram deduplicadas dentro de cada comando. Guardas de compatibilidade, diário antes das escritas, confirmação por leitura, snapshot completo após a aplicação e restauração foram preservados.

- 47 testes Rust aprovados na versão 0.6.9, incluindo guardas de sessão, falhas/recuperação e deduplicação de leituras. O estado completo após as escritas continua observando alterações externas nos blocos não afetados.
- Medição física em um AM8 normal USB B5 0.7.1: 162 consultas de 18 blocos, em intervalos de 80/16/8 ms, corresponderam à referência, sem escritas.
- Nove ajustes foram aplicados, confirmados novamente após 400 ms e restaurados ao estado inicial. Ruído: 5,22 s → 1,32 s; tom padrão: 5,82 s → 1,33 s. Esses tempos incluem o comando Rust, guardas, confirmação e snapshot; são uma execução por ajuste, sem medição do clique ou renderização da interface. Efeitos mais complexos continuam levando mais tempo.
- Estado inicial de todos os blocos e do ganho preservado após cada caso e ao terminar; diário sem recuperação pendente. Relatórios públicos sem serial ou caminhos pessoais em docs/performance; metodologia em docs/PERFORMANCE.md.
- Compilação release x64 e instalador NSIS 0.6.9 concluídos com saída 0.
- Executável ProductVersion 0.6.9; --smoke-test terminou com saída 0, sem comunicação USB. O diário existente permaneceu intacto durante o smoke.

Não houve nova avaliação auditiva, auditoria RustSec, instalação/desinstalação ou execução aprovada de CI no GitHub nesta verificação. A cadência de escritas e as esperas específicas de processamento foram mantidas. As medições físicas cobriram uma unidade/revisão; não garantem os mesmos tempos em outros computadores. Reverb continua experimental e esta alteração não afirma corrigir os estalos relatados anteriormente.

Binários sem assinatura Authenticode. A compatibilidade permanece restrita ao AM8 USB B5 0.7.1 com fluxo conhecido.

---

Os resultados abaixo são históricos.

# Verificação da entrega 0.6.8

- Compilação release e instalador NSIS concluídos.
- Executável: ProductVersion 0.6.8; --smoke-test terminou com saída 0, sem comunicação USB.
- Prévia em 1440×900 e 1100×720: sem overflow horizontal; estatísticas à direita e abaixo, respectivamente; ilustração ausente.
- Alteração visual; não houve nova auditoria de dependências, avaliação auditiva ou instalação/desinstalação nesta entrega.
- A execução anterior de CI no GitHub foi bloqueada por cobrança da conta antes de iniciar um runner; não há resultado aprovado de CI nesta alteração.

Os resultados anteriores abaixo são históricos, referentes à entrega 0.6.7.

---

# Verificação — AM8 Lab 0.6.7

07/10/2026. Redesign baseado na exportação de Voz e no guia visual do Stitch, adaptado às funções existentes do AM8 Lab. Backend de DSP, limites, compatibilidade e recuperação preservados. Não houve nova avaliação auditiva nem alteração real de parâmetros USB durante esta implementação.

- 42 testes Rust aprovados na versão 0.6.7.
- 28 verificações da nova interface com transporte USB simulado aprovadas: navegação, independência dos EQs e seus 50/51 parâmetros, ganho exclusivo dos fones, preservação dos valores preparados, perfis sem escritas, comparação, conflitos dos efeitos, falhas/recuperação e medidores novos/expirados. As 17 verificações de atualizações e o smoke anterior de ganho dos fones também passaram.
- Todas as cinco páginas conferidas visualmente em 1440×900 e 1100×720, incluindo atualizações e estado desconectado, sem overflow horizontal ou erros JavaScript. Movimento reduzido respeitado e medidores sem transições de largura/posição. As prévias capturadas são simuladas; a imagem entregue identifica isso.
- Compilação release x64 e setup NSIS concluídos. Smoke do executável portátil e do executável instalado terminou com código 0.
- Instalação isolada confirmou versão 0.6.7. Desinstalação terminou com código 0, sem executável remanescente; o diário original de recuperação permaneceu intacto.
- O executável instalado corresponde ao portátil após normalizar a marca Tauri de distribuição: três bytes entre `UNK` e `NSS`.

WebView2 já estava instalado; o ramo sem WebView2 não foi executado. O fluxo de atualizações não recebeu mudança no backend nem nova consulta real nesta etapa: a consulta WinHTTP anterior está documentada em docs/UPDATES.md. A abertura real do download no navegador não foi acionada durante estes testes.

Fontes públicas curadas, sem firmware, DLL extraída, grafo bruto, serial real, relatórios privados ou caminhos pessoais de compilação. A interface usa estilos, SVGs e fontes locais/do sistema. As versões das dependências e permissões foram preservadas; a auditoria RustSec histórica da 0.6.3 não foi repetida e não garante ausência de vulnerabilidades atuais.

Compatibilidade restrita ao AM8 USB B5 0.7.1 com fluxo conhecido. Binários sem assinatura Authenticode. Artefatos preparados localmente; GitHub e Actions não foram alterados por esta entrega.
