# Contribuir

Compile e rode `cargo test --locked --lib` e o executável com `--smoke-test`. Rode cargo audit com base atualizada. Descreva escopo e validação no PR.

Não acrescente escrita de firmware, comandos genéricos arbitrários na IPC, gravações de voz ou acesso de shell/rede sem projeto e revisão próprios. Parâmetros/canais não conhecidos devem permanecer bloqueados. Não enfraqueça a verificação do grafo/firmware para aceitar novas revisões sem testes e protocolo compreendido.

Testes físicos precisam identificar revisão e avaliar escrita, leitura, restauração e resultado audível separadamente. Não confirme redução de microfonia só por ausência de cortes. Não publique seriais, dumps brutos ou DLLs/firmware do fabricante. Utilize fixtures fictícias e limites conservadores.
