cosmic-files = Ficheiros COSMIC
empty-folder = Pasta vazia
empty-folder-hidden = Pasta vazia (tem ficheiros ocultos)
no-results = Nenhum resultado encontrado
filesystem = Sistema de ficheiros
home = Pasta Pessoal
notification-in-progress = Operações em curso.
trash = Lixo
undo = Anular
# List view
name = Nome
modified = Modificado
size = Tamanho

# Dialogs


## Empty Trash Dialog

empty-trash = Esvaziar lixo
empty-trash-warning = Pretende eliminar permanentemente todos os itens do Lixo?
# New File/Folder Dialog
create-new-file = Criar novo ficheiro
create-new-folder = Criar nova pasta
file-name = Nome do ficheiro
folder-name = Nome da pasta
file-already-exists = Já existe um ficheiro com esse nome.
folder-already-exists = Já existe uma pasta com esse nome.
name-hidden = Os nomes começados por "." serão ocultados.
name-invalid = O nome não pode ser "{ $filename }".
name-no-slashes = O nome não pode conter barras.
# Open/Save Dialog
cancel = Cancelar
open = Abrir
open-file = Abrir ficheiro
open-folder = Abrir pasta
open-in-new-tab = Abrir num novo separador
open-in-new-window = Abrir numa nova janela
open-item-location = Abrir localização do item
open-multiple-files = Abrir vários ficheiros
open-multiple-folders = Abrir várias pastas
save = Guardar
save-file = Guardar ficheiro
# Rename Dialog
rename-file = Renomear ficheiro
rename-folder = Renomear pasta
# Replace Dialog
replace = Substituir
replace-title = { $filename } já existe neste local.
replace-warning = Substituí-lo pelo que está a guardar? Se o substituir, o seu conteúdo será substituído.
replace-warning-operation = Pretende substituí-lo? Ao substituí-lo, o seu conteúdo será substituído.
original-file = Ficheiro original
replace-with = Substituir por
apply-to-all = Aplicar a tudo
keep-both = Manter ambos
skip = Ignorar

## Metadata Dialog

owner = Proprietário
group = Grupo
other = Outro
read = Leitura
write = Escrita
execute = Execução

# Context Pages


## About

git-description = Git commit { $hash } em { $date }

## Operations

edit-history = Editar histórico
history = Histórico
no-history = Nenhum item no histórico.
pending = Pendentes
failed = Com falha
complete = Concluído
copy_noun = Copiado
creating = A criar "{ $name }" em "{ $parent }"
created = "{ $name }" criado em "{ $parent }"
copying =
    A copiar { $items } { $items ->
        [one] item
       *[other] itens
    } de "{ $from }" para "{ $to }" ({ $progress })...
copied =
    { $items } { $items ->
        [one] item copiado
       *[other] itens copiados
    } de "{ $from }" para "{ $to }"
emptying-trash = A esvaziar { trash } ({ $progress })...
emptied-trash = { trash } esvaziado
extracting =
    A extrair { $items } { $items ->
        [one] item
       *[other] itens
    } de "{ $from }" par "{ $to }" ({ $progress })...
extracted =
    { $items } { $items ->
        [one] item extraído
       *[other] itens extraídos
    } de "{ $from }" para "{ $to }"
moving =
    A mover { $items } { $items ->
        [one] item
       *[other] itens
    } de "{ $from }" para "{ $to }" ({ $progress })...
moved =
    { $items } { $items ->
        [one] item movido
       *[other] itens movidos
    } de "{ $from }" para "{ $to }"
renaming = A renomear "{ $from }" para "{ $to }"
renamed = "{ $from }" renomeado para "{ $to }"
restoring =
    A restaurar { $items } { $items ->
        [one] item
       *[other] itens
    } de { trash } ({ $progress })...
restored =
    Restaurado { $items } { $items ->
        [one] item
       *[other] itens
    } para { trash }
unknown-folder = pasta desconhecida

## Open with

menu-open-with = Abrir com...
default-app = { $name } (predefinição)

## Show details

show-details = Mostrar detalhes

## Settings

settings = Definições

### Appearance

appearance = Aparência
theme = Tema
match-desktop = Estilo do sistema
dark = Escuro
light = Claro
# Context menu
extract-here = Extrair
add-to-sidebar = Adicionar à barra lateral
new-file = Novo ficheiro...
new-folder = Nova pasta...
open-in-terminal = Abrir no terminal
move-to-trash = Mover para o lixo
restore-from-trash = Restaurar do lixo
remove-from-sidebar = Remover da barra lateral
sort-by-name = Ordenar por nome
sort-by-modified = Ordenar por data de modificação
sort-by-size = Ordenar por tamanho

# Menu


## File

file = Ficheiro
new-tab = Novo separador
new-window = Nova janela
rename = Renomear...
menu-show-details = Mostrar detalhes...
close-tab = Fechar separador
quit = Sair

## Edit

edit = Editar
cut = Cortar
copy = Copiar
paste = Colar
select-all = Selecionar tudo

## View

