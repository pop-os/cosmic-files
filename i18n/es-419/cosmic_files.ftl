cosmic-files = Archivos de COSMIC
empty-folder = Carpeta vacía
empty-folder-hidden = Carpeta vacía (tiene elementos ocultos)
no-results = No se encontraron resultados
filesystem = Sistema de archivos
home = Inicio
networks = Redes
notification-in-progress = Las operaciones de archivo están en progreso.
trash = Papelera
recents = Recientes
undo = Deshacer
today = Hoy
# Desktop view options
desktop-view-options = Opciones de vista del escritorio...
show-on-desktop = Mostrar en el escritorio
desktop-folder-content = Contenido de la carpeta del escritorio
mounted-drives = Unidades montadas
trash-folder-icon = Icono de la papelera
icon-size-and-spacing = Tamaño y espaciado de los iconos
icon-size = Tamaño del icono
# List view
name = Nombre
modified = Modificado
trashed-on = Enviado a la papelera
size = Tamaño

# Dialogs


## Compress Dialog

create-archive = Crear archivo comprimido

## Empty Trash Dialog

empty-trash = Vaciar la papelera
empty-trash-warning = ¿Estás seguro de que deseas eliminar permanentemente todos los elementos de la papelera?

## New File/Folder Dialog

create-new-file = Crear nuevo archivo
create-new-folder = Crear nueva carpeta
file-name = Nombre de archivo
folder-name = Nombre de carpeta
file-already-exists = Ya existe un archivo con ese nombre.
folder-already-exists = Ya existe una carpeta con ese nombre.
name-hidden = Los nombres que comiencen con "." se ocultarán.
name-invalid = El nombre no puede ser "{ $filename }".
name-no-slashes = El nombre no puede contener barras.

## Open/Save Dialog

cancel = Cancelar
create = Crear
open = Abrir
open-file = Abrir archivo
open-folder = Abrir carpeta
open-in-new-tab = Abrir en una nueva pestaña
open-in-new-window = Abrir en una nueva ventana
open-item-location = Abrir ubicación del elemento
open-multiple-files = Abrir múltiples archivos
open-multiple-folders = Abrir múltiples carpetas
save = Guardar
save-file = Guardar archivo

## Open With Dialog

open-with-title = ¿Cómo deseas abrir "{ $name }"?
browse-store = Explorar { $store }

## Rename Dialog

rename-file = Cambiar el nombre del archivo
rename-folder = Cambiar el nombre de la carpeta

## Replace Dialog

replace = Reemplazar
replace-title = "{ $filename }" ya existe en esta ubicación.
replace-warning = ¿Quieres reemplazarlo con el que estás guardando? Reemplazarlo sobrescribirá su contenido.
replace-warning-operation = ¿Quieres reemplazarlo? Reemplazarlo sobrescribirá su contenido.
original-file = Archivo original
replace-with = Reemplazar con
apply-to-all = Aplicar a todos
keep-both = Conservar ambos
skip = Saltar

## Set as Executable and Launch Dialog

set-executable-and-launch = Establecer como ejecutable y ejecutar
set-executable-and-launch-description = ¿Deseas establecer "{ $name }" como ejecutable y abrirlo?
set-and-launch = Establecer y ejecutar

## Metadata Dialog

owner = Propietario
group = Grupo
other = Otro
read = Leer
write = Escribir
execute = Ejecutar

# Context Pages


## About

git-description = «Commit» { $hash } de Git del { $date }

## Add Network Drive

add-network-drive = Agregar unidad de red
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

edit-history = Historial de ediciones
history = Historial
no-history = No hay elementos en el historial.
pending = Pendientes
failed = Con error
complete = Completadas
compressing =
    Comprimiendo { $items } { $items ->
        [one] elemento
       *[other] elementos
    } de "{ $from }" a "{ $to }" ({ $progress })...
compressed =
    { $items ->
        [one] Se ha comprimido un elemento
       *[other] Se han comprimidos { $items } elementos
    } de "{ $from }" a "{ $to }"
copy_noun = Copiar
creating = Creando "{ $name }" en "{ $parent }"
created = Se ha creado "{ $name }" en "{ $parent }"
copying =
    Copiando { $items } { $items ->
        [one] elemento
       *[other] elementos
    } de "{ $from }" a "{ $to }" ({ $progress })...
copied =
    { $items ->
        [one] Se ha copiado un elemento
       *[other] Se han copiado { $items } elementos
    } de "{ $from }" a "{ $to }"
emptying-trash = Vaciando la { trash } ({ $progress })...
emptied-trash = Se ha vaciado la { trash }
extracting =
    Extrayendo { $items } { $items ->
        [one] elemento
       *[other] elementos
    } de "{ $from }" a "{ $to }" ({ $progress })...
extracted =
    { $items ->
        [one] Se ha extraído un elemento
       *[other] Se han extraído { $items } elementos
    } de "{ $from }" a "{ $to }"
setting-executable-and-launching = Estableciendo "{ $name }" como ejecutable y abriendo
set-executable-and-launched = Se ha establecido "{ $name }" como ejecutable y se ha abierto
moving =
    Moviendo { $items } { $items ->
        [one] elemento
       *[other] elementos
    } de "{ $from }" a "{ $to }" ({ $progress })...
