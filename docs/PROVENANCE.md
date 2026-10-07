# Origem e distribuição

Rust, controles e lógica de recuperação são código independente deste projeto. A UI adapta o design Stitch fornecido pelo usuário. `ui/styles.css` é gerado com Tailwind CSS 3.4.17, sob MIT.

`effect-schema.json` é uma seleção de nomes, limites e unidades de parâmetros consultados na biblioteca do próprio aparelho; `test-baseline.json` contém valores numéricos reduzidos usados em testes falsos. São metadados/fatos de interoperabilidade, não código executável da biblioteca. Eles não constituem autorização para redistribuir firmware ou arquivos do fabricante.

O fluxo conhecido é verificado apenas por comprimento (2931 bytes) e fingerprint SHA-256 `bd9dbe74994cbb96cb2981c6bb340aae5b894409cf290378c6e0201ffc2f131d`. O grafo bruto, firmware e DLLs extraídas foram excluídos. Seriais de teste são fictícios. Relatórios brutos, backups e caminhos pessoais não estão incluídos.

O EQ dos fones usa o bloco `eq1` (`0x8D`) identificado no caminho de reprodução conhecido, antes da mistura com a voz do microfone. O EQ da voz usa outro bloco, `eq7` (`0x99`). A identificação foi conferida contra os metadados de parâmetros e validada em um teste audível com restauração; [o registro público resumido](HEADPHONE-EQ.md) não inclui logs brutos nem informações pessoais.

Licença MIT cobre o código original. Dependências mantêm suas licenças próprias e a marca FIFINE continua pertencendo a seus titulares.
