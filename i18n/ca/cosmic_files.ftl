cosmic-files = Fitxers del COSMIC
empty-folder = Carpeta buida
empty-folder-hidden = Carpeta buida (conté elements ocults)
no-results = No s'ha trobat cap resultat
filesystem = Sistema de fitxers
home = Inici
networks = Xarxes
notification-in-progress = Operacions de fitxers en curs
trash = Paperera
recents = Recents
undo = Desfés
today = Avui
# Desktop view options
desktop-view-options = Opcions de visualització de l'escriptori
show-on-desktop = Mostra a l'escriptori
desktop-folder-content = Contingut de la carpeta de l'escriptori
mounted-drives = Unitats muntades
trash-folder-icon = Icona de la paperera
icon-size-and-spacing = Mida i espaiat de les icones
icon-size = Mida de les icones
grid-spacing = Espaiat de la quadrícula
# List view
name = Nom
modified = Modificat
trashed-on = Mogut a la paperera
size = Mida
# Progress footer
details = Detalls
dismiss = Descarta el missatge
operations-running =
    { $running } { $running ->
        [one] operació
       *[other] operacions
    } en curs ({ $percent }%)...
operations-running-finished =
    { $running }{ $running ->
        [one] operació
       *[other] operacions
    } en curs ({ $percent }%), { $finished } acabada/es...
pause = Pausa
resume = Reprèn

# Dialogs


## Compress Dialog

create-archive = Crea un arxiu

## Extract Dialog

extract-password-required = Cal una contrasenya

## Empty Trash Dialog

empty-trash = Buida la paperera
empty-trash-warning = Els elements dins la Paperera s'eliminaran permanentment

## Mount Error Dialog

mount-error = No es pot accedir a la unitat

## New File/Folder Dialog

create-new-file = Crea un nou fitxer
create-new-folder = Crea una nova carpeta
file-name = Nom del fitxer
folder-name = Nom de la carpeta
file-already-exists = Ja existeix un fitxer amb aquest nom
folder-already-exists = Ja existeix una carpeta amb aquest nom
name-hidden = Els noms que comencin amb "." s'ocultaran.
name-invalid = El nom no pot ser "{ $filename }"
name-no-slashes = El nom no pot contenir barres inclinades

## Open/Save Dialog

cancel = Cancel·lar
create = Crea
open = Obre
open-file = Obre el fitxer
open-folder = Obre la carpeta
open-in-new-tab = Obre a una pestanya nova
open-in-new-window = Obre a una finestra nova
open-item-location = Obre la ubicació de l'element
open-multiple-files = Obre múltiples fitxers
open-multiple-folders = Obre múltiples carpetes
save = Desa
save-file = Desa el fitxer

## Open With Dialog

open-with-title = Com voleu obrir «{ $name }»?
browse-store = Navega { $store }

## Rename Dialog

rename-file = Reanomena el fitxer
rename-folder = Reanomena la carpeta

## Replace Dialog

replace = Substitueix
replace-title = «{ $filename }» ja existeix aquí
replace-warning = Voleu substituir-lo amb el fitxer que esteu desant? Substituir el fitxer sobreescriurà el seu contingut.
replace-warning-operation = Voleu reemplaçar-lo? Reemplaçar el fitxer sobreescriurà el seu contingut.
original-file = Fitxer original
replace-with = Reemplaça amb
apply-to-all = Aplica-ho a tot
keep-both = Conserva ambdós
skip = Omet

## Set as Executable and Launch Dialog

set-executable-and-launch = Marca com a executable i executa
set-executable-and-launch-description = Voleu marcar «{ $name }» com a executable i executar-lo?
set-and-launch = Marca i executa

## Metadata Dialog

open-with = Obre amb
owner = Propietari
group = Grup
other = Altres

### Mode 0

none = Cap

### Mode 1 (unusual)

execute-only = Només executar

### Mode 2 (unusual)

write-only = Només escriure

### Mode 3 (unusual)

write-execute = Escriure i executar

### Mode 4

read-only = Només llegir

### Mode 5

read-execute = Llegir i executar

### Mode 6

read-write = Llegir i escriure

### Mode 7

read-write-execute = Llegir, escriure i executar

# Context Pages


## About


## Add Network Drive

