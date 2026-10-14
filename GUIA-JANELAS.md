# Guia de janelas do FeatherOffice

Como montar uma janela (ou tela, ou diálogo) Iced na suíte sem que ela saia
feia ou fora do padrão. Vale para FeatherSheet, FeatherDocs, FeatherSlide,
FeatherCAD e FeatherPDF. Decidido com o usuário em 13/10/2026.

**As duas referências oficiais**, que se copia antes de inventar:

| Referência | Onde | Serve de modelo para |
|---|---|---|
| **Ajuda F1 do FeatherSheet** | `feather-ui/src/ajuda/visual.rs` (conteúdo em `feathersheet-gui/src/app/ajuda.rs`) | toda janela de **consulta e navegação**: ajuda, galerias, catálogos, configurações, assistentes |
| **Editor de macros Alt+F11 do FeatherSheet** | `feathersheet-gui/src/app/macros/editor_visual.rs` (+ `estilos.rs`, `barra_menu.rs`, `console.rs`) | toda janela de **ferramenta / trabalho**: editores, OpenQuery, depurador, monitor |

A F1 é **o padrão de janela de ajuda** de toda a suíte: a ajuda de outro
aplicativo usa o mesmo componente (`feather_ui::ajuda::Catalogo::view`) e só
troca o conteúdo.

---

## 1. Regras que não se discutem

1. **Nunca emoji.** Nem em botão, rótulo, mensagem, título, aviso ou texto de
   ajuda. Emoji depende da fonte do sistema, sai colorido no meio de uma
   interface monocromática e muda de desenho entre Linux e Windows.
   Caracteres que fazem papel de ícone (`⌕`, `✓`, `✗`, `★`, `⚠`, `➜`) também
   não: são emoji disfarçado. Use **ícone**.
2. **Todo ícone é SVG do catálogo** (`feather_ui::icones::icone(nome, tamanho)`),
   no estilo Tabler: traço, sem preenchimento, cor do tema. Ordem para
   conseguir um ícone (detalhes na seção 6):
   1. o catálogo já tem (`feather-ui/dados/tabler-acoes.tsv`);
   2. o Tabler Icons tem: buscar em <https://tabler.io/icons> e acrescentar ao catálogo;
   3. nenhum serve: **desenhar um** no mesmo estilo (24 × 24, traço 2, pontas redondas).
3. **Cor só por token** de `feather_ui::tema` (`COR_PAINEL()`, `COR_TEXTO()`…).
   Nada de `Color::from_rgb8(…)` solto numa tela: quebra o tema escuro e a cor
   de destaque escolhida pelo usuário. As únicas cores fixas aceitas são as de
   **conteúdo** (a cor de um modelo de planilha, de uma série de gráfico).
4. **Nenhum comando morto.** Botão sem ação não recebe `on_press` (fica
   desabilitado) e ganha dica dizendo o que fará; não abre janela vazia nem
   "em construção".
5. **Texto em pt-BR**, frases curtas, sem ponto final em rótulo de botão.
6. **Antes de construir, descrever** (regra 21 do FeatherSheet): qual das duas
   referências segue, o que vai em cada área, e o usuário confirma. Um esboço
   ASCII da tela ajuda.

---

## 2. Qual modelo usar

| Vai construir… | Modelo | Peças prontas |
|---|---|---|
| Ajuda de um app | **F1** sem mudar nada | `feather_ui::ajuda` (`Catalogo`, `Estado`, `Evento`) |
| Configurações, preferências | **F1** (lateral + cartões) | `feather_ui::configuracoes::{tela, menu, linha}` |
| Galeria / catálogo / "inserir…" com muitos itens | **F1**: lateral com grupos, busca em cima, grade de cartões | cartão da F1 (`visual.rs`, `cartao`) |
| Assistente em passos | **F1**: lateral com os passos, Anterior/Próximo embaixo | `paginacao` da F1 |
| Editor, IDE, consulta de dados, depurador | **Alt+F11** | `editor_visual.rs` (áreas, abas, console, rodapé) |
| Diálogo pequeno (confirmar, renomear, um formulário curto) | seção 5 deste guia | `estilo_botao_primario/secundario` |
| Gerenciador: lista de itens + formulário do item | **Formatação condicional** do FeatherSheet: lista à esquerda (prévia, descrição, ações por ícone), formulário à direita, ação primária embaixo à direita | `feathersheet-gui/src/app/condicional_ui.rs` (`view_condicional`) |
| Tela inicial do app | tela inicial do FeatherSheet/FeatherSlide | `menu_principal.rs` (Sheet) |

