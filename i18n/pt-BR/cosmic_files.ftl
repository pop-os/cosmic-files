cosmic-files = Arquivos
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

# Desktop view options
desktop-view-options = Opções de visualização do desktop...
show-on-desktop = Mostrar no desktop
desktop-folder-content = Conteúdo da pasta do desktop
mounted-drives = Dispositivos montados
trash-folder-icon = Ícone da lixeira
icon-size-and-spacing = Tamanho e espaçamento do ícone
icon-size = Tamanho do ícone
grid-spacing = Espaçamento entre ícones

# List view
name = Nome
modified = Modificação
trashed-on = Exclusão
size = Tamanho

# Progress footer
details = Detalhes
dismiss = Dispensar mensagem
operations-running = {$running} operações em andamento ({$percent}%)...
operations-running-finished = {$running} operações em andamento ({$percent}%), {$finished} finalizadas...
pause = Pausar
resume = Continuar

# Dialogs

## Compress Dialog
create-archive = Compactar arquivos

## Extract Dialog
extract-password-required = Senha necessária
extract-to = Extrair para...
extract-to-title = Extrair para pasta

## Empty Trash Dialog
empty-trash = Esvaziar a lixeira
empty-trash-warning = Tem certeza de que deseja apagar permanentemente todos os itens da lixeira?

## Mount Error Dialog
mount-error = Não foi possível acessar o dispositivo

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

## Open With Dialog
open-with-title = Como deseja abrir "{$name}"?
browse-store = Procurar em {$store}

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

## Set as Executable and Launch Dialog
set-executable-and-launch = Marcar como executável e iniciar
set-executable-and-launch-description = Deseja marcar "{$name}" como executável e iniciá-lo?
set-and-launch = Marcar e iniciar

## Metadata Dialog
open-with = Abrir com
owner = Proprietário
group = Grupo
other = Outros
### Mode 0
none = Nenhum
### Mode 1 (unusual)
execute-only = Somente execução
### Mode 2 (unusual)
write-only = Somente escrita
### Mode 3 (unusual)
write-execute = Escrita e execução
### Mode 4
read-only = Somente leitura
### Mode 5
read-execute = Leitura e execução
### Mode 6
read-write = Leitura e escrita
### Mode 7
read-write-execute = Leitura, escrita e execução

## Favorite Path Error Dialog
favorite-path-error = Erro ao abrir diretório
favorite-path-error-description =
    Não foi possível abrir "{$path}".
    O item pode não existir ou você não tem permissão para abri-lo.
    
    Deseja removê-lo da barra lateral?
remove = Remover
keep = Manter

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
cancelled = Cancelado
edit-history = Editar histórico
history = Histórico
no-history = Nenhum item no histórico.
pending = Pendente
progress = {$percent}%
progress-cancelled = {$percent}%, cancelado
progress-paused = {$percent}%, pausado
failed = Com falha
complete = Concluído
compressing = Compactando {$items} {$items ->
        [one] item
        *[other] itens
    } de {$from} para {$to} ({$progress})...
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
    } de {$from} para {$to} ({$progress})...
copied = Copiado {$items} {$items ->
        [one] item
        *[other] itens
    } de {$from} para {$to}
deleting = Apagando {$items} {$items ->
        [one] item
        *[other] itens
    } da {trash} ({$progress})...
deleted = Apagado {$items} {$items ->
        [one] item
        *[other] itens
    } da {trash}
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
setting-executable-and-launching = Marcando "{$name}" como executável e iniciando
set-executable-and-launched = Marcado "{$name}" como executável e iniciado
moving = Movendo {$items} {$items ->
        [one] item
        *[other] itens
    } de {$from} para {$to} ({$progress})...
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
menu-open-with = Abrir com...
default-app = {$name} (padrão)

## Show details
show-details = Mostrar detalhes
type = Tipo: {$mime}
items = Itens: {$items}
item-size = Tamanho: {$size}
item-created = Criado: {$created}
item-modified = Modificado: {$modified}
item-accessed = Acessado: {$accessed}
calculating = Calculando...

## Settings
settings = Configurações
single-click = Clique simples para abrir

### Appearance
appearance = Aparência
theme = Tema
match-desktop = Acompanhar o ambiente de trabalho
dark = Escuro
light = Claro

### Type to Search
type-to-search = Pesquisar digitando
type-to-search-recursive = Pesquisa na pasta atual e em todas as subpastas
type-to-search-enter-path = Insere o caminho do diretório ou arquivo

# Context menu
add-to-sidebar = Adicionar à barra lateral
compress = Compactar
delete-permanently = Excluir permanentemente
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
sort-by-trashed = Ordernar por data de exclusão

## Desktop
change-wallpaper = Alterar papel de parede...
desktop-appearance = Aparência do desktop...
display-settings = Configurações da tela...

# Menu

## File
file = Arquivo
new-tab = Nova aba
new-window = Nova janela
rename = Renomear...
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
gallery-preview = Pré-visualizar
menu-settings = Configurações...
menu-about = Sobre o Arquivos...

## Sort
sort = Ordenar
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Mais novos primeiro
sort-oldest-first = Mais antigos primeiro
sort-smallest-to-largest = Do menor para o maior
sort-largest-to-smallest = Do maior para o menor