add-network-drive = Afegeix una unitat de xarxa
connect = Connecta
connect-anonymously = Connecta anònimament
connecting = Connectant...
domain = Domini
enter-server-address = Introduïu l'adreça del servidor
network-drive-description =
    Les adreces de servidor inclouen un prefix de protocol i una adreça.
    Exemples: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Protocols disponibles,Prefix
    AppleTalk,afp://
    File Transfer Protocol,ftp:// o ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// o ssh://
    WebDav,dav:// o davs://
network-drive-error = No s'ha pogut accedir a la unitat de xarxa
password = Contrasenya
remember-password = Recorda la contrasenya
try-again = Torna-ho a intentar
username = Nom d'usuari

## Operations

cancelled = Cancel·lades
edit-history = Historial d'edicions
history = Historial
no-history = Cap ítem a l'historial.
pending = Pendents
progress = { $percent } %
progress-cancelled = { $percent } %, cancel·lat
progress-paused = { $percent } %, en pausa
failed = Fallides
complete = Completades
compressing =
    { $items ->
        [one] S'està comprimint { $items } element
       *[other] S'estan comprimint { $items } elements
    } de «{ $from }» a «{ $to }» ({ $progress })...
compressed =
    { $items ->
        [one] S'ha comprimit { $items } element
       *[other] S'han comprimit { $items } elements
    } de «{ $from }» a «{ $to }»
copy_noun = Copia
creating = S'està creant «{ $name }» a «{ $parent }»
created = S'ha creat «{ $name }» a «{ $parent }»
copying =
    { $items ->
        [one] S'està copiant { $items } element
       *[other] S'estan copiant { $items } elements
    } de «{ $from }» a «{ $to }» ({ $progress })...
copied =
    { $items ->
        [one] S'ha copiat { $items } element
       *[other] S'han copiat { $items } elements
    } de «{ $from }» a «{ $to }»
emptying-trash = S'està buidant la { trash } ({ $progress })...
emptied-trash = S'ha buidat la { trash }
extracting =
    { $items ->
        [one] S'està extraient { $items } element
       *[other] S'estan extraient { $items } elements
    } de «{ $from }» a «{ $to }» ({ $progress })...
extracted =
    { $items ->
        [one] S'ha extret { $items } element
       *[other] S'han extret { $items } elements
    } de «{ $from }» a «{ $to }»
setting-executable-and-launching = S'està marcant «{ $name }» com a executable i executant
set-executable-and-launched = S'ha marcat «{ $name }» com a executable i s'ha executat
moving =
    { $items ->
        [one] S'està movent { $items } element
       *[other] S'estan movent { $items } elements
    } de «{ $from }» a «{ $to }» ({ $progress })...
moved =
    { $items ->
        [one] S'ha mogut { $items } element
       *[other] S'han mogut { $items } elements
    } de «{ $from }» a «{ $to }»
renaming = S'està reanomenant «{ $from }» a «{ $to }»
renamed = S'ha reanomenat «{ $from }» a «{ $to }»
restoring =
    { $items ->
        [one] S'està restaurant { $items } element
       *[other] S'estan restaurant { $items } elements
    } de la { trash } ({ $progress })...
restored =
    { $items ->
        [one] S'ha restaurat { $items } element
       *[other] S'han restaurat { $items } elements
    } de la { trash }
unknown-folder = carpeta desconeguda

## Open with

menu-open-with = Obre amb...
default-app = { $name } (per defecte)

## Show details

show-details = Mostra els detalls
type = Tipus: { $mime }
items = Elements: { $items }
item-size = Mida: { $size }
item-created = Creat: { $created }
item-modified = Modificat: { $modified }
item-accessed = Accedit: { $accessed }
calculating = S'està calculant...

## Settings

settings = Configuració

### Appearance

appearance = Aparença
theme = Tema
match-desktop = Igual que el sistema
dark = Fosc
light = Clar
# Context menu
add-to-sidebar = Afegeix a la barra lateral
compress = Comprimeix...
extract-here = Extreu
new-file = Nou fitxer...
new-folder = Nova carpeta...
open-in-terminal = Obre al terminal
move-to-trash = Mou a la paperera
restore-from-trash = Restaura des de la paperera
remove-from-sidebar = Treu de la barra lateral
sort-by-name = Ordena per nom
sort-by-modified = Ordena per data de modificació
sort-by-size = Ordena per mida
sort-by-trashed = Ordena per data de supressió

## Desktop

change-wallpaper = Canvia el fons de pantalla...
desktop-appearance = Aparença de l'escriptori...
display-settings = Configuració de pantalla...