Toda janela auxiliar (que não é a principal) é aberta por
`feather_ui::janelas::Janelas::alternar` (**uma janela por função**: apertar
de novo fecha) e vestida com `feather_ui::cromo::janela_com_barra` (barra de
título própria de 42 px com arrastar, minimizar, maximizar e fechar).

---

## 3. Anatomia da F1 (janela de consulta)

```
┌──────────────────────────────────────────────────────────────────────┐
│ Ajuda do FeatherSheet                                    ─  □  ✕     │  barra 42 px (cromo)
├────────────────┬─────────────────────────────────────────────────────┤
│ [ic] Início    │ ┌─────────────────────────────────────────────────┐ │
│                │ │ [search]  Buscar na ajuda              Ctrl + F │ │  busca: painel arredondado
│ BÁSICO         │ └─────────────────────────────────────────────────┘ │
│ [ic] Começar   │                                                     │
│ [ic] Abas      │  [marca 86]  Título 24 semibold      [ilustração]  │  hero
│ [ic] Fórmulas  │              introdução 13, apagada                │
│ AVANÇADO       │                                                     │
│ [ic] Dados     │  Tópicos principais  (18 semibold)                 │
│ [ic] Macros    │  ┌──────────┐ ┌──────────┐ ┌──────────┐            │  grade de cartões, 3 colunas
│ OUTROS         │  │[ic] Título│ │[ic] Título│ │[ic] Título│ [>]       │  altura 118, raio 11
│ [ic] Arquivos  │  │ resumo    │ │ resumo    │ │ resumo    │           │
│                │  └──────────┘ └──────────┘ └──────────┘            │
│                │  ───────────────────────────────────────           │
│  lateral       │  Guias rápidos   [ic] Criar  [ic] Abrir  …          │
│  255 px        │                                                     │
│                │  [ Anterior ]                        [ Próximo ]   │  paginação
└────────────────┴─────────────────────────────────────────────────────┘
```

O que faz a F1 ficar bonita, e deve ser repetido:

- **Duas superfícies e só duas**: a lateral em `COR_PAINEL_FAIXA()` com borda
  fina `COR_BORDA_SUAVE()`, e a área de leitura em `COR_PAINEL()`. Nada de
  caixa dentro de caixa dentro de caixa.
- **Lateral de 255 px**, rolável, com **rótulos de grupo** em caixa alta, 10,5 pt,
  `COR_TEXTO_MUTED()` (`BÁSICO`, `AVANÇADO`, `OUTROS`), e itens com ícone 18 +
  texto 12,5, padding `[10, 13]`, raio 9. Item ativo: fundo `COR_AZUL_FUNDO()`;
  passar o mouse: `COR_HOVER()`.
- **Busca no topo da área de conteúdo**, nunca na lateral: painel com raio 12,
  ícone `search` 18 à esquerda, campo sem borda própria e a **dica do atalho**
  (`Ctrl + F`) numa etiqueta à direita. A busca ignora acentos
  (`feather_ui::ajuda::normalizar`).
- **Hierarquia de texto em 4 degraus** (seção 7) e muito respiro: `spacing(17)`
  entre blocos, `12` entre cartões.
- **Cartão = ícone num quadrado pastel + título + resumo**: o ícone 25 fica
  num `container` com padding 13, fundo `COR_AZUL_FUNDO()` e raio 12. Título
  13,5 semibold, resumo 11,5 apagado cortado em ~84 caracteres com `…`.
  Mesma altura para todos (118 px); a última linha da grade é completada com
  espaços vazios para os cartões não esticarem.
- **Paginação** no pé: Anterior (secundário) à esquerda, Próximo (primário) à
  direita; some durante a busca.
