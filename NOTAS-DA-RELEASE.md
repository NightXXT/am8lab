# AM8 Lab 0.7.2 — atualizações pelo aplicativo

- Download dentro do app, com progresso e assinatura Minisign vinculada à versão.
- Botão Instalar e reiniciar: restaura o microfone antes de iniciar o setup. Se houver falha na restauração, a instalação é bloqueada.
- Suporte experimental às identidades B5 0.7.1 e B5 0.7.3 com o mesmo fluxo conhecido. B5 0.7.3 ainda precisa de teste físico de efeitos e restauração.
- Diagnóstico identify somente de leitura; campos USB históricos ficam identificados como não verificados no 0.7.3.

Versões antigas até 0.7.1 precisam instalar este setup uma vez para adotar o novo atualizador. O setup não tem assinatura Authenticode; a assinatura do atualizador é uma verificação separada. Não atualiza firmware. Reverb continua experimental por relato de estalos.

# AM8 Lab 0.7.1 — entrega local

Suporte experimental ao firmware B5 0.7.3, preservando B5 0.7.1. Aceitação por identidade exata, modo, nome e fluxo completo de 2931 bytes/SHA-256. A checagem periódica preserva a identidade exata da conexão; firmware real no snapshot e na interface.

Novo diagnóstico identify somente de leitura. Dados USB estáticos observados no B5 0.7.1 não são atribuídos ao 0.7.3. A evidência 0.7.3 vem do resumo técnico fornecido pelo usuário; aplicação, restauração e escuta nessa revisão continuam pendentes. Consulte [compatibilidade](docs/FIRMWARE-COMPATIBILITY.md) e [verificação](VERIFICACAO.md). Esta entrega não está publicada no GitHub.

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
