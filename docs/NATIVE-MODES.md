# Modos de áudio e informações do AM8 — 0.7.0

Validado em uma unidade AM8 normal USB, firmware B5 0.7.1. A implementação mantém os ajustes temporários e vinculados ao aparelho.

## Fones pelo P2

Na página **Fones**, prepare **Estéreo** ou **Mono** e clique em **Aplicar modo dos fones**. O modo estéreo preserva os canais; o mono envia a média dos dois canais aos dois lados. Isso ocorre no DAC do microfone e inclui o áudio que chega do computador por USB.

O teste físico manteve mono por 60 segundos com tons de 440 e 660 Hz em canais separados, aproximadamente −36 dBFS de pico e transições suaves. O usuário confirmou os dois tons nos dois ouvidos e, após restaurar, a alternância entre os lados. A leitura retornou ao estado original de todos os 14 campos do DAC, com efeitos e ganhos preservados.

O controle usa apenas DAC0 `0x09`, seletor 7: 0 para estéreo e 2 para mono. Os modos 1 e 3 não fazem parte dos controles liberados. A implementação salva o original no diário antes da primeira escrita, confirma o estado por leitura e integra o modo à comparação, à restauração e ao fechamento normal. Não altera taxas, volumes ou os conversores de canais da voz.

O diário com um ajuste de modo usa versão 4. Uma versão anterior do AM8 Lab deve recusar esse diário desconhecido, em vez de ignorar a recuperação. Feche e restaure pela versão nova antes de voltar a uma versão antiga.

## Voz: mono enviado em dois canais

O grafo validado usa `downmix_2to1` antes dos processadores mono e `upmix_1to2` depois. Os módulos têm formato fixo e nenhum parâmetro de seleção de canais na biblioteca extraída. Não foi encontrado um bypass seguro que libere captação espacial estéreo.

A medição de 60 segundos em 08/10/2026 usou captura WASAPI RAW compartilhada, dois canais, 48.000 Hz e float de 32 bits. Foram comparados **2.576.382 quadros acima do limiar de −60 dBFS**: 100% apresentaram canais iguais bit a bit, diferença RMS zero e correlação 1. O teste teve uma indicação de descontinuidade, sem erros de timestamp, amostras não finitas ou falha de captura. Nenhuma amostra de voz foi salva; o relatório contém estatísticas agregadas.

Isso confirma voz duplicada nos dois canais nesse caminho e nessa medição. Não prova o comportamento de todas as rotas possíveis. O estéreo da voz permanece **em investigação**, sem um seletor que sugira suporte já validado.

[RAW](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/ne-audioclient-audclnt_streamoptions) preserva processamento sempre ativo do endpoint, driver e hardware. A medição não é uma captura direta dos pacotes USB. O formato compartilhado do Windows também pode diferir do formato USB.

## Informações do aparelho

A página **Capacidades** reúne as especificações da revisão validada e leituras sob demanda do microfone e do Windows:

- USB: dois canais de captura e reprodução, 44.100/48.000 Hz e transporte PCM de 16/24 bits, Full Speed, UAC 1.0.
- Núcleo: 240 MHz reportados pelo diagnóstico desta unidade, separados da frequência de amostragem do áudio.
- Fluxo interno: índice interpretado como 44.100 Hz e quadros de 256 amostras, conforme o protocolo de referência.
- Formato atual do mixer compartilhado do Windows: consultado sem iniciar reprodução ou captura.
- Vazão nominal PCM: calculada com taxa, canais e bytes por amostra; exclui overhead USB.

Os 32 bits do mixer Windows não medem a resolução dos conversores. O transporte de 24 bits também não confirma resolução efetiva de 24 bits no ADC. O clock reportado não identifica o modelo físico do chip. A consulta de status limpa uma flag de atualização do diagnóstico segundo a referência; não envia setters de áudio.

As capacidades USB exibidas vêm dos descritores anteriormente conferidos na revisão B5 0.7.1. A interface não faz uma nova leitura dos descritores a cada atualização das informações.
