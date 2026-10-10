# Firmware compatível — AM8 Lab 0.7.2

A versão do aplicativo **0.7.2** inclui suporte experimental ao firmware **B5 0.7.3**. Os números do aplicativo e do firmware são independentes. O programa não grava nem atualiza firmware.

São aceitas somente duas identidades completas de 14 bytes:

| Firmware | Resposta de identidade |
| --- | --- |
| B5 0.7.1 | `42 00 07 01 02 2b 02 02 17 02 02 42 35 01` |
| B5 0.7.3 | `42 00 07 03 02 2b 02 02 17 02 02 42 35 01` |

Biblioteca 2.43.2 e engine 2.23.2 permanecem obrigatórias. O guard também exige VID:PID 3142:A010, MI_04, layout HID conhecido, modo de sete bytes com prefixo `00 ff 00 06`, byte 5 zero e marcador final 1; nome `HunXiang`; fluxo interno completo de 2931 bytes, comprimento declarado correspondente e SHA-256 `bd9dbe74994cbb96cb2981c6bb340aae5b894409cf290378c6e0201ffc2f131d`.

O rótulo só é aceito após a verificação completa. A checagem periódica compara a identidade exata da conexão atual, detectando também a troca entre 0.7.1 e 0.7.3. Falhas apagam o estado validado e exigem reconexão. O diário continua vinculado ao serial; os parâmetros e o modo dos fones continuam com confirmação e restauração.

## Evidência e limites

O resumo técnico fornecido pelo usuário em 09/10/2026 relata leituras de uma segunda unidade B5 0.7.3 feitas em 08/10/2026: identidade acima, modo `00 ff 00 06 00 00 01`, nome HunXiang e grafo com comprimento/hash idênticos ao aparelho B5 0.7.1. Essa evidência é fornecida pelo usuário, somente de leitura, e não foi replicada fisicamente nesta entrega. Os testes de 47 casos citados no resumo pertencem ao patch antigo sobre AM8 Lab 0.6.9.

Os testes físicos históricos de efeitos, escuta, mono/estéreo dos fones e recuperação pertencem ao B5 0.7.1. Aplicação, restauração e resultado audível no B5 0.7.3 continuam pendentes. Igualdade do grafo do DSP não comprova descritores USB idênticos. Por isso, taxas, canais, formatos e vazão USB da 0.7.1 não são apresentados como verificados na 0.7.3. Mixer Windows e diagnóstico interno continuam sendo consultados separadamente; o clock reportado não é medição física.

## Diagnóstico somente de leitura

Feche o AM8 Lab antes de executar, para evitar consultas simultâneas. A partir de src-tauri:

```powershell
cargo run --release --locked --no-default-features --example identify
cargo run --release --locked --no-default-features --example identify -- --completo
```

Sem flag: só identidade. Com --completo: identidade, modo, nome e comprimento/hash do fluxo. Não imprime serial nem grafo bruto, não grava áudio e não altera parâmetros. Código de saída diferente de zero indica identidade desconhecida, comunicação incompleta ou divergência. A identidade conhecida isoladamente não significa guard completo aprovado.

Os downloads públicos 0.7.0 aceitam somente B5 0.7.1. Esta alteração pertence à entrega local 0.7.1; não indica que uma Release nova foi publicada. Consulte [a verificação desta entrega](../VERIFICACAO.md).