- **Resultado vazio** tem título e uma frase de orientação, nunca uma área em branco.

---

## 4. Anatomia do Alt+F11 (janela de ferramenta)

```
┌──────────────────────────────────────────────────────────────────────┐
│ Arquivo  Editar  Exibir  Inserir  Executar  Depurar  Ferramentas  Ajuda│  barra de menus (popup flutuante)
│ [ic][ic][ic] │ [ic][ic] │ [ic] Executar            [ic][ic][ic]      │  barra de ferramentas: grupos com divisor
├──────────────┬───────────────────────────────────────────────────────┤
│ Explorador   │ [ic] Módulo1 ×  [ic] Planilha1  [ic] Formulário  [+]   │  abas dos documentos
│ ─ [ic] Pasta │ ┌───────────────────────────────────────────────────┐ │
│   [ic] Plan1 │ │ (Geral)                           (Declarações)   │ │  cabeçalho da área
│ ─ Módulos    │ │                                                   │ │
│   [ic] Mod1  │ │   editor de código (monoespaçada 13, realce)      │ │
├──────────────┤ │                                                   │ │
│ Propriedades │ ├───────────────────────────────────────────────────┤ │
│ (Nome) Mod1  │ │ JAVASCRIPT · MÓDULO          [Guardar] [Executar] │ │  barra de ações da área
│ Tipo   …     │ └───────────────────────────────────────────────────┘ │
│  280 px      │                                                       │
├──────────────┴───────────────────────────────────────────────────────┤
│ Console / Imediato / Inspeção                                        │  painel inferior (opcional)
├──────────────────────────────────────────────────────────────────────┤
│ Pronto · Ln 12, Col 4 · JavaScript                                   │  rodapé de status
└──────────────────────────────────────────────────────────────────────┘
```

O que copiar:

- **Ordem fixa de cima para baixo**: menus → ferramentas → corpo (laterais +
  centro) → painel inferior → rodapé. O usuário encontra tudo onde espera.
- **Cada área é um painel arredondado** (`estilo_area`: fundo, borda 1, raio 9)
  com uma **faixa de cabeçalho** (`estilo_cabecalho`: fundo um tom abaixo,
  raio 6) que diz o que a área é. Áreas separadas por `spacing(7–8)` sobre o
  fundo da janela, não por linhas desenhadas.
- **Laterais com largura fixa** (280 px), que podem ser ligadas e desligadas
  pelo menu Exibir; o centro fica com `Length::Fill`.
- **Abas dos documentos** em cima do centro, com ícone 15 + nome 12 e um `+`
  no fim; rolagem horizontal quando não cabem.
- **Ações da área no pé da própria área** (Guardar, Executar), alinhadas à
  direita, com a ação principal por último e em estilo primário.
- **Menus suspensos flutuam** (`stack![base, fechar_fora, popup]` com
  `iced::widget::opaque`): não empurram o layout; clicar fora fecha.
- Comando ainda sem motor aparece **sem `on_press`** (desabilitado), nunca
  com uma ação de fachada.

**Não copiar** do Alt+F11 a paleta própria em `estilos.rs` (`cor_fundo`,
`cor_borda`… com RGB fixo): é herança da migração do TypeScript. Janela
nova usa os tokens de `feather_ui::tema` (regra 3).

---

## 5. Onde fica cada coisa

