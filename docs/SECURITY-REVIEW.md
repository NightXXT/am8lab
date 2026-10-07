# Revisão de segurança — 06/10/2026 — versão 0.6.4

## Escopo

A revisão inicial da versão 0.6.3 cobriu localmente backend Rust, UI, permissões Tauri, fronteiras de entrada, restauração e arquivos de distribuição, com consulta cargo-audit 0.22.2 ao RustSec. A versão 0.6.4 acrescenta o EQ dos fones com os mesmos limites de acesso e recuperação, testes automatizados adicionais e um teste físico temporário/restaurável do novo caminho.

Não é pentest independente, fuzzing amplo ou prova de ausência de vulnerabilidades. Não foram executados testes de ataque contra Windows, Discord, firmware ou rede. A consulta ao RustSec abaixo é a revisão da versão 0.6.3; as versões das dependências foram mantidas e a auditoria não foi repetida para 0.6.4.

## Problemas corrigidos

| Item | Classificação | Correção |
|---|---|---|
| Recuperação pendente sem serial poderia ser usada em outro AM8 compatível | Integridade local / confiabilidade média | Exigir serial válido em journals não vazios, versões1–3; recusar mismatch antes de escrever; preservar arquivo |
| Limite64KiB era conferido depois de ler o arquivo inteiro | Negação de serviço local, baixa | Leitura limitada a64KiB+1 antes do parse |
| Perfis null/malformados podiam impedir inicialização | Disponibilidade local, baixa | Validação estrita, descarte de entradas inválidas, cap20 e nome até128 |
| Permissões core:default desnecessárias | Redução de superfície | Somente listen/unlisten e allowlist de comandos próprios via AppManifest |
| API Rust query(0x80,[]) podia indexar payload vazio | Confiabilidade local | Recusa antes do envio |
| Escrita de palavra no SDK permitia índices fora da descrição do bloco | Integridade local | Validar seletor, enable e faixa antes do envio |
| Duas sessões Windows poderiam controlar o aparelho | Confiabilidade | Mutex Global, mantendo Local para versões anteriores na mesma sessão |
| Dados extraídos e informações pessoais em arquivos de investigação | Distribuição / privacidade | Somente schema/fixture reduzidos e fingerprint; dumps, serial real, logs e backups excluídos; caminhos de compilação remapeados |

Nomes de perfil/erros/saída são tratados como texto. HTML dinâmico recebe valores validados; CSP bloqueia scripts remotos/inline, frames, objetos, base e envio de formulários. Sem plugins de shell, FS, HTTP ou abertura remota na UI. Esses controles não protegem contra malware com privilégios do usuário ou modificações arbitrárias do próprio processo.

## Dependências

`cargo audit --no-yanked --json`: **0 advisories classificados como vulnerabilities**, com dois avisos informativos no lockfile. Database commit `ef6173cbc5c50ec8166f9a5b28f07834144373ee`, atualização 03/10/2026. A checagem de crates yanked foi desativada nesta execução; não é uma checagem de malware nem de toda a cadeia de fornecimento.

- [RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370.html): proc-macro-error1.0.4 sem manutenção.
- [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html): glib0.18.5 tem unsoundness no VariantStrIter; corrigido nas versões>=0.20.

`cargo tree --target x86_64-pc-windows-msvc -i glib` e `-i proc-macro-error` não encontram dependentes. Ambos aparecem no lockfile por dependências de outros alvos e não entram na compilação Windows deste projeto. Não suprimi os avisos no relatório ou no CI. Um port Linux exigirá revisão própria desses avisos.

Arquivo bruto da auditoria: [rustsec-audit.json](rustsec-audit.json). Catálogo/versões podem mudar; repetir a revisão antes de novos lançamentos.

## Validação e limites

**24 testes Rust passaram** na versão 0.6.4. Os 18 testes existentes cobrem limites, dados de recuperação malformados, arquivos grandes, serial, falhas, desconexão, rollback, reinício, comparação e rotas. Seis novos testes cobrem o EQ dos fones: limites, isolamento do EQ da voz e das rotas, falha de comunicação/restauração, recuperação após reiniciar, vínculo com serial e ambos os EQs ativos com originais independentes.

O smoke da interface com transporte simulado passou, incluindo os destinos separados e os perfis. A validação auditiva inicial do EQ dos fones cobriu um passa-baixas de 2.500 Hz em uma única unidade por 60 segundos; as leituras, comparação/retomada e restauração exata foram verificadas. Uma conferência adicional no aparelho cobriu os dois EQs individualmente e ativos ao mesmo tempo, comparação/retomada e restauração de todos os blocos, sem erros. A conferência combinada não equivale a uma avaliação auditiva de combinações. Veja [o escopo do teste físico](HEADPHONE-EQ.md).

