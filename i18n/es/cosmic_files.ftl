cosmic-files = COSMIC Files
empty-folder = Carpeta vacía
empty-folder-hidden = Carpeta vacía (Contiene archivos ocultos)
no-results = No se encontraron resultados
filesystem = Sistema de archivos
home = Inicio
networks = Redes
notification-in-progress = Las operaciones de archivo están en progreso.
trash = Papelera
recents = Reciente
undo = Deshacer
today = Hoy

# Desktop view options
desktop-view-options = Opciones de vista del escritorio...
show-on-desktop = Mostrar en el escritorio
desktop-folder-content = Contenido de la carpeta del escritorio
mounted-drives = Unidades montadas
trash-folder-icon = Icono de la {trash}
icon-size-and-spacing = Tamaño y espaciado de los iconos
icon-size = Tamaño del icono
grid-spacing = Espaciado de la cuadrícula

# List view
name = Nombre
modified = Modificado
trashed-on = Enviado a la {trash}
size = Tamaño

# Progress footer
details = Detalles
dismiss = Descartar mensaje
operations-running = {$running} operaciones ejecutándose ({$percent}%)...
operations-running-finished = {$running} operaciones ejecutándose ({$percent}%), {$finished} finalizadas...
pause = Pausar
resume = Reanudar

# Dialogs

## Compress Dialog
create-archive = Crear archivo

## Extract Dialog
extract-password-required = Contraseña requerida
extract-to = Extraer en...
extract-to-title = Extraer a una carpeta

## Empty Trash Dialog
empty-trash = Vaciar la {trash} 
empty-trash-warning = ¿Está seguro de que quiere eliminar permanentemente todos los archivos de la {trash}?

## Mount Error Dialog
mount-error = No se puede acceder a la unidad

## New File/Folder Dialog
create-new-file = Crear nuevo archivo
create-new-folder = Crear nueva carpeta
file-name = Nombre del archivo
folder-name = Nombre de la carpeta
file-already-exists = Ya existe un archivo con ese nombre.
folder-already-exists = Ya existe una carpeta con ese nombre.
name-hidden = Nombres comenzando con "." serán ocultados.
name-invalid = El nombre no puede ser: "{$filename}".
name-no-slashes = El nombre no puede contener slashes (barras).

## Open/Save Dialog
cancel = Cancelar
create = Crear
open = Abrir
open-file = Abrir archivo
open-folder = Abrir carpeta
open-in-new-tab = Abrir en nueva pestaña
open-in-new-window = Abrir en nueva ventana
open-item-location = Abrir ubicación del archivo
open-multiple-files = Abrir multiples archivos
open-multiple-folders = Abrir multiples carpetas
save = Guardar
save-file = Guardar archivo

## Open With Dialog
open-with-title = ¿Cómo quiere abrir «{$name}»?
browse-store = Explorar {$store}
other-apps = Otras aplicaciones
related-apps = Aplicaciones relacionadas

## Permanently delete Dialog
selected-items = los {$items} archivos seleccionados
permanently-delete-question = Eliminar permanentemente
delete = Eliminar
permanently-delete-warning = ¿Quiere eliminar permanentemente {$nb_items ->
        [one] el archivo {$target}? El archivo no podrá ser restaurado
        *[other] {$target}? Los archivos no podrán ser restaurados
    }.

## Rename Dialog
rename-file = Renombrar archivo
rename-folder = Renombrar carpeta

## Replace Dialog
replace = Reemplazar
replace-title = {$filename} ya existe en esta ruta.
replace-warning = ¿Quiere remplazarlo con el que está guardando? Reemplazarlo sobrescribirá su contenido.
replace-warning-operation = ¿Quieres reemplazarlo? Reemplazarlo sobrescribirá su contenido.
original-file = Archivo original
replace-with = Reemplazar con
apply-to-all = Aplicar a todos
keep-both = Conservar ambos
skip = Saltar

## Set as Executable and Launch Dialog
set-executable-and-launch = Establecer como ejecutable y ejecutar
set-executable-and-launch-description = ¿Quieres establecer «{$name}» como ejecutable y ejecutar?
set-and-launch = Establecer y ejecutar

## Metadata Dialog
open-with = Abrir con
owner = Propietario
group = Grupo
other = Otro
### Mode 0
none = Ninguno
### Mode 1 (unusual)
execute-only = Únicamente ejecución 
### Mode 2 (unusual)
write-only = Únicamente escritura 
### Mode 3 (unusual)
write-execute = Escritura y ejecución
### Mode 4
read-only = Únicamente lectura 
### Mode 5
read-execute = Lectura y ejecución
### Mode 6
read-write = Lectura y escritura
### Mode 7
read-write-execute = Lectura, escritura y ejecución

## Favorite Path Error Dialog
favorite-path-error = Error al abrir el directorio
favorite-path-error-description =
    No se puede abrir "{$path}".
    Puede que no exista o que no tenga permisos para abrirlo.
    
    ¿Quiere eliminarlo de la barra lateral?
remove = Eliminar
keep = Mantener

# Context Pages

## About
git-description = Git Commit: {$hash} - Fecha: {$date}

## Add Network Drive
add-network-drive = Agregar una unidad de red
connect = Conectar
connect-anonymously = Conectar de forma anónima
connecting = Conectando...
domain = Dominio
enter-server-address = Ingresa la dirección del servidor
network-drive-description =
    Las direcciones de los servidores incluyen un prefijo de protocolo y una dirección.
    Ejemplos: ssh://192.168.0.1, ftp://[2001:db8::1]
### Make sure to keep the comma which separates the columns
network-drive-schemes =
    Protocolos disponibles,Prefijo
    AppleTalk,afp://
    Protocolo de transferencia de archivos,ftp:// o ftps://
    Sistema de archivos de red,nfs://
    Bloque de mensajes del servidor,smb://
    Protocolo de transferencia de archivos SSH,sftp:// o ssh://
    WebDav,dav:// o davs://
