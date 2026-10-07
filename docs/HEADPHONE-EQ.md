# Equalizador dos fones — versão 0.6.4

O equalizador dos fones ajusta o áudio do Windows reproduzido pelo AM8 e ouvido no P2 dele: amigos, músicas e jogos. O processamento acontece no DSP do AM8. Não exige dispositivo de áudio virtual, processamento de áudio no computador ou alteração da firmware.

## Caminho e controles

No fluxo verificado do firmware B5 0.7.1, o bloco `eq1` (`0x8D`) fica no caminho de reprodução, após o compressor dessa reprodução e antes da mistura com a voz do microfone. O EQ da voz é o bloco separado `eq7` (`0x99`). O app altera o EQ escolhido sem alterar os parâmetros do outro EQ ou abrir canais de mistura.

Na página **Equalizador**, selecione **Microfone** ou **Fones**. Cada destino tem dez filtros: pico, graves/agudos shelf, passa-baixas e passa-altas. A interface limita os ganhos a −6/+6 dB, as frequências a 20–16.000 Hz e compensa aumentos de ganho com o pré-ganho. A curva desenhada é uma estimativa; não foi obtida por medição acústica.

Os perfis novos podem incluir os dois destinos. Carregar apenas prepara os controles. Perfis antigos sem `playbackEq` preservam os controles atuais dos fones.

## Validação feita

- Em uma unidade AM8 normal por USB, B5 0.7.1, foi aplicado um passa-baixas de **2.500 Hz por 60 segundos**.
- O usuário confirmou, ouvindo o áudio do Windows pelo P2: **“Sim, ficou mais abafado”**.
- Os valores foram confirmados por leitura; a comparação com o estado original e a retomada do efeito foram verificadas.
- Ao terminar, os valores originais de todos os blocos observados foram restaurados exatamente, incluindo os dois equalizadores.
- Uma validação adicional no aparelho verificou escrita/leitura dos dois EQs individualmente e ativos ao mesmo tempo, comparação, retomada e restauração completa, sem erro de comunicação ou restauração. Esse teste de conferência não foi uma avaliação auditiva de combinações.
- Os 24 testes Rust incluem seis testes adicionados para isolamento do EQ dos fones, limites, falha de escrita/restauração, recuperação após reiniciar, vínculo com o serial e ambos os EQs ativos com originais independentes. O smoke da interface com transporte simulado passou.

Esse resultado confirma uma alteração audível no caminho de reprodução desta unidade. Não mede a resposta em frequência, a latência ou a qualidade de todas as combinações. Não confirma outras revisões de hardware/firmware.

## Uso e limites

1. Conecte os fones ao P2 do AM8.
2. No Windows e no aplicativo que estiver reproduzindo som, escolha o AM8 como dispositivo de saída.
3. Selecione **Fones**, prepare os ajustes e pressione **Aplicar**.
4. Use **Comparar com original** para alternar com os valores anteriores à sessão. Use **Restaurar sessão** ou feche normalmente para restaurá-los.

O app não detecta fisicamente o plugue P2 e não muda a saída do Windows. O EQ não afeta áudio enviado a outra placa de som. A monitoração direta da própria voz não foi caracterizada neste teste. Reverb permanece experimental e deve ficar desligado para o uso natural devido ao relato anterior de estalos na reprodução.

Se ocorrer uma falha de comunicação, reconecte o mesmo aparelho e restaure a sessão. Preserve o diário `%LOCALAPPDATA%/AM8Lab/efeitos-recuperacao.json` enquanto houver recuperação pendente.

Na versão 0.6.5, foi acrescentado [Ganho geral dos fones](HEADPHONE-GAIN.md), de 0 a +18 dB, mantendo a compensação dos filtros. Os testes históricos acima continuam delimitados à versão 0.6.4.
