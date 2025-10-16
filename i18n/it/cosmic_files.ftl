cosmic-files = COSMIC Files
empty-folder = Cartella vuota
empty-folder-hidden = Cartella vuota (con elementi nascosti)
no-results = Nessun risultato trovato
filesystem = Filesystem
home = Home
networks = Reti
notification-in-progress = Operazioni sui file in corso.
trash = Cestino
recents = Recenti
undo = Annulla
today = Oggi
# Desktop view options
desktop-view-options = Impostazioni visualizzazione Desktop...
show-on-desktop = Mostra sul Desktop
desktop-folder-content = Contenuto cartella del Desktop
mounted-drives = Dispositivi montati
trash-folder-icon = Icona del cestino
icon-size-and-spacing = Dimensioni e spaziatura icona
icon-size = Dimensione icona
grid-spacing = Spaziatura griglia
# List view
name = Nome
modified = Modificato
trashed-on = Cestinato
size = Dimensione
# Progress footer
details = Dettagli
dismiss = Nascondi messaggio
operations-running =
    { $running } { $running ->
        [one] operazione
       *[other] operazioni
    } in corso ({ $percent }%)...
operations-running-finished =
    { $running } { $running ->
        [one] operazione
       *[other] operazioni
    } in corso ({ $percent }%), { $finished } completata...
pause = Pausa
resume = Riprendi

# Dialogs


## Compress Dialog

create-archive = Crea archivio

## Extract Dialog

extract-password-required = Password richiesta
extract-to = Estrai in...
extract-to-title = Estrai nella cartella

## Empty Trash Dialog

empty-trash = Svuota cestino
empty-trash-warning = Sei sicuro di voler eliminare definitivamente tutti gli elementi nel cestino?

## Mount Error Dialog

mount-error = Impossibile accedere al dispositivo

## New File/Folder Dialog

create-new-file = Crea un nuovo file
create-new-folder = Crea una nuova cartella
file-name = Nome file
folder-name = Nome cartella
file-already-exists = Esiste già un file con questo nome.
folder-already-exists = Esiste già una cartella con questo nome.
name-hidden = I nomi che iniziano con "." verranno nascosti.
name-invalid = Il nome non può essere "{ $filename }".
name-no-slashes = I nomi non possono contenere gli slash.

## Open/Save Dialog

cancel = Annulla
create = Crea
open = Apri
open-file = Apri file
open-folder = Apri cartella
open-in-new-tab = Apri in una nuova scheda
open-in-new-window = Apri in una nuova finestra
open-item-location = Apri percorso file
open-multiple-files = Apri files multipli
open-multiple-folders = Apri cartelle multiple
save = Salva
save-file = Salva file

## Open With Dialog

open-with-title = Come vuoi aprire il file "{ $name }"?
browse-store = Cerca in { $store }
other-apps = Altre applicazioni
related-apps = Applicazioni simili

## Permanently delete Dialog

selected-items = i { $items } elementi selezionati
permanently-delete-question = Elimina definitivamente
delete = Elimina
permanently-delete-warning = Sei sicuro di voler eliminare definitivamente { $target }? Questa azione non può essere annullata.

## Rename Dialog

rename-file = Rinomina file
rename-folder = Rinomina cartella

## Replace Dialog

replace = Sostituisci
replace-title = "{ $filename }" esiste già in questo percorso.
replace-warning = Vuoi sostituirlo con quello che stai per salvare? La sostituzione sovrascriverà il suo contenuto.
replace-warning-operation = Vuoi sostituirlo? La sostituzione sovrascriverà il suo contenuto.
original-file = File originale
replace-with = Sostituisci con
apply-to-all = Applica a tutti
keep-both = Mantieni entrambi
skip = Salta

## Set as Executable and Launch Dialog

set-executable-and-launch = Imposta come "eseguibile" e apri
set-executable-and-launch-description = Vuoi impostare "{ $name }" come "eseguibile" e aprirlo?
set-and-launch = Imposta e apri