| Elemento | Lugar | Como |
|---|---|---|
| Título da janela | barra de 42 px | `cromo::janela_com_barra(titulo, corpo, acoes)` |
| Fechar | controles da barra (X vermelho ao passar o mouse) | não repetir um botão "Fechar" no corpo, salvo em diálogo |
| Navegação entre seções | lateral **esquerda** | 240–280 px, rótulos de grupo, item ativo com `COR_AZUL_FUNDO()` |
| Propriedades / detalhes do item | lateral (direita em editor de documento; esquerda abaixo do explorador em IDE) | tabela nome / valor, nome em cinza |
| Busca | topo da área de conteúdo | ícone `search` + campo + dica do atalho |
| Conteúdo principal | centro, `Length::Fill` | dentro de `feather_ui::tema::rolagem` quando puder passar da altura |
| Ação principal | canto **inferior direito** (diálogo) ou **superior direito** (tela) | `estilo_botao_primario`, uma só por tela |
| Ações secundárias | à esquerda da principal | `estilo_botao_secundario` |
| Ação destrutiva | separada das outras, pede confirmação | texto claro ("Excluir módulo"), nunca só um ícone de lixeira solto |
| Ações de uma linha de lista | à direita da linha, botões só-ícone 26 × 26 com dica | `tooltip(...).style(estilo_dica)` |
| Mensagem de estado / progresso | rodapé de status | uma linha, apagada |
| Aviso, erro | perto do campo que causou | texto curto + ícone `info-circle`, nunca só cor |
| Lista vazia | no lugar da lista | ícone 30 apagado + frase + ação ("Abrir planilha") |

**Diálogo pequeno** (confirmar, renomear, escolher uma opção): largura 420–520,
padding 20–24, título 17–18 semibold, uma frase de explicação apagada,
campos em coluna com `spacing(10)`, botões na última linha alinhados à direita
(`Cancelar` secundário, ação primária por último). Enter confirma, Esc cancela.

---

## 6. Ícones

- **Catálogo**: `feather-ui/dados/tabler-acoes.tsv`, uma linha por ícone:
  `nome<TAB>Categoria<TAB>tags,separadas<TAB>caminho1|caminho2|…`.
  Os caminhos são o atributo `d` de cada `<path>` do SVG do Tabler (versão
  3.49.0, licença MIT, `feather-ui/dados/TABLER-LICENSE.txt`).
- **Acrescentar um ícone do Tabler**: abrir o SVG em
  <https://tabler.io/icons> (variante *outline*), copiar o `d` de cada
  `<path>` e juntar com `|`. O renderizador só desenha `<path>`: se o SVG tiver
  `<circle>`, `<rect>` ou `<line>`, converter para caminho (círculo de centro
  `cx,cy` e raio `r`: `M cx-r cy a r r 0 1 0 2r 0 a r r 0 1 0 -2r 0`). Conferir
  com `feather_ui::icones::existe("nome")` num teste.
- **Desenhar um ícone próprio** quando o Tabler não tem: grade 24 × 24, margem
  de 2 px, só traço (`stroke-width 2`, pontas e junções redondas), sem
  preenchimento, no máximo 3–4 formas. Fica no mesmo catálogo, com nome em
  inglês minúsculo com hífen, como os do Tabler.
- **Tamanhos**: 14–15 em botão pequeno e aba, 17–19 em item de lista e barra,
  18 na lateral, 22–25 em cartão, 30+ em estado vazio. O ícone segue a cor do
  texto; destaque com `icones::icone_colorido` só em ação principal da faixa.
- **Ícone sozinho precisa de dica** (`tooltip` com `estilo_dica`) dizendo a
  ação e o atalho.
- Seta de "abrir/ir" é `chevron-right`, voltar é `chevron-left`, não os
  caracteres `›` e `‹`.

---

## 7. Medidas

**Texto** (fonte padrão da suíte; semibold = `Font { weight: Semibold }`):

| Papel | Tamanho | Peso / cor |
|---|---|---|
| Título da página | 22–24 | semibold |
| Título de seção | 16–18 | semibold |
| Título de cartão / item | 13–13,5 | semibold |
| Corpo, campos, listas | 12,5–14 | normal, `line_height(1.4–1.5)` em parágrafo |
| Descrição, ajuda, detalhe | 11–12 | `COR_TEXTO_MUTED()` |
| Rótulo de grupo | 10,5 | CAIXA ALTA, `COR_TEXTO_MUTED()` |
| Código, atalho | 13 | `Font::MONOSPACE` |

**Espaço**: área de conteúdo com padding `[16, 20]`; `spacing(17)` entre
blocos; `12` entre cartões; `8–10` entre controles de uma linha; `2` entre
itens da lateral. **Raios**: painel 12, cartão 11, item de lista 9, botão
8–9, campo/etiqueta 6. **Alturas**: barra de título 42; linha de lista 26–34;
cartão de ajuda 118. **Larguras**: lateral 255–280; diálogo 420–520; conteúdo
de tela larga limitado a ~1180 e centralizado.

