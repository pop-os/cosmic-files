cosmic-files = Fitxers del COSMIC
empty-folder = Carpeta buida
empty-folder-hidden = Carpeta buida (té elements ocults)
no-results = No s'ha trobat cap resultat
filesystem = Sistema de fitxers
home = Inici
networks = Xarxes
notification-in-progress = Hi ha operacions amb fitxers en curs.
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
        [one] operacion
       *[other] operacions
    } en curs ({ $percent }%)...
operations-running-finished =
    { $running }{ $running ->
        [one] operacion
       *[other] operacions
    } en curs ({ $percent }%), { $finished } acabat...
pause = Pausa
resume = Reprèn

# Dialogs


## Compress Dialog

create-archive = Crea un arxiu

## Extract Dialog

extract-password-required = Cal una contrasenya

## Empty Trash Dialog

empty-trash = Buida la paperera
empty-trash-warning = Voleu suprimir permanentment tots els fitxers de la paperera?

## Mount Error Dialog

mount-error = No es pot accedir a la unitat

## New File/Folder Dialog

create-new-file = Crea un nou fitxer
create-new-folder = Crea una nova carpeta
file-name = Nom del fixer
folder-name = Nom de la carpeta
file-already-exists = Ja existeix un fitxer amb aquest nom.
folder-already-exists = Ja existeix una carpeta amb aquest nom.
name-hidden = Els noms que comencin amb "." seran ocults.
name-invalid = El nom no pot ser "{ $filename }".
name-no-slashes = El nom no pot contenir barres.

## Open/Save Dialog

cancel = Cancel·la
create = Crea
open = Obre
open-file = Obre el fixer
open-folder = Obre la carpeta
open-in-new-tab = Obre en una pestanya nova
open-in-new-window = Obre en una finestra nova
open-item-location = Obre la ubicació del fitxer
open-multiple-files = Obre múltiples fitxers
open-multiple-folders = Obre múltiples carpetes
save = Desa
save-file = Desa el fitxer

## Open With Dialog

open-with-title = Com voleu obrir "{ $name }"?
browse-store = Navega { $store }

## Rename Dialog

rename-file = Canvia el nom del fitxer
rename-folder = Canvia el nom de la carpeta

## Replace Dialog

replace = Reemplaça
replace-title = Ja existeix "{ $filename }" en aquesta ubicació.
replace-warning = Voleu reemplaçar-lo pel fitxer que esteu desant? El seu contingut serà sobreescrit.
replace-warning-operation = Voleu reemplaçar-lo? El seu contingut serà sobreescrit.
original-file = Fitxer original
replace-with = Reemplaça amb
apply-to-all = Aplica-ho a tot
keep-both = Mantén els dos
skip = Omet

## Set as Executable and Launch Dialog

set-executable-and-launch = Defineix com a executable i executa
set-executable-and-launch-description = Voleu definir "{ $name }" com a executable i executar-lo?
set-and-launch = Defineix i executa

## Metadata Dialog

open-with = Obre amb
owner = Propietari
group = Grup
other = Altre

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

git-description = Git commit { $hash } el { $date }

## Add Network Drive

add-network-drive = Afegeix una unitat de la xarxa
connect = Connecta
connect-anonymously = Connecta anònimament
connecting = Connectant...
domain = Domini
enter-server-address = Introduïu l'adreça del servidor
network-drive-description =
    Les adreces de servidor estan formades per un prefix del protocol i una adreça.
    Examples: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Protocols disponibles,Prefix
    AppleTalk,afp://
    File Transfer Protocol,ftp:// or ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// or ssh://
    WebDav,dav:// or davs://
network-drive-error = No s'ha pogut accedir a la unitat de la xarxa
password = Contrasenya
remember-password = Recorda la contrasenya
try-again = Torna-ho a provar
username = Nom d'usuari

## Operations

