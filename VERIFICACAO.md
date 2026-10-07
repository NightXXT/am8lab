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