**Cores (tokens)**:

| Token | Uso |
|---|---|
| `COR_FUNDO_JANELA()` | fundo atrás dos painéis |
| `COR_PAINEL()` | área de leitura, cartões, campos |
| `COR_PAINEL_FAIXA()` | lateral, cabeçalhos, blocos de código |
| `COR_BORDA()` / `COR_BORDA_SUAVE()` | contorno de cartão / divisórias e painéis |
| `COR_TEXTO()` / `COR_TEXTO_MUTED()` | texto / texto secundário |
| `COR_AZUL_ACENTO()` | cor de destaque escolhida pelo usuário (botão primário, link) |
| `COR_AZUL_FUNDO()` | item selecionado, fundo de ícone de cartão |
| `COR_HOVER()` | passar o mouse |
| `COR_ERRO()` | texto/borda de erro e campo inválido (vermelho legível nos dois temas) |
| `COR_VEU()` | véu atrás de diálogo modal (preto, alfa 0,45 claro / 0,60 escuro) |
| `COR_SOMBRA()` | sombra de diálogo (preto, alfa 0,28; `offset (0, 6)`, `blur_radius 22`) |

**Botão fechar de diálogo**: `feather_ui::tema::botao_fechar(msg)` devolve o
botão pequeno com o ícone `x` (15) e a dica "Fechar"; colocar no canto direito
do cabeçalho do diálogo. Não montar o botão à mão com `×`.

Estilos prontos em `feather_ui::tema`: `estilo_botao_primario`,
`estilo_botao_secundario`, `estilo_botao_pequeno(ativo)`, `estilo_dica`,
`estilo_linha`, `estilo_painel_faixa`, `estilo_rodape`, `estilo_toggle`,
`estilo_menu_da_faixa`, `rolagem`.

---

## 8. Armadilhas do Iced 0.13

- **Conteúdo de `scrollable` vertical nunca tem altura `Fill`**: dá panic.
  O viewport pode ser `Fill`; o filho é `Shrink`. Usar
  `feather_ui::tema::rolagem(...)`.
- **`center_y(Length::Fill)` dentro de rolagem** vira altura infinita (foi o
  travamento da F1); centralizar com tamanho fixo (`center_y(Length::Fixed(..))`).
- **Texto não tem reticências automáticas**: cortar à mão
  (`chars().take(n)` + `…`) calculando pela largura (~7 px por caractere
  em 13 pt). Nome de arquivo corta no fim; caminho de pasta corta no começo.
- **Popup e menu** flutuam com `stack!` + `opaque`, e uma `mouse_area` atrás
  fecha ao clicar fora; não inserir o popup no layout.
- **Grade de cartões** se monta em linhas (`row` por linha, quantos couberem
  na largura): o `wrap` estica os cartões até a borda.
- Cartões de uma mesma grade têm **altura fixa**, senão o texto de duas linhas
  desalinha a fileira.

---

## 9. Lista de conferência antes de entregar

- [ ] Segue a F1 ou o Alt+F11 (e o usuário viu a descrição antes)?
- [ ] Nenhum emoji e nenhum caractere fazendo papel de ícone?
- [ ] Todos os ícones vêm do catálogo; ícone sozinho tem dica?
- [ ] Só tokens de cor; ficou bom no tema **claro e no escuro** e com outra cor de destaque?
- [ ] Uma só ação primária, no canto certo; destrutiva separada e confirmada?
- [ ] Nada desabilitado sem explicação, nada clicável sem ação?
- [ ] Estado vazio, erro e "carregando" desenhados?
- [ ] Textos cortados com `…` em vez de quebrar no meio da palavra?
- [ ] Rolagem sem `Fill` dentro; janela pequena (800 × 600) não quebra?
- [ ] Teclado: Esc fecha/cancela, Enter confirma, Ctrl+F vai para a busca?
- [ ] Teste que monta a `view` nos dois temas (como `janela_monta_todas_as_areas_nas_duas_aparencias`)?