zoom-in = Aumentar
default-size = Tamanho predefinido
zoom-out = Diminuir
view = Ver
grid-view = Visualização em grelha
list-view = Visualização em lista
show-hidden-files = Mostrar ficheiros ocultos
list-directories-first = Listar primeiro os diretórios
menu-settings = Definições...
menu-about = Acerca do Ficheiros COSMIC...
repository = Repositório
support = Suporte
details = Detalhes
dismiss = Dispensar mensagem
remove = Remover
cancelled = Canceladas
networks = Redes
recents = Recentes
today = Hoje
desktop-view-options = Opções de visualização da área de trabalho...
show-on-desktop = Mostrar na área de trabalho
desktop-folder-content = Conteúdo da pasta da área de trabalho
mounted-drives = Dispositivos montados
trash-folder-icon = Ícone do lixo
icon-size-and-spacing = Tamanho e espaçamento do ícone
icon-size = Tamanho do ícone
grid-spacing = Espaçamento entre ícones
trashed-on = Enviado para o lixo
operations-running =
    { $running } { $running ->
        [one] operação
       *[other] operações
    } em execução ({ $percent }%)...
operations-running-finished =
    { $running } { $running ->
        [one] operação
       *[other] operações
    } em execução ({ $percent }%), { $finished } concluídas...
pause = Pausa
resume = Retomar
create-archive = Criar arquivo
extract-password-required = Palavra-passe necessária
extract-to = Extrair para...
extract-to-title = Extrair para pasta
mount-error = Não foi possível aceder ao dispositivo
create = Criar
open-with-title = Como pretende abrir "{ $name }"?
browse-store = Procurar em { $store }
other-apps = Outras aplicações
related-apps = Aplicações relacionadas
selected-items = os { $items } itens selecionados
permanently-delete-question = Eliminar permanentemente
delete = Eliminar
permanently-delete-warning = Tem a certeza de que pretende eliminar { $target } permanentemente? Esta ação não pode ser anulada.
set-executable-and-launch = Definir como executável e iniciar
set-executable-and-launch-description = Pretende definir "{ $name }" como executável e iniciá-lo?
set-and-launch = Definir e iniciar
open-with = Abrir com
none = Nenhum(a)
execute-only = Executar-apenas
write-only = Gravar-apenas
write-execute = Gravação e execução
read-only = Apenas-leitura
read-execute = Leitura e execução
read-write = Leitura e escrita
read-write-execute = Leitura, gravação e execução
favorite-path-error = Erro ao abrir diretório
favorite-path-error-description =
    Não foi possível abrir "{ $path }".
    O item pode não existir ou não tem permissão para abri-lo.

    Pretende removê-lo da barra lateral?
keep = Manter
add-network-drive = Adicionar unidade de rede
connect = Ligar
connect-anonymously = Ligar anonimamente
connecting = A ligar…
domain = Domínio
enter-server-address = Insira o endereço do servidor
network-drive-description =
    Endereços de servidor incluem um prefixo de protocolo e um endereço.
    Exemplos: ssh://192.168.0.1, ftp://[2001:db8::1]
network-drive-schemes =
    Protocolos disponíveis,Prefixo
    AppleTalk,afp://
    File Transfer Protocol,ftp:// ou ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// ou ssh://
    WebDAV,dav:// ou davs://
network-drive-error = Não é possível aceder à unidade de rede
password = Palavra-passe
remember-password = Memorizar palavra-passe
try-again = Tentar novamente
username = Nome de utilizador
progress = { $percent }%
progress-cancelled = { $percent }%, cancelado
progress-failed = { $percent }%, com falha
progress-paused = { $percent }%, em pausa
compressing =
    A comprimir { $items } { $items ->
        [one] item
       *[other] itens
    } de "{ $from }" para "{ $to }" ({ $progress })...
compressed =
    { $items } { $items ->
        [one] item comprimido
       *[other] itens comprimidos
    } de "{ $from }" para "{ $to }"
deleting =
    A eliminar { $items } { $items ->
        [one] item
       *[other] itens
    } do { trash } ({ $progress })...
deleted =
    { $items } { $items ->
        [one] item eliminado
       *[other] itens eliminados
    } do { trash }
setting-executable-and-launching = A definir "{ $name }" como executável e a iniciar
set-executable-and-launched = "{ $name }" definido como executável e iniciado
setting-permissions = A definir permissões de "{ $name }" para { $mode }
set-permissions = Definir permissões de "{ $name }" para { $mode }
permanently-deleting =
    A eliminar permanentemente { $items } { $items ->
        [one] item
       *[other] itens
    }
permanently-deleted =
    { $items } { $items ->
        [one] item eliminado
       *[other] itens eliminados
    } permanentemente
removing-from-recents =
    A remover { $items } { $items ->
        [one] item
       *[other] itens
    } de { recents }
removed-from-recents =
    { $items } { $items ->
        [one] item removido
       *[other] itens removidos
    } de { recents }
type = Tipo: { $mime }
items = Itens: { $items }
item-size = Tamanho: { $size }
item-created = Criado: { $created }
item-modified = Modificado: { $modified }
item-accessed = Acedido: { $accessed }
calculating = A calcular...
single-click = Um único clique para abrir
type-to-search = Escreva para pesquisar
type-to-search-recursive = Pesquisa na pasta atual e em todas as subpastas
type-to-search-enter-path = Insere o caminho do diretório ou ficheiro
compress = Comprimir
delete-permanently = Eliminar permanentemente
eject = Ejetar
sort-by-trashed = Ordenar por data de eliminação
remove-from-recents = Remover dos itens recentes
change-wallpaper = Alterar papel de parede...
desktop-appearance = Aparência da área de trabalho...
display-settings = Definições do ecrã...
reload-folder = Recarregar pasta
gallery-preview = Pré-visualizar
sort = Ordenar
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Mais recentes primeiro
sort-oldest-first = Mais antigos primeiro
sort-smallest-to-largest = Do menor para o maior
sort-largest-to-smallest = Do maior para o menor