A verificação do fingerprint usa SHA-256 de2931bytes e não embute o grafo original. O firmware/layout USB ainda são restringidos aB5 0.7.1. O app salva o journal antes de toda mudança e tenta confirmar/voltar ao estado anterior quando a escrita falha.

Permanece limitação de hardware: reverb pode provocar estalos na reprodução; sua causa não foi determinada e o uso recomendado é sem reverb. A supressão de microfonia não teve eficácia acústica medida. Testes históricos em um único aparelho não comprovam qualidade em todas as combinações ou unidades. Binários não são assinados; o SHA-256 não substitui assinatura ou avaliação de confiança.

Validação de entrega, **versão 0.6.4**: setup instalado em pasta de teste com WebView2 já presente; somente app, licenças e desinstalador instalados. O smoke Tauri do portátil e do app instalado passou. Os executáveis diferem apenas em três bytes do marcador de tipo de pacote do Tauri (UNK/NSS); após normalizar esse marcador, são idênticos. Desinstalação terminou e o diário permaneceu intacto. O ramo WebView2 ausente não foi executado nesta máquina. Workflow GitHub ainda não executado.

## Atualização 0.6.5 — 07/10/2026

Acrescentado ganho geral nativo dos fones de 0 a +18 dB, com validação de inteiro/faixa apenas no EQ de reprodução. +20 dB e campos indevidos são recusados antes das escritas. Perfis antigos continuam aceitos sem habilitar ganho novo automaticamente. Os 31 testes Rust passaram; sete são novos para ganho, compensação, reaplicação e recuperação. Verificação física de +1 dB confirmou leitura, comparação, retomada e restauração completa. Ganhos altos não foram aplicados nem avaliados audivelmente; positivo pode distorcer e não existe limitador automático. Veja [o escopo](HEADPHONE-GAIN.md).

As dependências continuam com as mesmas versões da revisão 0.6.3. A auditoria RustSec herdada não foi repetida nesta alteração; não é uma afirmação de ausência de vulnerabilidades atuais. As permissões e a lista de comandos Tauri permaneceram iguais. A entrega 0.6.5 recebeu smoke do executável Tauri e teste de instalação/desinstalação com WebView2 já existente, preservando o diário.

## Atualização 0.6.6 — 07/10/2026

Adicionados quatro comandos próprios tipados para estado, consulta manual e abertura de Releases/installer. A lista de comandos Tauri e suas permissões foi ampliada especificamente para eles, sem plugins genéricos de rede, shell ou arquivos. No Windows, novos recursos da dependência windows habilitam WinHTTP e abertura do navegador; versões das dependências não foram alteradas.

A API de consulta é fixa em HTTPS; resposta limitada a 1 MiB, prazo de 15 segundos, um trabalhador por vez e cache que não recebe resultados atrasados. O instalador precisa ter nome/versão canônicos e URL exata do repositório/tag/arquivo validado. Download significa abrir o navegador; não há autoexecução, autenticação, atualização de firmware ou envio de dados do AM8. Títulos e notas são exibidos como texto. Smoke e preview bloqueiam rede e navegador.

42 testes Rust e 17 verificações simuladas da interface aprovados. A revisão RustSec de 0.6.3 foi herdada, sem nova consulta, e não representa uma garantia de ausência de vulnerabilidades atuais. O vínculo com uma Release não é assinatura do arquivo; os executáveis continuam sem Authenticode. O resumo da verificação real da entrega acompanha o pacote. Veja [a implementação e os limites](UPDATES.md).

## Atualização 0.6.7 — 07/10/2026

Redesign em HTML/CSS/JavaScript, mantendo o backend de DSP, a restauração e os comandos tipados de atualizações. No Rust, a mudança funcional desta entrega é a versão do aplicativo; permissões, lista de comandos e versões das dependências foram preservadas. SVGs e estilos são locais; fontes usam recursos locais/do sistema, sem importar pedidos de fonte/CDN do export de design.

Títulos, notas de Release e nomes de perfil continuam tratados como texto nos fluxos já revisados. A estética não é usada para apresentar números simulados como medidas acústicas. O limite de compatibilidade e a política de restauração permanecem iguais. Os testes finais do executável e da interface acompanham a entrega; não houve nova auditoria RustSec nem avaliação auditiva nesta alteração.
