# Ganho geral dos fones — 0.6.5

O controle acrescenta ganho geral ao áudio do Windows reproduzido pelo AM8 e ouvido no P2, usando o pré-ganho do EQ nativo de reprodução (`eq1`, `0x8D`). O EQ da voz permanece independente. Não há captura ou processamento de áudio no computador.

## Limite e cálculo

O descritor extraído deste AM8 B5 0.7.1 limita o pré-ganho a −24576..4608 em Q8.8, isto é, −96..+18 dB. +20 dB exigiria 5120, acima desse limite. O app oferece ganho geral de 0 a +18 dB; não altera outros módulos para ultrapassar a faixa.

O pré-ganho enviado é `ganho geral − soma dos reforços positivos dos filtros ativos`. Filtros passa-altas/baixas não contribuem com ganho de banda. Exemplo: ganho geral +12 dB e reforços de +3 dB resultam em pré-ganho +9 dB. A interface mostra ganho geral, compensação e pré-ganho preparado, sem apresentar essa estimativa como medição acústica.

Ganho positivo pode saturar o áudio. O EQ está depois do compressor de reprodução; não há proteção automática por um limitador novo. O padrão é 0 dB e a alteração só é enviada ao pressionar Aplicar. O app não aplica ganho alto automaticamente.

## Uso

1. Use o AM8 como saída de reprodução no Windows ou no aplicativo de áudio.
2. Na página Equalizador, selecione Fones.
3. Marque Ativar EQ, prepare o Ganho geral dos fones e clique em Aplicar EQ dos fones.
4. Voltar a 0 dB prepara o ganho padrão; pressione Aplicar para enviá-lo. Comparar com original alterna com a sessão anterior e Restaurar sessão reverte os blocos completos.

Zerar filtros preserva o ganho geral preparado. Perfis antigos que contêm EQ dos fones sem o campo de ganho carregam com 0 dB; perfis sem EQ dos fones preservam os controles atuais desse destino. Carregar um perfil apenas prepara controles.

## Verificação

07/10/2026: 31 testes Rust aprovados, incluindo sete testes novos para extremos 0/+18 dB, compensação, recusa de valores inválidos, chamadas antigas, isolamento do EQ da voz, comparação, reaplicação, falha de escrita e recuperação após reiniciar. A interface foi verificada com transporte simulado, inclusive perfis antigos e a separação entre 50 parâmetros do microfone e 51 parâmetros dos fones.

No aparelho foi aplicado apenas +1 dB com bandas em 0 dB. O valor nativo 256 foi confirmado por leitura, comparação, retomada e restauração completa. Esse teste verifica comunicação e recuperação; não constitui avaliação auditiva nem confirma ausência de distorção em +18 dB. Não foi aplicado ganho alto no aparelho para essa entrega.

A validação audível anterior do caminho dos fones cobriu um corte de agudos por 60 segundos. Veja [o relatório do EQ](HEADPHONE-EQ.md). Outras unidades ou revisões permanecem bloqueadas pelo fingerprint e pelos limites de versão.
