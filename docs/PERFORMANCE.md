# Tempo de aplicação — AM8 Lab 0.6.9

07/10/2026. O botão **Aplicar** foi mantido. Mover os controles continua preparando valores; o aplicativo só anuncia confirmação depois de receber o resultado do comando USB.

## O que mudou

- As respostas de controle HID são consultadas em intervalos de 8 ms, em vez de 80 ms. O orçamento de espera pela resposta permanece em 800 ms; erros e respostas incompatíveis são recusados.
- Leituras de preparação repetidas são reaproveitadas apenas durante o mesmo comando verificado. Não há cache de parâmetros entre comandos.
- O estado completo do aparelho é lido após a aplicação; alterações externas em blocos não afetados continuam aparecendo no resultado.
- A resposta do fluxo interno distingue seletor e sequência, inclusive quando modo e fluxo compartilham o mesmo opcode.

A verificação completa de firmware, modo e fingerprint continua antes dos comandos. O diário vinculado ao serial é salvo antes da primeira escrita; confirmação por leitura, reversão em caso de falha, comparação e restauração foram preservados. A cadência de 120 ms entre escritas e as esperas específicas de processamento não foram reduzidas.

## Medição real de aplicação

Uma unidade do AM8 normal USB B5 0.7.1, no mesmo computador Windows. O aplicativo foi fechado normalmente antes do diagnóstico, evitando consultas simultâneas. O tempo medido é o de `Session::apply` no Rust, até retornar confirmação e o snapshot completo; não inclui o clique, renderização da interface ou percepção auditiva.

Cada caso aplicou um ajuste temporário com diário de recuperação. O estado retornado e outra leitura após 400 ms corresponderam ao solicitado. A restauração por caso correspondeu ao estado inicial de todos os blocos e do ganho; ao terminar, não havia recuperação pendente. O comparativo anterior foi executado somente para ruído e tom padrão.

| Ajuste | 0.6.8: aplicar | 0.6.9: aplicar | 0.6.9: restaurar |
| :--- | ---: | ---: | ---: |
| Redução de ruído | 5,22 s | 1,32 s | 1,41 s |
| Tom padrão | 5,82 s | 1,33 s | 1,46 s |
| EQ do microfone | Não medido | 2,29 s | 2,27 s |
| EQ dos fones | Não medido | 2,17 s | 2,28 s |
| Compressor | Não medido | 1,71 s | 2,29 s |
| Tom Pro | Não medido | 2,20 s | 2,27 s |
| Transformação Pro | Não medido | 4,43 s | 6,43 s |
| Reverb Sala | Não medido | 2,76 s | 3,44 s |
| Reverb Plate | Não medido | 3,83 s | 4,21 s |

Esses tempos são observações de uma aplicação por caso, não médias de várias execuções nem uma garantia para todos os computadores. EQs alteraram dez palavras cada; transformações e reverbs envolvem parâmetros, roteamento e esperas adicionais. O resultado permanece confirmado antes de liberar os controles.

Não houve nova avaliação auditiva durante esta medição. Reverb continua experimental, com o relato anterior de estalos; o teste de comunicação não comprova que esse problema foi corrigido. Supressão de microfonia, ganho positivo dos fones e todas as combinações não receberam nova medição de aplicação nesta etapa.

## Medição de consultas, sem escritas

O diagnóstico comparou 18 blocos com a referência, em três passagens alternando a ordem, para cada intervalo. Todas as 162 leituras corresponderam à referência; nenhuma palavra foi escrita. O tempo da guarda inclui identidade, modo e fingerprint completos.

| Intervalo de consulta | Leituras | Mediana por leitura | Guarda completa |
| ---: | ---: | ---: | ---: |
| 80 ms | 54 | 86,0 ms | 1406 ms |
| 16 ms | 54 | 23,0 ms | 354 ms |
| 8 ms | 54 | 17,0 ms | 260 ms |

Esta medição sustenta a escolha de 8 ms nesta unidade/revisão. Não caracteriza todos os dispositivos USB, revisões desconhecidas ou possíveis desconexões. O limite de compatibilidade permanece o mesmo.

## Evidências e reprodução

Os relatórios publicados contêm somente tempos, contagens e resultados de confirmação/restauração, sem serial ou caminhos pessoais:

- [Consultas de controle](performance/control-timing-v069.json).
- [Aplicação anterior, 0.6.8](performance/apply-timing-v068-before.json).
- [Aplicação otimizada, 0.6.9](performance/apply-timing-v069-after.json).

Os exemplos são destinados a desenvolvedores e não são instalados pelo setup. Feche o aplicativo normalmente antes de executar. `am8-diagnostic --control-timing` apenas consulta o aparelho; o exemplo `apply-timing` escreve efeitos temporários e restaura por caso, recusando sessões já pendentes. Leia o código antes de reproduzir e preserve a recuperação caso a comunicação falhe.

```powershell
cd src-tauri
cargo run --locked --no-default-features --example am8-diagnostic -- --control-timing --report control-timing.json
cargo run --locked --no-default-features --example apply-timing -- --report apply-timing.json
```

As demais opções do diagnóstico podem emitir identidade pessoal do aparelho; não publique relatórios genéricos ou o diário de recuperação. Esta alteração não inclui nova auditoria RustSec; o escopo histórico está em [SECURITY-REVIEW.md](SECURITY-REVIEW.md).