## Metadata Dialog

open-with = Apri con
owner = Proprietario
group = Gruppo
other = Altro

### Mode 0

none = Nessuno

### Mode 1 (unusual)

execute-only = Sola esecuzione

### Mode 2 (unusual)

write-only = Sola scrittura

### Mode 3 (unusual)

write-execute = Scrittura ed esecuzione

### Mode 4

read-only = Sola lettura

### Mode 5

read-execute = Lettura e esecuzione

### Mode 6

read-write = Lettura e scrittura

### Mode 7

read-write-execute = Lettura, scrittura ed esecuzione

## Favorite Path Error Dialog

favorite-path-error = Errore nell'apertura della cartella
favorite-path-error-description =
    Impossibile aprire "{ $path }".
    Potrebbe non esistere o potresti non avere i permessi di accesso.

    Vuoi rimuoverla dalla barra laterale?
remove = Rimuovi
keep = Mantieni

# Context Pages


## About

repository = Repository
support = Supporto

## Add Network Drive

add-network-drive = Aggiungi dispositivo di rete
connect = Connetti
connect-anonymously = Connetti in modo anonimo
connecting = Connessione in corso...
domain = Dominio
enter-server-address = Inserisci indirizzo di rete
network-drive-description =
    Gli indirizzi dei server includono il prefisso del protocollo e l'indirizzo.
    Esempi: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Protocolli disponibili,Prefisso
    AppleTalk,afp://
    File Transfer Protocol,ftp:// or ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// or ssh://
    WebDav,dav:// or davs://
network-drive-error = Impossibile accedere al dispositivo di rete
password = Password
remember-password = Ricorda password
try-again = Riprova
username = Nome utente

## Operations

cancelled = Annullato
edit-history = Modifica cronologia
history = Cronologia
no-history = Nessun elemento nella cronologia.
pending = In coda
progress = { $percent }%
progress-cancelled = { $percent }%, annullato
progress-paused = { $percent }%, in pausa
failed = Fallito
complete = Completato
compressing =
    Compressione in corso di { $items } { $items ->
        [one] elemento
       *[other] elementi
    } da "{ $from }" a "{ $to }" ({ $progress })...
compressed =
    Compressi { $items } { $items ->
        [one] elemento
       *[other] elementi
    } da "{ $from }" a "{ $to }"
copy_noun = Copia
creating = Creazione "{ $name }" in "{ $parent }"
created = Creato "{ $name }" in "{ $parent }"
copying =
    Copia in corso di { $items } { $items ->
        [one] elemento
       *[other] elementi
    } da "{ $from }" a "{ $to }" ({ $progress })...
copied =
    Copiati { $items } { $items ->
        [one] elemento
       *[other] elementi
    } da "{ $from }" a "{ $to }"
deleting =
    Eliminazione in corso di { $items } { $items ->
        [one] elemento
       *[other] elementi
    } dal { trash }: ({ $progress })...
deleted =
    Eliminati { $items } { $items ->
        [one] elemento
       *[other] elementi
    } dal { trash }
emptying-trash = Svuotamento del { trash }: ({ $progress })...
emptied-trash = { trash } svuotato
extracting =
    Estrazione in corso di { $items } { $items ->
        [one] elemento
       *[other] elementi
    } da "{ $from }" a "{ $to }": ({ $progress })...
extracted =
    Estratti { $items } { $items ->
        [one] elemento
       *[other] elementi
    } da "{ $from }" a "{ $to }"
setting-executable-and-launching = Impostazione in corso di "{ $name }" come "eseguibile" e avvio
set-executable-and-launched = Impostato "{ $name }" come "eseguibile" e avviato
moving =
    Spostamento in corso di { $items } { $items ->
        [one] elemento
       *[other] elementi
    } da "{ $from }" a "{ $to }": ({ $progress })...
