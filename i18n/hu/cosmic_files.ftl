cosmic-files = COSMIC Fájlok
empty-folder = Üres mappa
empty-folder-hidden = Üres mappa (Rejtett elemeket tartalmaz)
no-results = Nincs találat
filesystem = Fájlrendszer
home = Saját mappa
networks = Hálózatok
notification-in-progress = A fájlműveletek még folyamatban vannak.
trash = Kuka
recents = Legutóbbiak
undo = Visszavonás
today = Ma

# Desktop view options
desktop-view-options = Asztali nézet beállításai...
show-on-desktop = Megjelenítés az asztalon
desktop-folder-content = Asztal mappa tartalma
mounted-drives = Csatolt meghajtók
trash-folder-icon = Kuka ikonja
icon-size-and-spacing = Ikonméret és távolság
icon-size = Ikonméret
grid-spacing = Rácsköz

# List view
name = Név
modified = Módosítva
trashed-on = Kukába helyezve
size = Méret

# Progress footer
details = Részletek
dismiss = Üzenet elvetése
operations-running = {$running} művelet fut ({$percent}%)...
operations-running-finished = {$running} művelet fut ({$percent}%), {$finished} befejeződött...
pause = Szünet
resume = Folytatás

# Dialogs

## Compress Dialog
create-archive = Tömörített fájl létrehozása

## Extract Dialog
extract-password-required = Jelszó szükséges
extract-to = Kibontás ide...
extract-to-title = Kibontás mappába

## Empty Trash Dialog
empty-trash = Kuka ürítése
empty-trash-warning = Biztosan véglegesen törölni szeretné a kukában lévő összes elemet?

## Mount Error Dialog
mount-error = A meghajtó nem elérhető

## New File/Folder Dialog
create-new-file = Új fájl létrehozása
create-new-folder = Új mappa létrehozása
file-name = Fájlnév
folder-name = Mappanév
file-already-exists = Már létezik ilyen nevű fájl.
folder-already-exists = Már létezik ilyen nevű mappa.
name-hidden = A ponttal kezdődő nevek rejtve lesznek.
name-invalid = A név nem lehet "{$filename}".
name-no-slashes = A név nem tartalmazhat perjelet.

## Open/Save Dialog
cancel = Mégse
create = Létrehozás
open = Megnyitás
open-file = Fájl megnyitása
open-folder = Mappa megnyitása
open-in-new-tab = Megnyitás új lapon
open-in-new-window = Megnyitás új ablakban
open-item-location = Útvonal megnyitása
open-multiple-files = Több fájl megnyitása
open-multiple-folders = Több mappa megnyitása
save = Mentés
save-file = Fájl mentése

## Open With Dialog
open-with-title = Hogyan szeretné megnyitni ezt: "{$name}"?
browse-store = {$store} böngészése

## Rename Dialog
rename-file = Fájl átnevezése
rename-folder = Mappa átnevezése

## Replace Dialog
replace = Csere
replace-title = "{$filename}" már létezik.
replace-warning = Le szeretné cserélni a meglévő fájlt? A cseréje felülírja annak tartalmát.
replace-warning-operation = Ki szeretné cserélni? A csere felülírja annak tartalmát.
original-file = Eredeti fájl
replace-with = Csere erre
apply-to-all = Alkalmazás mindegyikre
keep-both = Mindkettő megtartása
skip = Kihagyás

## Set as Executable and Launch Dialog
set-executable-and-launch = Végrehajthatóvá tétel és indítás
set-executable-and-launch-description = Szeretné végrehajthatóvá tenni a(z) "{$name}" fájlt és elindítani?
set-and-launch = Alkalmazás és indítás

## Metadata Dialog
open-with = Megnyitás ezzel
owner = Tulajdonos
group = Csoport
other = Többi
### Mode 0
none = Nincs
### Mode 1 (unusual)
execute-only = Csak végrehajtás
### Mode 2 (unusual)
write-only = Csak írás
### Mode 3 (unusual)
write-execute = Írás és végrehajtás
### Mode 4
read-only = Csak olvasás
### Mode 5
read-execute = Olvasás és végrehajtás
### Mode 6
read-write = Olvasás és írás
### Mode 7
read-write-execute = Olvasás, írás és végrehajtás

## Favorite Path Error Dialog
favorite-path-error = Hiba a könyvtár megnyitásakor
favorite-path-error-description =
    Nem sikerült megnyitni: "{$path}".
    Lehet, hogy nem létezik, vagy nincs megfelelő jogosultsága a megnyitásához.
    
    Szeretné eltávolítani az oldalsávról?
remove = Eltávolítás
keep = Megtartás

# Context Pages

## About
git-description = Git commit {$hash} ekkor: {$date}

## Add Network Drive
add-network-drive = Hálózati meghajtó hozzáadása
connect = Csatlakozás
connect-anonymously = Csatlakozás névtelenül
connecting = Csatlakozás...
domain = Tartomány
enter-server-address = Add meg a szerver címét
network-drive-description =
    A szervercímek tartalmazzák a protokoll előtagot és a címet.
    Példák: ssh://192.168.0.1, ftp://[2001:db8::1]
### Make sure to keep the comma which separates the columns
network-drive-schemes =
    Elérhető protokollok,Előtag
    AppleTalk,afp://
    File Transfer Protocol,ftp:// vagy ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// vagy ssh://
    WebDav,dav:// vagy davs://
