# Política de segurança

Este projeto experimental controla hardware USB. A revisão inicial não equivale a uma auditoria independente completa nem garante ausência de vulnerabilidades.

Não usa APIs de shell, arquivos ou rede pela UI; permite somente comandos próprios tipados, validados no Rust, e escuta de eventos. O app não altera firmware, não envia gravações e não inclui telemetria de rede. O instalador pode precisar de internet para WebView2.

Veja [docs/SECURITY-REVIEW.md](docs/SECURITY-REVIEW.md) para escopo, correções e avisos das dependências. Use apenas a versão mais recente disponível e respeite a lista de firmware validada.

Após criar o repositório, o mantenedor deve habilitar **Private vulnerability reporting** nas configurações de segurança do GitHub. Use a aba Security para relato privado quando disponível. Até lá, não publique segredos, dados pessoais nem exploração detalhada em Issues públicas; solicite um canal privado ao mantenedor. Não há canal privado configurado neste pacote local.

Para relatos de uso, remova serial, nomes de dispositivos/caminhos pessoais e diários de recuperação. A recuperação foi projetada para o mesmo aparelho: não copie um diário para outro AM8. Falhas de USB preservam o diário; não o apague para contornar a proteção.
