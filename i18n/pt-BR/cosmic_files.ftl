cosmic-files = Arquivos do COSMIC
empty-folder = Pasta vazia
empty-folder-hidden = Pasta vazia (contém itens ocultos)
no-results = Nenhum item encontrado
filesystem = Sistema de arquivos
home = Pasta pessoal
networks = Redes
notification-in-progress = Há operações de arquivo em andamento.
trash = Lixeira
recents = Recentes
undo = Desfazer
today = Hoje

# List view
name = Nome
modified = Modificação
size = Tamanho

# Dialogs

## Compress Dialog
create-archive = Compactar arquivos

## Empty Trash Dialog
empty-trash = Esvaziar lixeira
empty-trash-warning = Tem certeza de que deseja apagar permanentemente todos os itens da lixeira?

## New File/Folder Dialog
create-new-file = Criar novo arquivo
create-new-folder = Criar nova pasta
file-name = Nome do arquivo
folder-name = Nome da pasta
file-already-exists = Já existe um arquivo com este nome.
folder-already-exists = Já existe uma pasta com este nome.
name-hidden = Nomes iniciando com "." serão ocultados.
name-invalid = O nome não pode ser "{$filename}".
name-no-slashes = O nome não pode conter barras.

## Open/Save Dialog
cancel = Cancelar
create = Criar
open = Abrir
open-file = Abrir arquivo
open-folder = Abrir pasta
open-in-new-tab = Abrir em uma nova aba
open-in-new-window = Abrir em uma nova janela
open-item-location = Abrir local do item
open-multiple-files = Abrir vários arquivos
open-multiple-folders = Abrir várias pastas
save = Salvar
save-file = Salvar arquivo

## Rename Dialog
rename-file = Renomear arquivo
rename-folder = Renomear pasta

## Replace Dialog
replace = Substituir
replace-title = {$filename} já existe neste local.
replace-warning = Deseja substituir o arquivo com o que você está salvando? Substituí-lo irá sobrescrever seu conteúdo.
replace-warning-operation = Deseja substituir o arquivo? Substituí-lo irá sobrescrever seu conteúdo.
original-file = Arquivo original
replace-with = Substituir por
apply-to-all = Aplicar a todos
keep-both = Manter ambos
skip = Ignorar

## Metadata Dialog
owner = Proprietário
group = Grupo
other = Outros
read = Leitura
write = Escrita
execute = Execução

# Context Pages

## About
git-description = Git commit {$hash} de {$date}

## Add Network Drive
add-network-drive = Adicionar local de rede
connect = Conectar
connect-anonymously = Conectar anonimamente
connecting = Conectando...
domain = Domínio
enter-server-address = Insira o endereço do servidor
network-drive-description =
    Endereços de servidor incluem um prefixo de protocolo e um endereço.
    Exemplos: ssh://192.168.0.1, ftp://[2001:db8::1]
### Certifique-se de manter a vírgula que separa as colunas
network-drive-schemes =
    Protocolos disponíveis,Prefixo
    AppleTalk,afp://
    File Transfer Protocol,ftp:// ou ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// ou ssh://
    WebDav,dav:// ou davs://
network-drive-error = Não foi possível acessar o local de rede
password = Senha
remember-password = Lembrar senha
try-again = Tente novamente
username = Usuário

## Operations
edit-history = Editar histórico
history = Histórico
no-history = Nenhum item no histórico.
pending = Pendente
failed = Com falha
complete = Completo
compressing = Compactando {$items} {$items ->
        [one] item
        *[other] itens
    } de {$from} para {$to}
compressed = Compactado {$items} {$items ->
        [one] item
        *[other] itens
    } de {$from} para {$to}
copy_noun = Copiado
creating = Criando {$name} em {$parent}
created = Criado {$name} em {$parent}
copying = Copiando {$items} {$items ->
        [one] item
        *[other] itens
    } de {$from} para {$to}
copied = Copiado {$items} {$items ->
        [one] item
        *[other] itens
    } de {$from} para {$to}
emptying-trash = Esvaziando a lixeira
emptied-trash = Lixeira vazia
extracting = Extraindo {$items} {$items ->
        [one] item
        *[other] itens
    } de {$from} para {$to}
extracted = Extraído {$items} {$items ->
        [one] item
        *[other] itens
    } de {$from} para {$to}
moving = Movendo {$items} {$items ->
        [one] item
        *[other] itens
    } de {$from} para {$to}
moved = Movido {$items} {$items ->
        [one] item
        *[other] itens
    } de {$from} para {$to}
renaming = Renomeando {$from} para {$to}
renamed = Renomeado {$from} para {$to}
restoring = Restaurando {$items} {$items ->
        [one] item
        *[other] itens
    } da lixeira
restored = Restaurado {$items} {$items ->
        [one] item
        *[other] itens
    } da lixeira
unknown-folder = pasta desconhecida

## Open with
open-with = Abrir com...
default-app = {$name} (padrão)

## Show details
show-details = Mostrar detalhes

## Settings
settings = Configurações
settings-tab = Aba
settings-show-hidden = Mostrar arquivos ocultos
default-view = Visualização padrão
icon-size-list = Tamanho do ícone (lista)
icon-size-grid = Tamanho do ícone (grade)
sorting-name = Ordenar por
direction = Classificar
ascending = Crescente
descending = Decrescente

### Appearance
appearance = Aparência
theme = Tema
match-desktop = Acompanhar o ambiente de trabalho
dark = Escuro
light = Claro

# Context menu
add-to-sidebar = Adicionar à barra lateral
compress = Compactar
extract-here = Extrair
new-file = Novo arquivo...
new-folder = Nova pasta...
open-in-terminal = Abrir no terminal
move-to-trash = Mover para a lixeira
restore-from-trash = Restaurar da lixeira
remove-from-sidebar = Remover da barra lateral
sort-by-name = Ordenar por nome
sort-by-modified = Ordenar por data de modificação
sort-by-size = Ordenar por tamanho

# Menu

## File
file = Arquivo
new-tab = Nova aba
new-window = Nova janela
rename = Renomear...
menu-show-details = Mostrar detalhes...
close-tab = Fechar aba
quit = Sair

## Edit
edit = Editar
cut = Recortar
copy = Copiar
paste = Colar
select-all = Selecionar tudo

## View
zoom-in = Ampliar
default-size = Tamanho padrão
zoom-out = Reduzir
view = Exibir
grid-view = Exibição em grade
list-view = Exibição em lista
show-hidden-files = Mostrar arquivos ocultos
list-directories-first = Listar pastas primeiro
menu-settings = Configurações...
menu-about = Sobre o Arquivos do COSMIC...

## Sort
sort = Ordenar
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Mais novos primeiro
sort-oldest-first = Mais antigos primeiro
sort-smallest-to-largest = Do menor para o maior
sort-largest-to-smallest = Do maior para o menor