# Menu


## File

file = Fitxer
new-tab = Nova pestanya
new-window = Nova finestra
rename = Reanomena...
close-tab = Tanca la pestanya
quit = Surt

## Edit

edit = Edita
cut = Retalla
copy = Copia
paste = Enganxa
select-all = Selecciona‐ho tot

## View

zoom-in = Amplia
default-size = Mida predeterminada
zoom-out = Redueix
view = Visualització
grid-view = Vista de graella
list-view = Vista de llista
show-hidden-files = Mostra els fitxers ocults
list-directories-first = Mostra les carpetes primer
gallery-preview = Vista prèvia en galeria
menu-settings = Configuració...
menu-about = Quant a Fitxers del COSMIC...

## Sort

sort = Ordena
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Més recents primer
sort-oldest-first = Més antics primer
sort-smallest-to-largest = De petit a gran
sort-largest-to-smallest = De gran a petit
run = Executa
extract-to-title = Extreu a una carpeta
empty-trash-title = Voleu buidar la paperera?
extract-to = Extreu a...
delete = Suprimeix
comment = Gestor de fitxers de l'escriptori COSMIC
copy-to-title = Escolliu el destí de la còpia
copy-to-button-label = Copia
move-to-title = Escolliu el destí del trasllat
move-to-button-label = Mou
keywords = Carpeta;Fitxers;Arxius;Gestor;Explorador;
related-apps = Aplicacions relacionades
permanently-delete-warning = { $target } se suprimirà permanentment. Aquesta operació és irreversible.
other-apps = Altres aplicacions
selected-items = Els { $items } elements seleccionats
context-action-confirm-title = Executar «{ $name }»?
context-action-confirm-warning =
    Aquesta operació s'executarà sobre { $items } { $items ->
        [one] ítem
       *[other] ítems
    }.
permanently-delete-question = Suprimir permanentment?
context-action = Acció contextual
sidebar-label = Etiqueta de la barra lateral
favorite-path-error = Error en obrir la carpeta
type-to-search-enter-path = Introdueix el camí a un directori o fitxer
favorite-path-error-description =
    No es pot obrir «{ $path }».
    «{ $path }» potser no existeix o podríeu no tenir permisos per obrir‐lo

    Voleu suprimir‐lo de la barra lateral?
support = Suport
eject = Expulsa
type-to-search = Escriure per a cercar
type-to-search-recursive = Cerca a la carpeta actual i totes les seves subcarpetes
delete-permanently = Suprimeix permanentment
deleted =
    { $items ->
        [one] S'ha suprimit { $items } element
       *[other] S'han suprimit { $items } elements
    } de la { trash }
reload-folder = Recarrega la carpeta
removing-from-recents =
    { $items ->
        [one] S'està traient { $items } element
       *[other] S'estan traient { $items } elements
    } de { recents }
set-permissions = S'han establert els permisos per a «{ $name }» a { $mode }
keep = Manté
permanently-deleting =
    { $items ->
        [one] S'està suprimint { $items } element
       *[other] S'estan suprimint { $items } elements
    } permanentment
repository = Repositori
remove-from-recents = Treu dels recents
deleting =
    { $items ->
        [one] S'està suprimint { $items } fitxer
       *[other] S'estan suprimint { $items } fitxers
    } de la { trash } ({ $progress })...
single-click = Obre amb un sol clic
setting-permissions = Establint els permisos per a «{ $name }» a { $mode }
removed-from-recents =
    { $items ->
        [one] S'ha tret { $items } element
       *[other] S'han tret { $items } elements
    } de { recents }
progress-failed = { $percent } %, ha fallat
permanently-deleted =
    { $items ->
        [one] S'ha suprimit { $items } element
       *[other] S'han suprimit { $items } elements
    } permanentment
remove = Elimina
pasted-image = Imatge Enganxada
pasted-text = Text Enganxat
pasted-video = Vídeo Enganxat
show-recents = Carpetes recents a la barra lateral
type-to-search-select = Selecciona el primer fitxer o carpeta coincident
clear-recents-history = Neteja l'historial de Recents
copy-to = Copia a...
move-to = Mou a...
copy-path = Copia el camí
rename-confirm = Reanomena
mixed = Mixt
checksum = Suma de verificació { $kind }
calculate = Calcula
error = Error
search-application = Cerca per nom de l'aplicació
change-sidebar-label = Canvia l'etiqueta de la barra lateral
