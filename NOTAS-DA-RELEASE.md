# AM8 Lab 0.7.0

Modo nativo **Mono/Estéreo dos fones** com botão Aplicar, comparação e restauração. Uma unidade AM8 USB B5 0.7.1 confirmou a mudança e o retorno ao estéreo por escuta; a integração verificou o diário, os 14 campos do DAC e a recuperação após reabrir a sessão.

A página **Capacidades** agora mostra informações USB da revisão validada, formatos atuais do mixer Windows, vazão PCM calculada e clock do núcleo reportado pelo firmware. Uma medição de 60 segundos confirmou voz mono duplicada em dois canais USB no caminho atual. O estéreo da voz permanece em investigação.

62 testes Rust passaram. O executável e instalador x64 foram compilados; o teste integrado de interface terminou com saída 0. Aplicação e restauração também foram conferidas pela janela conectada ao AM8. Consulte [método e limites](docs/NATIVE-MODES.md) e [verificação](VERIFICACAO.md).

Feche o aplicativo normalmente antes de atualizar. O diário de modo usa versão 4; restaure pela versão nova antes de voltar a versões anteriores. Compatibilidade experimental restrita à revisão B5 0.7.1. Reverb continua experimental. Os binários não têm assinatura Authenticode; confira SHA256SUMS.txt.

---

# Histórico — AM8 Lab 0.6.9

O botão Aplicar continua manual, com menos espera na comunicação USB e menos leituras repetidas durante o mesmo comando. Em uma unidade AM8 B5 0.7.1, ruído caiu de 5,22 s para 1,32 s e tom padrão de 5,82 s para 1,33 s, incluindo verificação do aparelho, confirmação por leitura e estado completo após a escrita.

Nove ajustes foram aplicados, confirmados novamente após 400 ms e restaurados ao estado inicial. Efeitos mais complexos mantêm esperas de processamento e levam mais tempo. A medição não incluiu nova avaliação auditiva.

Verificação completa de compatibilidade, diário antes das escritas, confirmação, reversão e restauração preservados. Não há aplicação automática ao mover controles nem mudança na cadência das escritas. Consulte [a metodologia e os resultados](https://github.com/NightXXT/am8lab/blob/main/docs/PERFORMANCE.md).

Compatibilidade experimental: AM8 USB B5 0.7.1. Reverb permanece experimental, com o relato anterior de estalos. Binários sem assinatura Authenticode; confira SHA256SUMS.txt e feche o aplicativo normalmente antes de instalar.
