# Redesign do estúdio — 0.6.7

A versão 0.6.7 aplica ao AM8 Lab o material visual fornecido pelo usuário, exportado do Google Stitch. O pacote de referência contém o HTML de uma tela de Voz (`code.html`) e um guia de design (`DESIGN.md`). A imagem `screen.png` do export não contém uma prévia utilizável e não foi usada como referência visual. O pacote não contém versões completas e funcionais das demais telas nem uma biblioteca de áudio.

## Como o material virou a interface

| Referência do Stitch | Implementação no AM8 Lab |
| --- | --- |
| Estética de equipamento de estúdio | Superfícies grafite, divisórias finas, acentos violeta e hierarquia de leitura. |
| Módulos de processamento de voz | Cartões ligados aos controles existentes, com Aplicar por efeito e indicação de estado. |
| Navegação lateral | Estúdio, Voz, Fones, Perfis e Capacidades; telas complementares adaptadas às funções reais. |
| Medidores compactos | Leituras HID reais em escala visual relativa, sem recalibrar o áudio pelo design. |
| Curva e controles do EQ | Curva estimada e dez filtros já suportados, mantendo a separação Voz/Fones. |
| Indicadores de recursos | Categorias de evidência, com limites e recursos experimentais visíveis. |
| Componentes, tipografia e animações | HTML/CSS, SVGs locais e fontes do sistema; transições discretas e respeito à preferência de movimento reduzido. |

As telas não exportadas foram construídas seguindo a mesma linguagem visual, sem fingir que o Stitch forneceu um aplicativo completo. O código exportado foi adaptado; controles visuais sem função no AM8 não foram mantidos como recursos operacionais.

## Controles e comportamento preservados

- Cada efeito da voz é enviado por seu próprio botão Aplicar. Mover controles ou carregar um perfil prepara valores.
- O EQ do microfone e o EQ dos fones têm destinos independentes, com dez filtros e limites já existentes. Fones abre o destino de reprodução; a seleção do destino permanece disponível no equalizador.
- Ganho geral dos fones continua de 0 a +18 dB, com compensação de reforços dos filtros. Ganho positivo pode distorcer; não existe novo limitador automático.
- Tom, transformação e reverb continuam mutuamente exclusivos. Reverb e supressão de microfonia mantêm a indicação experimental.
- Comparação usa os valores anteriores à sessão. Restauração, diário local, fingerprint e bloqueio de revisões desconhecidas permanecem no Rust.
- Perfis continuam locais, limitados a 20. Atualizações continuam manuais e usam o mesmo destino NightXXT/am8lab; o novo visual não instala programas sozinho.

## Medidores e dados que não foram inventados

O export do Stitch contém exemplos visuais e números demonstrativos. A implementação não usa esses números como resultados reais. Não exibe latência ou CPU fictícias, níveis calibrados em dBFS, detecção real de clipping, phantom power, FFT, forma de onda de gravação ou uma especificação de áudio não consultada no aparelho.

Os medidores de voz e reprodução recebem valores HID e mostram uma escala relativa. A subida visual não acrescenta uma espera de um segundo; o processamento e a amostragem continuam sujeitos ao caminho existente de comunicação e áudio. Pico visual não é uma medição certificada de clipping. A curva do EQ é calculada a partir dos controles, sem medir o som recebido.

## Capacidades e limitações

A página Capacidades separa três tipos de evidência: confirmado por escuta, comunicação/restauração verificadas e experimental. O alcance continua uma unidade de AM8 normal USB B5 0.7.1 com fluxo conhecido. Não é uma promessa para toda revisão, nem uma declaração oficial do fabricante.

RGB, afinação automática, eco e compressor multibanda continuam fora dos controles. A porta P2 não fornece detecção física de plugue pelo aplicativo; a consulta da saída do Windows não muda seu roteamento. XLR sozinho não recebe os comandos USB.

Não há fontes remotas, CDN de ícones ou imagens hospedadas exigidas para abrir a interface. O app continua sem gravar voz ou processar o áudio no PC. A consulta de atualizações é a mesma conexão de rede manual da versão anterior.

## Verificação

Em 07/10/2026, passaram 42 testes Rust, 28 verificações da interface com USB simulado, 17 verificações de atualizações e o smoke de ganho dos fones. Estúdio, Voz, equalizador/Fones, Perfis e Capacidades foram conferidos em 1440×900 e 1100×720, com atualizações, estado desconectado e movimento reduzido, sem overflow horizontal ou erros JavaScript. Medidores mantêm transições de largura e posição desativadas. O executável portátil e o instalado passaram no smoke; instalação e desinstalação isoladas confirmaram a versão 0.6.7 e preservaram o diário. WebView2 já estava presente; o ramo sem WebView2 e o clique real do download não foram testados. Não houve nova escuta nem mudança real de parâmetros do AM8 nesta implementação. A prévia de Estúdio entregue é simulada e está identificada na imagem. As evidências auditivas dos efeitos e do EQ permanecem nos relatórios históricos; um redesign visual não valida novos valores acústicos.
