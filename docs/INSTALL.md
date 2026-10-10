# Instalação e verificação

Windows 10/11 x64. Escolha o setup nas Releases, confira o hash no PowerShell:

```powershell
Get-FileHash -LiteralPath .\AM8-Lab-Setup-v0.7.2.exe -Algorithm SHA256
```

Compare com SHA256SUMS.txt da mesma Release. Um hash confere integridade em relação à lista; uma lista alterada junto com o arquivo não oferece autenticidade. Este lançamento é sem assinatura Authenticode.

Execute o setup para o usuário atual. Há seletor Português/English, destino, atalhos e desinstalação nas configurações do Windows. WebView2 é detectado; se ausente, seu instalador oficial precisa acessar a internet. Não instala driver próprio nem altera a saída padrão do Windows.

Depois conecte o AM8 USB. A checagem do aparelho acontece no aplicativo e revisões não reconhecidas são recusadas. XLR sozinho não permite controle.

Para usar o **EQ dos fones**, conecte os fones ao P2 do AM8 e selecione o dispositivo de reprodução do AM8 nas configurações de som do Windows. Jogos e aplicativos que escolhem uma saída própria também precisam usar esse dispositivo. Abra **Fones** na navegação ou selecione **Fones** no equalizador, prepare os controles e pressione **Aplicar**. O programa consulta a saída padrão, mas não a altera nem detecta fisicamente a inserção do plugue P2.

Antes de desinstalar, fechar normalmente e restaurar a sessão. O desinstalador não deve ser tratado como ferramenta de recuperação do DSP. Mantenha o diário se houver falha/desconexão. A versão portátil exige WebView2 já instalado.

Na 0.7.0, **Fones** também oferece **Mono/Estéreo**, com seu próprio botão Aplicar. **Comparar** alterna entre o modo aplicado e o original; **Restaurar** e o fechamento normal recuperam o modo salvo. Perfis locais continuam guardando os controles de efeitos/EQ; o modo dos fones não é salvo nos perfis nesta versão. Um diário que inclua o modo usa versão 4 e deve ser recuperado pela versão nova antes de voltar a um aplicativo antigo. Veja [modos e informações](NATIVE-MODES.md).

## Atualizar uma instalação

Na 0.7.2 ou posterior, use **Atualizações → Verificar atualizações → Baixar atualização → Instalar e reiniciar**. O download é feito no app e a assinatura verificada antes de liberar a instalação. O app restaura a sessão antes de iniciar o setup; se a restauração falhar, reconecte o microfone e tente novamente. Salve os controles em um perfil para reaplicar depois. A atualização não grava firmware nem envia o diário.

Versões até 0.7.1 precisam instalar a 0.7.2 uma vez pelo setup. Feche a versão antiga normalmente antes de executar esse setup. O pacote portátil também usa o instalador Windows para atualizar; não substitui apenas o executável portátil. Veja [detalhes e critérios](UPDATES.md).

## Navegação da interface 0.6.7

Estúdio mostra a sessão e seus caminhos de áudio. Voz reúne os módulos com Aplicar por efeito. Fones abre o equalizador do áudio reproduzido pelo AM8, com ganho geral separado. Perfis guarda controles localmente; carregar continua apenas preparando os valores. Capacidades mostra o alcance dos testes e os recursos experimentais.

Comparar com original e Restaurar sessão ficam no topo. Atualizações permanece na parte inferior da barra lateral, com consulta e download manuais. O redesign não altera a compatibilidade USB nem acrescenta efeitos, detecção física do P2 ou gravação de firmware.

A versão 0.7.2 aceita B5 0.7.3 de forma experimental, com identidade e fluxo exatos; os testes físicos de efeitos nessa revisão ainda estão pendentes. Consulte [compatibilidade e limites](FIRMWARE-COMPATIBILITY.md). Feche normalmente para restaurar antes de executar o novo setup.
