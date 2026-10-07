# Verificação final

Versão0.6.3, 06/10/2026. 18 testes Rust aprovados na cópia de fontes pública. Smoke UI/ACL/perfis aprovado, inclusive app instalado. Instalador x64 NSIS para usuário atual com WebView2 bootstrapper oficial, idiomasPortuguês/English, licenças e desinstalador. Instalação/desinstalação testadas com WebView2 existente; diário intacto. Ramo de runtime ausente não testado. Fonte pública sem grafo bruto/DLL/firmware/seriais reais/logs privados, binário remapeado sem caminho pessoal de compilação.

cargo-audit: zero alertas classificados como vulnerabilities e dois avisos informativos no lockfile, fora do alvoWindows; detalhes em am8-lab/docs/SECURITY-REVIEW.md. Isso não garante ausência de vulnerabilidades. Reverb experimental após relato de estalos na reprodução.

Lista SHA-256 em Downloads-Windows/SHA256SUMS.txt. Código e instalador não enviados ao GitHub. Upload pelo usuário via navegador conforme COMO-PUBLICAR.md. Actions ainda não executadas no GitHub.
