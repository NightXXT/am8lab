# Instalação e verificação

Windows 10/11 x64. Escolha o setup nas Releases, confira o hash no PowerShell:

```powershell
Get-FileHash -LiteralPath .\AM8-Lab-Setup-v0.6.9.exe -Algorithm SHA256
```

Compare com SHA256SUMS.txt da mesma Release. Um hash confere integridade em relação à lista; uma lista alterada junto com o arquivo não oferece autenticidade. Este lançamento é sem assinatura Authenticode.

Execute o setup para o usuário atual. Há seletor Português/English, destino, atalhos e desinstalação nas configurações do Windows. WebView2 é detectado; se ausente, seu instalador oficial precisa acessar a internet. Não instala driver próprio nem altera a saída padrão do Windows.

Depois conecte o AM8 USB. A checagem do aparelho acontece no aplicativo e revisões não reconhecidas são recusadas. XLR sozinho não permite controle.

Para usar o **EQ dos fones**, conecte os fones ao P2 do AM8 e selecione o dispositivo de reprodução do AM8 nas configurações de som do Windows. Jogos e aplicativos que escolhem uma saída própria também precisam usar esse dispositivo. Abra **Fones** na navegação ou selecione **Fones** no equalizador, prepare os controles e pressione **Aplicar**. O programa consulta a saída padrão, mas não a altera nem detecta fisicamente a inserção do plugue P2.

Antes de desinstalar, fechar normalmente e restaurar a sessão. O desinstalador não deve ser tratado como ferramenta de recuperação do DSP. Mantenha o diário se houver falha/desconexão. A versão portátil exige WebView2 já instalado.

## Atualizar uma instalação

Na parte inferior do aplicativo, abra **Atualizações → Verificar atualizações**. Havendo um setup mais novo, **Baixar instalador** abre o arquivo da Release de [NightXXT/am8lab](https://github.com/NightXXT/am8lab/releases) no navegador padrão. A consulta não exige o microfone conectado e só acessa a internet quando você pede.

Confira o hash da mesma Release, restaure a sessão e feche o AM8 Lab normalmente. Execute o novo setup para o mesmo usuário. O app não instala a atualização sozinho e não atualiza firmware. Os perfis locais são mantidos; carregar um perfil prepara os controles e Aplicar envia os ajustes.

Se não houver setup válido, use **Abrir Releases** para consultar a página. Enviar arquivos de código ao repositório não publica um instalador. Veja [os critérios da consulta](UPDATES.md).

## Navegação da interface 0.6.7

Estúdio mostra a sessão e seus caminhos de áudio. Voz reúne os módulos com Aplicar por efeito. Fones abre o equalizador do áudio reproduzido pelo AM8, com ganho geral separado. Perfis guarda controles localmente; carregar continua apenas preparando os valores. Capacidades mostra o alcance dos testes e os recursos experimentais.

Comparar com original e Restaurar sessão ficam no topo. Atualizações permanece na parte inferior da barra lateral, com consulta e download manuais. O redesign não altera a compatibilidade USB nem acrescenta efeitos, detecção física do P2 ou gravação de firmware.
