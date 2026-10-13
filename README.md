# feather-ui

Biblioteca de interface compartilhada da suíte **FeatherOffice**: tema, cores,
ícones, marca, faixa de opções (Ribbon), barra de janela, ajuda, configurações
e os componentes visuais comuns a todos os aplicativos — FeatherSheet,
FeatherDocs, FeatherSlide, FeatherCAD e FeatherPDF.

É uma **biblioteca**, não um aplicativo: cada programa da suíte a compila
dentro do próprio executável. Quem instala só a planilha não precisa instalar
`feather-ui` à parte, nem DLL ou `.so`.

## Por que ela existe

Os aplicativos são escritos em Rust com [Iced](https://iced.rs). O Iced dá os
blocos básicos (botão, texto, rolagem); ele não diz como uma ferramenta do
FeatherOffice deve parecer nem se comportar. Sem uma peça comum, cada
aplicativo reinventaria a própria faixa de opções, o próprio tema escuro e os
próprios ícones — e eles iriam se afastando um do outro a cada mudança.

A `feather-ui` existe para três coisas:

1. **Integrar as ferramentas.** A mesma barra de janela, a mesma Ribbon, a
   mesma ajuda (F1) e as mesmas configurações em todos os aplicativos. Quem
   aprende um, já sabe usar os outros.
2. **Garantir o padrão de qualidade.** As regras visuais estão no código e
   nos testes, não na boa vontade de cada tela: ícone só do catálogo, cor só
   por token do tema, nenhum emoji, nenhum botão sem ação. Ver
   [`GUIA-JANELAS.md`](GUIA-JANELAS.md).
3. **Personalizar tudo num lugar só.** Tema claro/escuro, cor de destaque,
   ícones e marca mudam aqui e valem para a suíte inteira, sem tocar em cada
   aplicativo.

Por isso ela **não conhece nenhum motor**: não sabe o que é uma planilha, um
documento ou um slide. Regras de documento, cálculo, arquivos e
desfazer/refazer ficam nos motores de cada aplicativo; aqui fica só o desenho.

## Decisões de projeto

- **Ícones embutidos no executável.** O catálogo (`dados/tabler-acoes.tsv`,
  no estilo [Tabler Icons](https://tabler.io/icons)) entra com `include_str!`
  e vira SVG em memória. Nada é baixado nem lido do disco em tempo de
  execução: funciona sem rede e sai igual no Linux e no Windows.
- **Cores como funções, não constantes** (`COR_PAINEL()`, `COR_TEXTO()`…):
  o tema e a cor de destaque mudam com o programa aberto.
- **Marca vetorial** (`assets/logos/`): duas penas em caminhos SVG, uma cor
  de fábrica por aplicativo (FeatherSheet verde, FeatherDocs azul,
  FeatherSlide amarelo), que acompanha a cor de destaque quando a pessoa
  escolhe uma.
- **Sem dependências além do Iced**, para compilar rápido e não arrastar
  bibliotecas em C para os aplicativos.
- **Privacidade**: nenhum componente faz acesso à rede nem telemetria.

## O que tem

| Módulo | Para quê |
|---|---|
| `tema` | paleta clara/escura, cores de destaque, estilos de botão, painel e rolagem |
| `icones` | catálogo de ícones SVG embutido |
| `marca` | penas do FeatherOffice e a cor de cada aplicativo |
| `cromo` | barra de título própria (arrastar, minimizar, maximizar, fechar) |
| `janelas` | abrir e fechar janelas auxiliares, uma por função |
| `ribbon`, `componentes` | faixa de opções: abas, grupos, botões grandes e pequenos |
| `dropdown`, `seletor_cor`, `formatacao`, `alinhamento` | menus suspensos, cores, fonte e alinhamento |
| `ajuda` | janela de ajuda F1 pesquisável, a mesma em todos os aplicativos |
| `configuracoes` | tela de configurações (lateral + cartões) |
| `atalhos`, `acessibilidade` | atalhos de teclado e apoio à acessibilidade |
| `efeitos_imagem` | painel de ajustes rápidos de imagem |
| `creditos` | bloco "Colaboradores" do Sobre, igual em todos os aplicativos |

## Como usar

No `Cargo.toml` do aplicativo:

```toml
[dependencies]
feather-ui = { git = "https://github.com/FeatherOffice/feather-ui", tag = "v0.2.0" }
```

Para desenvolver a biblioteca e o aplicativo lado a lado, aponte para a cópia
local sem mudar a dependência acima:

```toml
[patch."https://github.com/FeatherOffice/feather-ui"]
feather-ui = { path = "../feather-ui" }
```

No código:

```rust
use feather_ui::tema::{definir_tema, COR_PAINEL};
use feather_ui::componentes::{botao_grande, botao_pequeno, grupo_faixa};
use feather_ui::icones::icone;
```

## Desenvolvimento

```bash
cargo test
cargo clippy --all-targets
```

Antes de construir qualquer janela, tela ou diálogo de um aplicativo da suíte,
leia o [`GUIA-JANELAS.md`](GUIA-JANELAS.md).

Ao acrescentar um ícone ao catálogo, o teste
`representacao_svg_tem_hash_estavel_em_linux_e_windows` falha de propósito:
confira o desenho e atualize o valor de referência no mesmo commit.

## Licença

GPL-3.0 (ver [`LICENSE`](LICENSE)). Os ícones no estilo Tabler seguem a
licença MIT em [`dados/TABLER-LICENSE.txt`](dados/TABLER-LICENSE.txt).