cancelled = Cancel·lat
edit-history = Edita l'historial
history = Historial
no-history = Historial buit.
pending = Pendent
progress = { $percent }%
progress-cancelled = { $percent }%, cancel·lat
progress-paused = { $percent }%, en pausa
failed = Ha fallat
complete = Complet
compressing =
    { $items ->
        [one] S'està comprimint { $items } element
       *[other] S'estan comprimint { $items } elements
    } de "{ $from }" a "{ $to }" ({ $progress })...
compressed =
    { $items ->
        [one] S'ha comprimit { $items } element
       *[other] S'han comprimit { $items } elements
    } de "{ $from }" a "{ $to }"
copy_noun = Copia
creating = S'està creant "{ $name }" a "{ $parent }"
created = S'ha creat "{ $name }" a "{ $parent }"
copying =
    { $items ->
        [one] S'està copiant { $items } element
       *[other] S'estan copiant { $items } elements
    } de "{ $from }" a "{ $to }" ({ $progress })...
copied =
    { $items ->
        [one] S'ha copiat { $items } element
       *[other] S'han copiat { $items } elements
    } de "{ $from }" a "{ $to }"
emptying-trash = S'està buidant { trash } ({ $progress })...
emptied-trash = S'ha buidat { trash }
extracting =
    { $items ->
        [one] S'està extraient { $items } element
       *[other] S'estan extraient { $items } elements
    } de "{ $from }" a "{ $to }" ({ $progress })...
extracted =
    { $items ->
        [one] S'ha extret { $items } element
       *[other] S'han extret { $items } elements
    } de "{ $from }" a "{ $to }"
setting-executable-and-launching = S'està definint "{ $name }" com a executable i executant
set-executable-and-launched = S'ha definit "{ $name }" com a executable i executat
moving =
    { $items ->
        [one] S'està movent { $items } element
       *[other] S'estan movent { $items } elements
    } de "{ $from }" a "{ $to }" ({ $progress })...
moved =
    { $items ->
        [one] S'ha mogut { $items } element
       *[other] S'han mogut { $items } elements
    } de "{ $from }" a "{ $to }"
renaming = S'està canviant el nom de "{ $from }" a "{ $to }"
renamed = S'ha canviat el nom de "{ $from }" a "{ $to }"
restoring =
    { $items ->
        [one] S'està restaurant { $items } element
       *[other] S'estan restaurant { $items } elements
    } de { trash } ({ $progress })...
restored =
    { $items ->
        [one] S'ha restaurat { $items } element
       *[other] S'han restaurat { $items } elements
    } de { trash }
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
match-desktop = Coincideix amb l'escriptori
dark = Fosc
light = Clar
# Context menu
add-to-sidebar = Afegeix a la barra lateral
compress = Comprimeix
extract-here = Extreu
new-file = Nou fitxer...
new-folder = Nova carpeta...
open-in-terminal = Obre al terminal
move-to-trash = Mou a la paperera
restore-from-trash = Restaura de la paperera
remove-from-sidebar = Elimina de la barra lateral
sort-by-name = Ordena per nom
sort-by-modified = Ordena per data de modificació
sort-by-size = Ordena per mida
sort-by-trashed = Ordena per data de supressió

## Desktop

change-wallpaper = Canvia el fons de pantalla...
desktop-appearance = Aparença de l'escriptori...
display-settings = Configuració de visualització...

# Menu


## File

file = Fitxer
new-tab = Pestanya nova
new-window = Finestra nova
rename = Canvia el nom...
close-tab = Tanca la pestanya
quit = Surt

## Edit

edit = Edita
cut = Retalla
copy = Copia
paste = Enganxa
select-all = Selecciona-ho tot

## View

zoom-in = Amplia
default-size = Mida predeterminada
zoom-out = Redueix
view = Visualització
grid-view = Vista de graella
list-view = Vista de llista
show-hidden-files = Mostra els fitxers ocults
list-directories-first = Mostra els directoris primer
gallery-preview = Vista prèvia en galeria
menu-settings = Configuració...
menu-about = Quant a Fitxers del COSMIC...

## Sort

sort = Ordena
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Primer més recents
sort-oldest-first = Primer més antics
sort-smallest-to-largest = De petit a gran
sort-largest-to-smallest = De gran a petit
