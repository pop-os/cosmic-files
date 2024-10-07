cosmic-files = Fișiere COSMIC
empty-folder = Dosar gol
empty-folder-hidden = Dosar gol (conține elemente ascunse)
no-results = Niciun rezultat găsit
filesystem = Sistem de fișiere
home = Acasă
networks = Rețele
notification-in-progress = Operațiuni de fișiere în desfășurare.
trash = Coș de gunoi
recents = Recente
undo = Anulează
today = Astăzi

# Desktop view options
desktop-view-options = Opțiuni de vizualizare pe desktop...
show-on-desktop = Afișează pe desktop
desktop-folder-content = Conținutul dosarului de pe desktop
mounted-drives = Drive-uri montate
trash-folder-icon = Pictogramă dosar coș de gunoi
icon-size-and-spacing = Dimensiunea și spațierea pictogramelor
icon-size = Dimensiunea pictogramei

# List view
name = Nume
modified = Modificat
trashed-on = În coș de gunoi
size = Dimensiune

# Dialogs

## Compress Dialog
create-archive = Creează o arhivă

## Empty Trash Dialog
empty-trash = Golește coșul de gunoi
empty-trash-warning = Sigur dorești să ștergi definitiv toate elementele din coșul de gunoi?

## New File/Folder Dialog
create-new-file = Creează fișier nou
create-new-folder = Creează dosar nou
file-name = Nume fișier
folder-name = Nume dosar
file-already-exists = Un fișier cu acest nume există deja.
folder-already-exists = Un dosar cu acest nume există deja.
name-hidden = Numele care încep cu "." vor fi ascunse.
name-invalid = Numele nu poate fi "{$filename}".
name-no-slashes = Numele nu poate conține bare oblice (/).

## Open/Save Dialog
cancel = Anulează
create = Creează
open = Deschide
open-file = Deschide fișier
open-folder = Deschide dosar
open-in-new-tab = Deschide într-o filă nouă
open-in-new-window = Deschide într-o fereastră nouă
open-item-location = Deschide locația elementului
open-multiple-files = Deschide mai multe fișiere
open-multiple-folders = Deschide mai multe dosare
save = Salvează
save-file = Salvează fișierul

## Open With Dialog
open-with-title = Cum dorești să deschizi "{$name}"?
browse-store = Caută în {$store}

## Rename Dialog
rename-file = Redenumește fișier
rename-folder = Redenumește dosar

## Replace Dialog
replace = Înlocuiește
replace-title = {$filename} există deja în această locație.
replace-warning = Dorești să-l înlocuiești cu cel pe care îl salvezi? Înlocuirea va suprascrie conținutul său.
replace-warning-operation = Doriți să-l înlocuiți? Înlocuirea va suprascrie conținutul său.
original-file = Fișier original
replace-with = Înlocuiește cu
apply-to-all = Aplică la toate
keep-both = Păstrează ambele
skip = Sari

## Set as Executable and Launch Dialog
set-executable-and-launch = Setează ca executabil și lansează
set-executable-and-launch-description = Dorești să setezi "{$name}" ca executabil și să-l lansezi?
set-and-launch = Setează și lansează

## Metadata Dialog
owner = Proprietar
group = Grup
other = Alții
read = Citire
write = Scriere
execute = Executare

# Context Pages

## About
git-description = Commit Git {$hash} pe {$date}

## Add Network Drive
add-network-drive = Adaugă drive de rețea
connect = Conectează
connect-anonymously = Conectează anonim
connecting = Conectare...
domain = Domeniu
enter-server-address = Introdu adresa serverului
network-drive-description =
    Adresele serverului includ un prefix de protocol și adresa.
    Exemple: ssh://192.168.0.1, ftp://[2001:db8::1]
### Make sure to keep the comma which separates the columns
network-drive-schemes =
    Protocoale disponibile,Prefix
    AppleTalk,afp://
    File Transfer Protocol,ftp:// sau ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// sau ssh://
    WebDav,dav:// sau davs://
