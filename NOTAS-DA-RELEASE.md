Versão experimental para Windows x64 e AM8 USB B5 0.7.1.

Inclui setup para usuário atual com verificação WebView2, versão portátil, interface em português, DSP nativo, EQ de dez filtros, ruído, compressor, pitch/transformação e medidores reais.

Segurança: recuperação pendente exige serial válido; arquivos/perfis malformados são limitados e recusados; permissões da UI foram reduzidas; comandos Rust têm ACL explícita e validação; grafo bruto substituído por fingerprint. Revisão e avisos em docs/SECURITY-REVIEW.md.

Limitações: somente um aparelho/revisão validado; outras revisões recusadas. Reverb pode causar estalos na reprodução — mantenha desligado para uso natural. Supressor de microfonia experimental. Sem RGB, afinação automática ou flash de firmware. Binários sem assinatura Authenticode. Testes automáticos usam hardware falso; as confirmações USB históricas não garantem todo valor/combinação ou toda revisão.

Validação de entrega: setup final instalado em pasta de teste com WebView2 já presente; somente app, licenças e desinstalador instalados. Smoke do app instalado passou; hash coincide com o portátil. Desinstalação terminou e o diário original permaneceu intacto. O ramo WebView2 ausente não foi executado nesta máquina. Workflow GitHub ainda não executado.