moved =
    Spostati { $items } { $items ->
        [one] elemento
       *[other] elementi
    } da "{ $from }" a "{ $to }"
renaming = Rinominazione di "{ $from }" in "{ $to }"
renamed = Rinominato "{ $from }" in "{ $to }"
restoring =
    Ripristino in corso di { $items } { $items ->
        [one] elemento
       *[other] elementi
    } dal { trash }: ({ $progress })...
restored =
    Ripristinati { $items } { $items ->
        [one] elemento
       *[other] elementi
    } dal { trash }
unknown-folder = cartella sconosciuta

## Open with

menu-open-with = Apri con...
default-app = { $name } (predefinito)

## Show details

show-details = Mostra dettagli
type = Tipo: { $mime }
items = Files: { $items }
item-size = Dimensione: { $size }
item-created = Creato: { $created }
item-modified = Modificato in data: { $modified }
item-accessed = Accesso eseguito in data: { $accessed }
calculating = Calcolo in corso...

## Settings

settings = Impostazioni
single-click = Click singolo per aprire

### Appearance

appearance = Aspetto
theme = Tema
match-desktop = Sistema
dark = Scuro
light = Chiaro

### Type to Search

type-to-search = Digita per cercare
type-to-search-recursive = Cerca nella cartella attuale e nelle sue sotto-cartelle
type-to-search-enter-path = Inserisci il percorso della cartella o del file
# Context menu
add-to-sidebar = Aggiungi alla barra laterale
compress = Comprimi
delete-permanently = Eliminazione definitiva
eject = Espelli
extract-here = Estrai
new-file = Nuovo file...
new-folder = Nuova cartella...
open-in-terminal = Apri nel terminale
move-to-trash = Sposta nel cestino
restore-from-trash = Ripristina dal cestino
remove-from-sidebar = Rimuovi dalla barra laterale
sort-by-name = Ordina per nome
sort-by-modified = Ordina per data di modifica
sort-by-size = Ordina per dimensione
sort-by-trashed = Ordina per data di eliminazione
remove-from-recents = Rimuovi da recenti

## Desktop

change-wallpaper = Modifica sfondo...
desktop-appearance = Aspetto del Desktop...
display-settings = Impostazioni del display...

# Menu


## File

file = File
new-tab = Nuova scheda
new-window = Nuova finestra
reload-folder = Aggiorna cartella
rename = Rinomina...
close-tab = Chiudi scheda
quit = Esci

## Edit

edit = Modifica
cut = Taglia
copy = Copia
paste = Incolla
select-all = Seleziona tutto

## View

zoom-in = Aumenta zoom
default-size = Dimensione predefinita
zoom-out = Diminuisci zoom
view = Visualizza
grid-view = Visualizzazione a griglia
list-view = Visualizzazione a elenco
show-hidden-files = Mostra file nascosti
list-directories-first = Mostra prima le cartelle
gallery-preview = Anteprima immagine
menu-settings = Impostazioni...
menu-about = Informazioni su COSMIC Files...

## Sort

sort = Ordina
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Prima i più recenti
sort-oldest-first = Prima i più vecchi
sort-smallest-to-largest = Dal più piccolo al più grande
sort-largest-to-smallest = Dal più grande al più piccolo
progress-failed = { $percent }%, fallito
setting-permissions = Impostazione dei permessi per "{ $name }" su { $mode }
set-permissions = Permessi impostati per "{ $name }" su { $mode }
permanently-deleting =
    Eliminazione definitva in corso di { $items } { $items ->
        [one] elemento
       *[other] elementi
    }
permanently-deleted =
    Eliminazione definitivamente { $items } { $items ->
        [one] elemento
       *[other] elementi
    }
removing-from-recents =
    Rimozione in corso di { $items } { $items ->
        [one] elemento
       *[other] elementi
    } da { recents }
removed-from-recents =
    Rimossi { $items } { $items ->
        [one] elemento
       *[other] elementi
    } da { recents }