network-drive-error = Nu se poate accesa drive-ul de rețea
password = Parolă
remember-password = Ține minte parola
try-again = Încearcă din nou
username = Nume utilizator

## Operations
edit-history = Editare istoric
history = Istoric
no-history = Nu există elemente în istoric.
pending = În așteptare
failed = Eșuat
complete = Complet
compressing = Se comprimă {$items} {$items ->
        [one] element
        *[other] elemente
    } din {$from} în {$to}
compressed = Comprimat {$items} {$items ->
        [one] element
        *[other] elemente
    } din {$from} în {$to}
copy_noun = Copie
creating = Se creează {$name} în {$parent}
created = Creat {$name} în {$parent}
copying = Se copiază {$items} {$items ->
        [one] element
        *[other] elemente
    } din {$from} în {$to}
copied = Copiat {$items} {$items ->
        [one] element
        *[other] elemente
    } din {$from} în {$to}
emptying-trash = Se golește {trash}
emptied-trash = Coșul de gunoi golit
extracting = Se extrag {$items} {$items ->
        [one] element
        *[other] elemente
    } din {$from} în {$to}
extracted = Extragerea completă pentru {$items} {$items ->
        [one] element
        *[other] elemente
    } din {$from} în {$to}
setting-executable-and-launching = Se setează "{$name}" ca executabil și se lansează
set-executable-and-launched = "{$name}" setat ca executabil și lansat
moving = Se mută {$items} {$items ->
        [one] element
        *[other] elemente
    } din {$from} în {$to}
moved = Mutat {$items} {$items ->
        [one] element
        *[other] elemente
    } din {$from} în {$to}
renaming = Se redenumește {$from} în {$to}
renamed = Redenumit {$from} în {$to}
restoring = Se restaurează {$items} {$items ->
        [one] element
        *[other] elemente
    } din {trash}
restored = Restaurat {$items} {$items ->
        [one] element
        *[other] elemente
    } din {trash}
unknown-folder = dosar necunoscut

## Open with
open-with = Deschide cu...
default-app = {$name} (implicit)

## Show details
show-details = Afișează detalii

## Settings
settings = Setări

### Appearance
appearance = Aspect
theme = Temă
match-desktop = Potrivește cu desktopul
dark = Întunecat
light = Deschis

# Context menu
add-to-sidebar = Adaugă în bara laterală
compress = Comprimă
extract-here = Extrage aici
new-file = Fișier nou...
new-folder = Dosar nou...
open-in-terminal = Deschide în terminal
move-to-trash = Mută în coșul de gunoi
restore-from-trash = Restaurează din coșul de gunoi
remove-from-sidebar = Elimină din bara laterală
sort-by-name = Sortează după nume
sort-by-modified = Sortează după modificare
sort-by-size = Sortează după dimensiune
sort-by-trashed = Sortează după ora ștergerii

## Desktop
change-wallpaper = Schimbă fundalul...
desktop-appearance = Aspectul desktopului...
display-settings = Setări afișaj...

# Menu

## File
file = Fișier
new-tab = Filă nouă
new-window = Fereastră nouă
rename = Redenumește...
close-tab = Închide fila
quit = Ieși

## Edit
edit = Editare
cut = Decupează
copy = Copiază
paste = Lipește
select-all = Selectează tot

## View
zoom-in = Mărește
default-size = Dimensiune implicită
zoom-out = Micșorează
view = Vizualizare
grid-view = Vizualizare grilă
list-view = Vizualizare listă
show-hidden-files = Afișează fișierele ascunse
list-directories-first = Listează directoarele primele
gallery-preview = Previzualizare galerie
menu-settings = Setări...
menu-about = Despre Fișiere COSMIC...

## Sort
sort = Sortează
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Cele mai noi primele
sort-oldest-first = Cele mai vechi primele
sort-smallest-to-largest = Cele mai mici la cele mai mari
sort-largest-to-smallest = Cele mai mari la cele mai mici