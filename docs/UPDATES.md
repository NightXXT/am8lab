# Atualizações manuais — 0.6.6

O botão Atualizações mostra a versão local e consulta Releases públicas de [NightXXT/am8lab](https://github.com/NightXXT/am8lab/releases) quando o usuário pressiona Verificar atualizações. Não há consulta automática ao abrir o app, instalação em segundo plano ou gravação de firmware. O AM8 não precisa estar conectado.

## Comportamento

- Uma versão mais nova com setup válido libera Baixar instalador; o botão abre esse arquivo no navegador padrão do Windows. A transferência e a execução do instalador ficam com o usuário.
- A mesma versão mostra Você está atualizado. Uma versão local superior mostra Versão local mais recente e não oferece downgrade.
- Releases vazias mostram que ainda não há lançamentos. Releases sem setup válido mostram indisponibilidade do instalador. Falhas de rede permitem tentar novamente.
- Abrir Releases usa sempre o endereço fixo do projeto. A interface não fornece URLs arbitrárias ao backend.
- Feche o aplicativo normalmente antes de executar outro setup, para restaurar os parâmetros da sessão. O diário de recuperação e os perfis não são enviados ao GitHub.

## Formato exigido ao publicar

Anexe um arquivo chamado exatamente `AM8-Lab-Setup-vX.Y.Z.exe` à Release; para esta entrega, `AM8-Lab-Setup-v0.6.6.exe`. Cada componente da versão tem somente algarismos, sem zeros à esquerda. O setup precisa estar completamente enviado, com tamanho entre 1 byte e 500 MiB. Mantenha SHA256SUMS.txt e as limitações conhecidas junto ao lançamento.

O aplicativo verifica até 30 Releases publicadas e escolhe a maior versão válida pelo nome do setup. Pré-releases publicadas entram na busca e aparecem identificadas como experimentais. O nome da tag não precisa ser uma versão, por isso a tag histórica Progams continua reconhecida quando contém um setup válido. Tags aceitas têm somente letras ASCII, números, ponto, hífen ou sublinhado e não podem ser um caminho.

O endereço da página e o download precisam pertencer ao mesmo repositório, à mesma tag e ao mesmo arquivo. Drafts, arquivos com outro nome, URLs externas e versões não canônicas são recusados. Commits novos ou um ZIP de fontes não são instaladores e não geram uma oferta de atualização.

## Rede e privacidade

A consulta usa WinHTTP e HTTPS para `api.github.com/repos/NightXXT/am8lab/releases?per_page=30`. Não há token de autenticação, envio de áudio, serial do AM8, perfis ou diário de recuperação. O User-Agent informa AM8Lab seguido da versão instalada; o GitHub recebe os dados comuns de uma requisição de rede, incluindo o endereço IP. Baixar ou abrir Releases usa o navegador padrão e as configurações desse navegador.

A resposta é limitada a 1 MiB e há um prazo de 15 segundos para a consulta. Existe apenas um trabalhador de rede por vez; um resultado atrasado não substitui o estado depois de um timeout. A consulta fica separada do mutex da sessão do microfone. Cookies, autenticação automática e redirecionamentos da API são desativados; a validação normal do certificado TLS é mantida.

Notas e títulos de Release são exibidos como texto, sem interpretar HTML. O backend oferece comandos próprios para consultar estado, verificar Releases e abrir apenas os destinos permitidos; não acrescenta uma API genérica de rede ou shell à interface. Os modos --smoke-test e --design-preview recusam acesso de rede e abertura de navegador.

O app não baixa nem verifica uma assinatura do binário. Os executáveis desta entrega continuam sem assinatura Authenticode; confira o SHA-256 publicado antes de instalar. A validação do endereço reduz destinos indevidos, mas não substitui a confiança na conta que publica as Releases.

## Verificação

07/10/2026: 42 testes Rust aprovados, incluindo 11 para seleção e validação das atualizações. Cobertura: nomes/versões, tag histórica, URLs, drafts e pré-releases, ordenação, ausência de installer, limites e cache. A interface passou 17 verificações com transporte simulado, incluindo notas tratadas como texto, falhas e estados de versão. O teste anterior da interface de ganho dos fones continuou aprovado.

Em 07/10/2026, a consulta real pelo módulo WinHTTP reconheceu o instalador 0.6.3 da tag histórica `Progams` e retornou `ahead` para o aplicativo local 0.6.6, sem oferecer downgrade. O resultado refere-se ao repositório no momento do teste. O executável portátil e o instalado passaram no smoke; instalação e desinstalação em pasta de teste preservaram o diário. WebView2 já estava presente; o ramo sem WebView2 e o clique real para abrir o download no navegador não foram executados. As URLs e o bloqueio de downloads indevidos foram verificados nos testes automatizados. As validações físicas dos efeitos e do ganho permanecem as descritas nos relatórios históricos; esta mudança não acrescenta avaliação auditiva.

## Interface 0.6.7

O painel de atualizações foi integrado à nova barra lateral. O fluxo, o destino e os critérios continuam os mesmos da versão 0.6.6. Para publicar a entrega 0.6.7, anexe exatamente AM8-Lab-Setup-v0.6.7.exe. Os resultados históricos da consulta acima continuam referentes à entrega 0.6.6 e ao momento indicado.