network-drive-error = No se puede acceder a la unidad de red
password = Contraseña
remember-password = Recordar contraseña
try-again = Intentar de nuevo
username = Nombre de usuario

## Operations
cancelled = Cancelada
edit-history = Historial de ediciones
history = Historial
no-history = No hay archivos en el historial.
pending = Pendiente
progress = {$percent}%
progress-cancelled = {$percent}%, cancelado
progress-paused = {$percent}%, pausado
failed = Fallidas
complete = Completadas
compressing = {$items ->
        [one] Comprimiendo un archivo
        *[other] Comprimiendo {$items} archivos
    } de {$from} a {$to}
compressed = {$items ->
        [one] Comprimido un archivo
        *[other] Comprimidos {$items} archivos
    } de {$from} a {$to}
copy_noun = Copia
creating = Creando {$name} en {$parent}
created = Se han creado {$name} en {$parent}
copying = {$items ->
        [one] Copiando un archivo
        *[other] Copiando {$items} archivos
    } desde {$from} a {$to}
copied = {$items ->
        [one] Se ha copiado un archivo
        *[other] Se han copiado {$items} archivos
    } desde {$from} a {$to}
deleting = {$items ->
        [one] Eliminando un archivo
        *[other] Eliminando {$items} archivos
    } de la {trash} ({$progress})...
deleted = {$items ->
        [one] Se ha eliminado un archivo
        *[other] Se han eliminado {$items} archivos
    } de la {trash}
emptying-trash = Vaciando la {trash}
emptied-trash = Se ha vaciado la {trash}
extracting = {$items ->
        [one] Extrayendo un archivo
        *[other] Extrayendo {$items} archivos
    } de {$from} a {$to}
extracted = {$items ->
        [one] Se ha extraído un archivo
        *[other] Se han extraído {$items} archivos
    } de {$from} a {$to}
setting-executable-and-launching = Estableciendo «{$name}» como ejecutable y lanzando
set-executable-and-launched = Se ha establecido «{$name}» como ejecutable y lanzado
moving = {$items ->
        [one] Moviendo un archivo
        *[other] Moviendo {$items} archivos
    } desde {$from} a {$to}
moved = {$items ->
        [one] Se ha movido un archivo
        *[other] Se han movido {$items} archivos
    } desde {$from} a {$to}
permanently-deleting = {$items ->
        [one] Eliminando un archivo permanentemente
        *[other] Eliminando permanentemente {$items} archivos
    }
permanently-deleted = {$items ->
        [one] Se ha eliminado un archivo permanentemente
        *[other] Se han eliminado {$items} archivos permanentemente
    }
renaming = Renombrando {$from} a {$to}
renamed = Se ha renombrado {$from} a {$to}
restoring = {$items ->
        [one] Restaurando un archivo
        *[other] Restaurando {$items} archivos
    } desde la {trash}
restored = {$items ->
        [one] Se ha restaurado un archivo
        *[other] Se han restaurado {$items} archivos
    } desde la {trash} 
unknown-folder = carpeta desconocida

## Open with
menu-open-with = Abrir con
default-app = {$name} (predeterminado)

## Show details
show-details = Mostrar detalles
type = Tipo: {$mime}
items = Archivos: {$items}
item-size = Tamaño: {$size}
item-created = Fecha de creación: {$created}
item-modified = Última modificación: {$modified}
item-accessed = Último acceso: {$accessed}
calculating = Calculando...

## Settings
settings = Configuración
single-click = Abrir con solo un clic

### Appearance
appearance = Apariencia
theme = Tema
match-desktop = Seguir el estilo del escritorio
dark = Oscuro
light = Claro

### Type to Search
type-to-search = Escriba para buscar
type-to-search-recursive = Buscar en la carpeta actual y todas sus subcarpetas
type-to-search-enter-path = Escriba la ruta del directorio o archivo

# Context menu
add-to-sidebar = Añadir a la barra lateral
compress = Comprimir
delete-permanently = Eliminar permanentemente
extract-here = Extraer aquí
new-file = Nuevo archivo
new-folder = Nueva carpeta
open-in-terminal = Abrir en la consola 
move-to-trash = Mover a la {trash}
restore-from-trash = Restaurar desde la {trash}
remove-from-sidebar = Quitar de la barra lateral
sort-by-name = Ordenar por nombre
sort-by-modified = Ordenar por fecha de modificación
sort-by-size = Ordenar por tamaño
sort-by-trashed = Ordenar por fecha de eliminación

## Desktop
change-wallpaper = Cambiar fondo de pantalla...
desktop-appearance = Apariencia del escritorio...
display-settings = Configuración de pantalla...

# Menu

## File
file = Archivo
new-tab = Nueva pestaña
new-window = Nueva ventana
rename = Renombrar...
close-tab = Cerrar pestaña
quit = Cerrar

## Edit
edit = Editar
cut = Cortar
copy = Copiar
paste = Pegar
select-all = Seleccionar todo

## View
zoom-in = Ampliar
default-size = Tamaño predeterminado
zoom-out = Disminuir
view = Vista
grid-view = Vista de cuadrícula
list-view = Vista de lista
show-hidden-files = Mostrar archivos ocultos
list-directories-first = Enumerar los directorios primero
gallery-preview = Vista previa de la galería
menu-settings = Configuración...
menu-about = Acerca de COSMIC Files...

## Sort
sort = Ordenar
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Más reciente primero
sort-oldest-first = Más antiguo primero
sort-smallest-to-largest = De menor a mayor
sort-largest-to-smallest = De mayor a menor