network-drive-error = A hálózati meghajtó nem elérhető
password = Jelszó
remember-password = Jelszó megjegyzése
try-again = Újra
username = Felhasználónév

## Operations
cancelled = Megszakítva
edit-history = Fájlműveleti előzmények
history = Előzmények
no-history = Nem találhatók elemek az előzményekben.
pending = Függőben
progress = {$percent}%
progress-cancelled = {$percent}%, megszakítva
progress-paused = {$percent}%, szüneteltetve
failed = Sikertelen
complete = Befejeződött
compressing = {$items} {$items ->
        [one] elem
        *[other] elem
    } tömörítése innen: "{$from}" ide: "{$to}" ({$progress})...
compressed = {$items} {$items ->
        [one] elem
        *[other] elem
    } tömörítve innen: "{$from}" ide: "{$to}"
copy_noun = Másolás
creating = "{$name}" létrehozása itt: "{$parent}"
created = "{$name}" létrehozva itt: "{$parent}"
copying = {$items} {$items ->
        [one] elem
        *[other] elem
    } másolása innen: "{$from}" ide: "{$to}" ({$progress})...
copied = {$items} {$items ->
        [one] elem
        *[other] elem
    } másolva innen: "{$from}" ide: "{$to}"
deleting = {$items} {$items ->
        [one] elem
        *[other] elem
    } törlése a kukából ({$progress})...
deleted = {$items} {$items ->
        [one] elem
        *[other] elem
    } törölve a kukából
emptying-trash = {trash} kiürítése ({$progress})...
emptied-trash = {trash} kiürítve
extracting = {$items} {$items ->
        [one] elem
        *[other] elem
    } kibontása innen: "{$from}" ide: "{$to}" ({$progress})...
extracted = {$items} {$items ->
        [one] elem
        *[other] elem
    } kibontva innen: "{$from}" ide: "{$to}"
setting-executable-and-launching = "{$name}" végrehajthatóvá tétele és futtatása
set-executable-and-launched = "{$name}" végrehajthatóvá lett téve és futtatva
moving = {$items} {$items ->
        [one] elem
        *[other] elem
    } áthelyezése innen: "{$from}" ide: "{$to}" ({$progress})...
moved = {$items} {$items ->
        [one] elem
        *[other] elem
    } áthelyezve innen: "{$from}" ide: "{$to}"
renaming = Átnevezés "{$from}"-ról "{$to}"-ra
renamed = Átnevezve "{$from}"-ról "{$to}"-ra
restoring = {$items} {$items ->
        [one] elem
        *[other] elem
    } visszaállítása a kukából ({$progress})...
restored = {$items} {$items ->
        [one] elem
        *[other] elem
    } visszaállítva a kukából
unknown-folder = ismeretlen mappa

## Open with
menu-open-with = Megnyitás mással...
default-app = {$name} (alapértelmezett)

## Show details
show-details = Részletek mutatása
type = Típus: {$mime}
items = Elemek: {$items}
item-size = Méret: {$size}
item-created = Létrehozva: {$created}
item-modified = Módosítva: {$modified}
item-accessed = Hozzáférve: {$accessed}
calculating = Számolás...

## Settings
settings = Beállítások
single-click = Egykattintásos megnyitás

### Appearance
appearance = Megjelenés
theme = Téma
match-desktop = Rendszertéma
dark = Sötét
light = Világos

### Type to Search
type-to-search = Gépeléssel keresés
type-to-search-recursive = A jelenlegi mappában és almappákban keres
type-to-search-enter-path = Elérési út megadása

# Context menu
add-to-sidebar = Hozzáadás az oldalsávhoz
compress = Tömörítés
delete-permanently = Végleges törlés
extract-here = Kibontás
new-file = Új fájl...
new-folder = Új mappa...
open-in-terminal = Megnyitás a terminálban
move-to-trash = Kukába helyezés
restore-from-trash = Visszaállítás a kukából
remove-from-sidebar = Eltávolítás az oldalsávról
sort-by-name = Név szerinti rendezés
sort-by-modified = Módosítás szerinti rendezés
sort-by-size = Méret szerinti rendezés
sort-by-trashed = Törlés ideje szerinti rendezés

## Desktop
change-wallpaper = Háttérkép cseréje...
desktop-appearance = Asztali megjelenés...
display-settings = Képernyő beállításai...

# Menu

## File
file = Fájl
new-tab = Új lap
new-window = Új ablak
rename = Átnevezés...
close-tab = Lap bezárása
quit = Kilépés

## Edit
edit = Szerkesztés
cut = Kivágás
copy = Másolás
paste = Beillesztés
select-all = Összes kijelölése

## View
zoom-in = Nagyítás
default-size = Alapértelmezett méret
zoom-out = Kicsinyítés
view = Nézet
grid-view = Rácsnézet
list-view = Listanézet
show-hidden-files = Rejtett fájlok megjelenítése
list-directories-first = Könyvtárak listázása először
gallery-preview = Galéria előnézet
menu-settings = Beállítások...
menu-about = A COSMIC Fájlok névjegye...

## Sort
sort = Rendezés
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Legújabb előre
sort-oldest-first = Legrégibb előre
sort-smallest-to-largest = Legkisebbtől a legnagyobbig
sort-largest-to-smallest = Legnagyobbtól a legkisebbig
