# Instalador do Grafeno Vetor

Este arquivo contém o instalador Windows do **Grafeno Vetor** (versão 0.21.0).

## O que é o Grafeno Vetor?

O **Grafeno Vetor** (também conhecido como TraceDraw) é um editor vetorial profissional e de código aberto, escrito em Rust. Ele permite criar ilustrações vetoriais de alta qualidade para impressão digital, design gráfico, diagramação, ilustrações artísticas, logos, ícones e muito mais.

### Principais recursos:
- Desenho vetorial profissional (curvas Bézier, formas geométricas, textos)
- Suporte a múltiplas páginas e camadas
- Arquivos .cdr do CorelDraw (versões 1 a 27)
- Exportação para PDF/X (impressão profissional)
- Ilustração com efeitos, gradientes e texturas
- Interface em 18 idiomas

## Como instalar

1. Execute `grafeno-vetor-0.21.0-windows-x86_64-setup.exe`
2. Siga as instruções na tela
3. O programa será instalado em `Program Files\GrafenoVetor\`
4. Um ícone será criado no menu Iniciar

## Requisitos do sistema

- Windows 10 ou superior (64-bit)
- Placa gráfica com suporte a DirectX 11+
- Pelo menos 4 GB de RAM

## Arquivos no pacote

| Arquivo | Descrição |
|---|---|
| `grafeno-vetor-0.21.0-windows-x86_64-setup.exe` | Instalador Windows (Inno Setup) |
| `grafeno.exe` | Aplicação principal do Grafeno Vetor |
| `grafeno-cli.exe` | Linha de comando para automação |

## Links úteis

- Repositório: https://github.com/josue2016-del/grafeno-vetor
- Página inicial: https://igtoth.github.io/grafeno/

## Status do projeto

O Grafeno Vetor está em **alpha**. Isso significa que:
- A maioria dos recursos funciona bem
- Podem existir bugs e erros inesperados
- O formato nativo (.tdraw) pode mudar entre versões
- Arquivos importados podem não abrir perfeitamente

## Licença

O Grafeno Vetor é disponibilizado sob a licença **MIT** ou **Apache-2.0**. Você é livre para usar, modificar e distribuir o software, desde que mantenha os avisos de copyright.