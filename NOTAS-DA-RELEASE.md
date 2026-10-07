# AM8 Lab 0.6.9

O botão Aplicar continua manual, com menos espera na comunicação USB e menos leituras repetidas durante o mesmo comando. Em uma unidade AM8 B5 0.7.1, ruído caiu de 5,22 s para 1,32 s e tom padrão de 5,82 s para 1,33 s, incluindo verificação do aparelho, confirmação por leitura e estado completo após a escrita.

Nove ajustes foram aplicados, confirmados novamente após 400 ms e restaurados ao estado inicial. Efeitos mais complexos mantêm esperas de processamento e levam mais tempo. A medição não incluiu nova avaliação auditiva.

Verificação completa de compatibilidade, diário antes das escritas, confirmação, reversão e restauração preservados. Não há aplicação automática ao mover controles nem mudança na cadência das escritas. Consulte [a metodologia e os resultados](https://github.com/NightXXT/am8lab/blob/main/docs/PERFORMANCE.md).

Compatibilidade experimental: AM8 USB B5 0.7.1. Reverb permanece experimental, com o relato anterior de estalos. Binários sem assinatura Authenticode; confira SHA256SUMS.txt e feche o aplicativo normalmente antes de instalar.
