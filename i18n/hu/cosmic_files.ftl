cosmic-files = COSMIC Fájlok
comment = Fájlkezelő a COSMIC asztali környezethez
keywords = mappa;fájl;kezelő;
empty-folder = Üres mappa
empty-folder-hidden = Üres mappa (rejtett elemeket tartalmaz)
no-results = Nincs találat
filesystem = Fájlrendszer
home = Saját mappa
networks = Hálózatok
notification-in-progress = A fájlműveletek folyamatban vannak
trash = Kuka
recents = Legutóbbiak
undo = Visszavonás
today = Ma
# Desktop view options
desktop-view-options = Asztali nézet beállításai…
show-on-desktop = Megjelenítés az asztalon
desktop-folder-content = Asztal mappa tartalma
mounted-drives = Csatolt meghajtók
trash-folder-icon = Kuka ikon
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
dismiss = Üzenet bezárása
operations-running =
    { $running } { $running ->
        [one] művelet
       *[other] művelet
    } fut ({ $percent }%)…
operations-running-finished =
    { $running } { $running ->
        [one] művelet
       *[other] művelet
    } fut ({ $percent }%), { $finished } befejeződött…
pause = Szünet
resume = Folytatás

# Dialogs


## Compress Dialog

create-archive = Archívum létrehozása

## Extract Dialog

extract-password-required = Jelszó szükséges
extract-to = Kibontás ide…
extract-to-title = Kibontási cél kiválasztása

## Empty Trash Dialog

empty-trash = A Kuka ürítése
empty-trash-warning = A Kukában lévő összes elem véglegesen törölve lesz

## Mount Error Dialog

mount-error = Nem érhető el a meghajtó

## New File/Folder Dialog

create-new-file = Új fájl létrehozása
create-new-folder = Új mappa létrehozása
file-name = Fájlnév
folder-name = Mappa neve
file-already-exists = Már létezik ilyen nevű fájl
folder-already-exists = Már létezik ilyen nevű mappa
name-hidden = A ponttal kezdődő nevek rejtve lesznek
name-invalid = A név nem lehet „{ $filename }”
name-no-slashes = A név nem tartalmazhat „/” jelet

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

open-with-title = Hogyan szeretnéd megnyitni a(z) „{ $name }” fájlt?
browse-store = { $store } böngészése
other-apps = Egyéb alkalmazások
related-apps = Hasonló alkalmazások

## Permanently delete Dialog

selected-items = A(z) { $items } kijelölt elem
permanently-delete-question = Végleges törlés?
delete = Törlés
permanently-delete-warning = { $target } véglegesen törölve lesz. A művelet nem vonható vissza.

## Rename Dialog

rename-file = Fájl átnevezése
rename-folder = Mappa átnevezése

## Replace Dialog

replace = Csere
replace-title = „{ $filename }” már létezik ezen a helyen
replace-warning = Szeretnéd lecserélni a meglévő fájlt a mentendő fájllal? A cserével felülírod annak tartalmát.
replace-warning-operation = Szeretnéd lecserélni? A csere felülírja annak tartalmát.
original-file = Eredeti fájl
replace-with = Csere erre
apply-to-all = Alkalmazás mindegyikre
keep-both = Mindkettő megtartása
skip = Kihagyás

## Set as Executable and Launch Dialog

set-executable-and-launch = Végrehajthatóvá tétel és indítás
set-executable-and-launch-description = Szeretnéd végrehajthatóvá tenni a(z) „{ $name }” fájlt és elindítani?
set-and-launch = Beállítás és indítás

## Metadata Dialog

open-with = Megnyitás ezzel
owner = Tulajdonos
group = Csoport
other = Mások

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
    Nem sikerült megnyitni ezt: „{ $path }”
    A(z) „{ $path }” útvonal lehet, hogy nem létezik, vagy nincs jogosultságod a megnyitásához

    Szeretnéd eltávolítani az oldalsávról?
remove = Eltávolítás
keep = Megtartás

# Context Pages


## About


## Add Network Drive

add-network-drive = Hálózati meghajtó hozzáadása
connect = Kapcsolódás
connect-anonymously = Kapcsolódás névtelenül
connecting = Kapcsolódás…
domain = Tartomány
enter-server-address = Add meg a kiszolgáló címét
network-drive-description =
    A kiszolgálócímek tartalmazzák a protokoll előtagját és a címet.
    Például: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Elérhető protokollok,Előtag
    AppleTalk,afp://
    File Transfer Protocol,ftp:// vagy ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// vagy ssh://
    WebDAV,dav:// vagy davs://
network-drive-error = Nem érhető el a hálózati meghajtó
password = Jelszó
remember-password = Jelszó megjegyzése
try-again = Próbáld újra
username = Felhasználónév

## Operations

