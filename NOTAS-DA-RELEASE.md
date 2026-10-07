# AM8 Lab 0.6.7

Interface redesenhada a partir do material do Google Stitch: estúdio grafite com acentos violeta, navegação Estúdio/Voz/Fones/Perfis/Capacidades e medidores finos. Cada módulo de voz mantém seu Aplicar e estado próprio. Fones abre o EQ de reprodução, independente do EQ da voz.

A página Capacidades mostra o que foi confirmado por escuta, conferido por comunicação/restauração e o que continua experimental. O design usa recursos locais/do sistema, sem fontes ou ícones carregados de CDN. Medidores continuam relativos e a curva do EQ continua estimada; números fictícios do mockup foram removidos.

Backend nativo, limites, diário, comparação/restauração e consulta manual de atualizações mantidos. Esta mudança não acrescenta processamento de áudio no PC, efeitos novos ou validação auditiva. Carregar perfil prepara controles; aplicar envia os ajustes.

Compatibilidade AM8 normal USB B5 0.7.1 e fluxo conhecido. Ganho dos fones 0 a +18 dB; positivo pode distorcer e não há limitador automático. Reverb e supressão de microfonia continuam experimentais. Binários sem assinatura Authenticode; confira SHA256SUMS.txt e feche o app normalmente antes de instalar.

O resumo dos testes está em VERIFICACAO.md. Para o botão Atualizações reconhecer esta Release, o setup deve se chamar exatamente AM8-Lab-Setup-v0.6.7.exe.