moved =
    { $items ->
        [one] Se ha movido un elemento
       *[other] Se han movido { $items } elementos
    } de "{ $from }" a "{ $to }"
renaming = Cambiando el nombre de "{ $from }" a "{ $to }"
renamed = Se ha cambiado el nombre de "{ $from }" a "{ $to }"
restoring =
    Restaurando { $items } { $items ->
        [one] elemento
       *[other] elementos
    } de la { trash } ({ $progress })...
restored =
    { $items ->
        [one] Se ha restaurado un elemento
       *[other] Se han restaurado { $items } elementos
    } de la { trash }
unknown-folder = carpeta desconocida

## Open with

menu-open-with = Abrir con...
default-app = { $name } (predeterminado)

## Show details

show-details = Mostrar detalles

## Settings

settings = Configuración

### Appearance

appearance = Apariencia
theme = Tema
match-desktop = Igual que el escritorio
dark = Oscuro
light = Claro
# Context menu
add-to-sidebar = Añadir a la barra lateral
compress = Comprimir
extract-here = Extraer aquí
new-file = Archivo nuevo...
new-folder = Carpeta nueva...
open-in-terminal = Abrir en una terminal
move-to-trash = Mover a la papelera
restore-from-trash = Restaurar de la papelera
remove-from-sidebar = Quitar de la barra lateral
sort-by-name = Ordenar por nombre
sort-by-modified = Ordenar por modificado
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
rename = Cambiar nombre...
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
view = Vistar
grid-view = Vista de cuadrícula
list-view = Vista de lista
show-hidden-files = Mostrar archivos ocultos
list-directories-first = Mostrar directorios primero
gallery-preview = Vista previa de la galería
menu-settings = Configuración...
menu-about = Acerca de archivos COSMIC...

## Sort

sort = Ordenar
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Más reciente primero
sort-oldest-first = Más antiguo primero
sort-smallest-to-largest = De menor a mayor
sort-largest-to-smallest = De mayor a menor
repository = Repositorio
support = Apoyo
details = Detalles
dismiss = Descartar mensaje
remove = Eliminar
cancelled = Canceladas
grid-spacing = Espaciado de cuadrícula
operations-running =
    { $running ->
        [one] Operación de { $running }
       *[other] Operaciones de { $running }
    } en ejecución ({ $percent } %)...
operations-running-finished =
    { $running ->
        [one] Operación de { $running }
       *[other] Operaciones de { $running }
    } en ejecución ({ $percent } %), { $finished } completada(s)...
pause = Pausar
resume = Reanudar
extract-password-required = Contraseña requerida
extract-to = Extraer en...
extract-to-title = Extraer a una carpeta
mount-error = No se puede acceder a la unidad
other-apps = Otras aplicaciones
related-apps = Aplicaciones relacionadas
selected-items = los { $items } archivos seleccionados
permanently-delete-question = Eliminar de forma permanente
delete = Eliminar
permanently-delete-warning = ¿Estás seguro de que quieres eliminar { $target } de forma permanente? Esta acción no se puede deshacer.
open-with = Abrir con
none = Ninguno
execute-only = Solo ejecución
write-only = Solo escritura
write-execute = Escritura y ejecución
read-only = Solo lectura
read-execute = Lectura y ejecución
read-write = Lectura y escritura
read-write-execute = Lectura, escritura y ejecución
favorite-path-error = Error al abrir el directorio
favorite-path-error-description =
    No se puede abrir "{ $path }".
    Puede que no exista o que no tengas permiso a abrirlo.

    ¿Quieres eliminarlo de la barra lateral?
keep = Reservar
progress = { $percent } %
progress-cancelled = { $percent } %, cancelado
progress-failed = { $percent } %, con errores
progress-paused = { $percent } %, pausado
setting-permissions = Estableciendo permisos de "{ $name }" como { $mode }
set-permissions = Establecer permisos de "{ $name }" como { $mode }
permanently-deleting =
    Eliminando { $items } { $items ->
        [one] elemento
       *[other] elementos
    } de forma permanente
deleting =
    Eliminando { $items } { $items ->
        [one] elemento
       *[other] elementos
    } de la { trash } ({ $progress })...
deleted =
    { $items ->
        [one] Se ha eliminado un elemento
       *[other] Se han eliminado { $items } elementos
    } de la { trash }
permanently-deleted =
    { $items ->
        [one] Se ha eliminado un elemento
       *[other] Se han eliminado { $items } elementos
    } de forma permanente
removing-from-recents =
    Quitando { $items } { $items ->
        [one] elemento
       *[other] elementos
    } de { recents }
removed-from-recents =
    { $items ->
        [one] Se ha quitado elemento
       *[other] Se han quitado { $items } elementos
    } de { recents }
reload-folder = Recargar la carpeta
remove-from-recents = Quitar de recientes
calculating = Calculando...
type = Tipo: { $mime }
items = Elementos: { $items }
item-size = Tamaños: { $size }
item-created = Creado: { $created }
item-modified = Modificado: { $modified }
item-accessed = Accedido: { $accessed }
single-click = Abrir con un solo clic
type-to-search = Escribir para buscar
type-to-search-recursive = Buscar en la carpeta actual y todas las subcarpetas
type-to-search-enter-path = Introducir la ruta al directorio o archivo
delete-permanently = Eliminar de forma permanente
eject = Expulsar