cancelled = Megszakítva
edit-history = Fájlműveleti előzmények
history = Előzmények
no-history = Nem találhatók elemek az előzményekben.
pending = Függőben
progress = { $percent }%
progress-cancelled = { $percent }%, megszakítva
progress-paused = { $percent }%, szüneteltetve
failed = Sikertelen
complete = Befejezett
compressing =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } tömörítése innen: „{ $from }” ide: „{ $to }” ({ $progress })…
compressed =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } tömörítve innen: „{ $from }” ide: „{ $to }”
copy_noun = Másolat
creating = „{ $name }” létrehozása itt: „{ $parent }”
created = „{ $name }” létrehozva itt: „{ $parent }”
copying =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } másolása innen: „{ $from }” ide: „{ $to }” ({ $progress })…
copied =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } másolva innen: „{ $from }” ide: „{ $to }”
deleting =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } törlése a Kukából ({ $progress })…
deleted =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } törölve a Kukából
emptying-trash = { trash } ürítése ({ $progress })…
emptied-trash = { trash } kiürítve
extracting =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } kibontása innen: „{ $from }” ide: „{ $to }” ({ $progress })…
extracted =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } kibontva innen: „{ $from }” ide: „{ $to }”
setting-executable-and-launching = „{ $name }” végrehajthatóvá tétele és futtatása
set-executable-and-launched = „{ $name }” végrehajthatóvá lett téve és futtatva
setting-permissions = „{ $name }” jogosultságainak beállítása: { $mode }
set-permissions = „{ $name }” jogosultságai beállítva: { $mode }
moving =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } áthelyezése innen: „{ $from }” ide: „{ $to }” ({ $progress })…
moved =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } áthelyezve innen: „{ $from }” ide: „{ $to }”
permanently-deleting =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } végleges törlése
permanently-deleted =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } véglegesen törölve
removing-from-recents =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } eltávolítása a { recents }ból
removed-from-recents =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } eltávolítva a { recents }ból
renaming = „{ $from }” átnevezése erre: „{ $to }”
renamed = „{ $from }” átnevezve erre: „{ $to }”
restoring =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } visszaállítása a Kukából ({ $progress })…
restored =
    { $items } { $items ->
        [one] elem
       *[other] elem
    } visszaállítva a Kukából
unknown-folder = ismeretlen mappa

## Open with

menu-open-with = Megnyitás mással…
default-app = { $name } (alapértelmezett)

## Show details

show-details = Részletek megjelenítése
type = Típus: { $mime }
items = Elemek: { $items }
item-size = Méret: { $size }
item-created = Létrehozva: { $created }
item-modified = Módosítva: { $modified }
item-accessed = Utolsó hozzáférés: { $accessed }
calculating = Számítás…

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
type-to-search-recursive = Keresés a jelenlegi mappában és az almappákban
type-to-search-enter-path = Elérési út megnyitása
# Context menu
add-to-sidebar = Hozzáadás az oldalsávhoz
compress = Tömörítés…
delete-permanently = Végleges törlés
eject = Kiadás
extract-here = Kibontás
new-file = Új fájl…
new-folder = Új mappa…
open-in-terminal = Megnyitás a terminálban
move-to-trash = Áthelyezés a Kukába
restore-from-trash = Visszaállítás a Kukából
remove-from-sidebar = Eltávolítás az oldalsávról
sort-by-name = Név szerinti rendezés
sort-by-modified = Módosítás szerinti rendezés
sort-by-size = Méret szerinti rendezés
sort-by-trashed = Törlés ideje szerinti rendezés
remove-from-recents = Eltávolítás a legutóbbiak közül

## Desktop

change-wallpaper = Háttérkép cseréje…
desktop-appearance = Asztal megjelenése…
display-settings = Kijelzőbeállítások…

# Menu


## File

file = Fájl
new-tab = Új lap
new-window = Új ablak
reload-folder = Mappa újratöltése
rename = Átnevezés…
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
gallery-preview = Galéria-előnézet
menu-settings = Beállítások…
menu-about = A COSMIC Fájlok névjegye…

## Sort

sort = Rendezés
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Legújabb elöl
sort-oldest-first = Legrégibb elöl
sort-smallest-to-largest = Legkisebbtől a legnagyobbig
sort-largest-to-smallest = Legnagyobbtól a legkisebbig
repository = Tároló
support = Támogatás
progress-failed = { $percent }%, sikertelen
empty-trash-title = Kiüríted a Kukát?
type-to-search-select = Kijelöli az első egyező fájlt vagy mappát
pasted-image = Beillesztett kép
pasted-text = Beillesztett szöveg
pasted-video = Beillesztett videó
copy-to-title = Másolási cél kiválasztása
copy-to-button-label = Másolás
move-to-title = Áthelyezési cél kiválasztása
move-to-button-label = Áthelyezés
copy-to = Másolás ide…
move-to = Áthelyezés ide…
show-recents = Legutóbbiak mappa megjelenítése az oldalsávban
copy-path = Útvonal másolása
clear-recents-history = Legutóbbiak előzményének törlése
mixed = Vegyes
context-action = Helyi művelet
context-action-confirm-title = Futtatod ezt: „{ $name }”?
context-action-confirm-warning =
    Ez a művelet { $items } { $items ->
        [one] elemen
       *[other] elemen
    } fog lefutni.
run = Futtatás
rename-confirm = Átnevezés
