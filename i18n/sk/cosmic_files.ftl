cosmic-files = Súbory COSMIC
empty-folder = Priečinok je prázdny
empty-folder-hidden = Priečinok je prázdny (obsahuje skryté položky)
no-results = Neboli nájdené žiadne výsledky
filesystem = Súborový systém
home = Domov
networks = Siete
notification-in-progress = Prebiehajú operácie so súbormi.
trash = Kôš
recents = Nedávne
undo = Späť
today = Dnes
# Desktop view options
desktop-view-options = Možnosti zobrazenia pracovnej plochy...
show-on-desktop = Zobraziť na pracovnej ploche
desktop-folder-content = Obsah priečinka Pracovná plocha
mounted-drives = Pripojené disky
trash-folder-icon = Ikona priečinka Kôš
icon-size-and-spacing = Veľkosť ikon a rozostupy
icon-size = Veľkosť ikon
grid-spacing = Rozostupy mriežky
# List view
name = Názov
modified = Upravené
trashed-on = Vymazané
size = Veľkosť
# Progress footer
details = Podrobnosti
dismiss = Zavrieť správu
operations-running =
    { $running } { $running ->
        [one] operácia
        [few] operácie
        [many] operácií
       *[other] operácie
    } prebieha ({ $percent }%)...
operations-running-finished =
    { $running } { $running ->
        [one] operácia
        [few] operácie
        [many] operácií
       *[other] operácie
    } prebieha ({ $percent }%), { $finished } dokončených...
pause = Pozastaviť
resume = Pokračovať

# Dialogs


## Compress Dialog

create-archive = Vytvoriť archív

## Extract Dialog

extract-password-required = Vyžaduje sa heslo
extract-to = Extrahovať do...
extract-to-title = Extrahovať do priečinka

## Empty Trash Dialog

empty-trash = Vyprázdniť kôš
empty-trash-warning = Naozaj chcete trvalo odstrániť všetky položky v koši?

## Mount Error Dialog

mount-error = Nie je možné získať prístup k disku

## New File/Folder Dialog

create-new-file = Vytvoriť nový súbor
create-new-folder = Vytvoriť nový priečinok
file-name = Názov súboru
folder-name = Názov priečinka
file-already-exists = Súbor s týmto názvom už existuje.
folder-already-exists = Priečinok s týmto názvom už existuje.
name-hidden = Názvy začínajúce bodkou budú skryté.
name-invalid = Názov nemôže byť "{ $filename }".
name-no-slashes = Názov nemôže obsahovať lomítka.

## Open/Save Dialog

cancel = Zrušiť
create = Vytvoriť
open = Otvoriť
open-file = Otvoriť súbor
open-folder = Otvoriť priečinok
open-in-new-tab = Otvoriť v novej záložke
open-in-new-window = Otvoriť v novom okne
open-item-location = Otvoriť umiestnenie položky
open-multiple-files = Otvoriť viac súborov
open-multiple-folders = Otvoriť viac priečinkov
save = Uložiť
save-file = Uložiť súbor

## Open With Dialog

open-with-title = Ako chcete otvoriť "{ $name }"?
browse-store = Prehľadávať { $store }
other-apps = Iné aplikácie
related-apps = Súvisiace aplikácie

## Permanently delete Dialog

selected-items = { $items } vybraných položiek
permanently-delete-question = Trvalo odstrániť
delete = Odstrániť
permanently-delete-warning = Naozaj chcete trvalo odstrániť { $target }? Toto nie je možné vrátiť späť.

## Rename Dialog

rename-file = Premenovať súbor
rename-folder = Premenovať priečinok

## Replace Dialog

replace = Nahradiť
replace-title = "{ $filename }" už existuje v tomto umiestnení.
replace-warning = Chcete ho nahradiť tým, ktorý práve ukladáte? Nahradením sa prepíše jeho obsah.
replace-warning-operation = Chcete ho nahradiť? Nahradením sa prepíše jeho obsah.
original-file = Pôvodný súbor
replace-with = Nahradiť s
apply-to-all = Použiť na všetky
keep-both = Ponechať oboje
skip = Preskočiť

## Set as Executable and Launch Dialog

set-executable-and-launch = Nastaviť ako spustiteľné a spustiť
set-executable-and-launch-description = Chcete nastaviť "{ $name }" ako spustiteľné a spustiť ho?
set-and-launch = Nastaviť a spustiť

## Metadata Dialog

open-with = Otvoriť pomocou
owner = Vlastník
group = Skupina
other = Ostatní

### Mode 0

none = Žiadne

### Mode 1 (unusual)

execute-only = Len spúšťanie

### Mode 2 (unusual)

write-only = Len zápis

### Mode 3 (unusual)

write-execute = Zápis a spúšťanie

### Mode 4

read-only = Len čítanie

### Mode 5

read-execute = Čítanie a spúšťanie

### Mode 6

read-write = Čítanie a zápis

### Mode 7

read-write-execute = Čítanie, zápis a spúšťanie

## Favorite Path Error Dialog

favorite-path-error = Chyba pri otváraní adresára
favorite-path-error-description =
    Nepodarilo sa otvoriť "{ $path }".
    Možno neexistuje alebo nemáte povolenie na jeho otvorenie.

    Chcete ho odstrániť z bočného panela?
remove = Odstrániť
keep = Ponechať

# Context Pages


## About

git-description = Git commit { $hash } z { $date }

## Add Network Drive

add-network-drive = Pridať sieťový disk
connect = Pripojiť
connect-anonymously = Pripojiť anonymne
connecting = Pripájanie...
domain = Doména
enter-server-address = Zadajte adresu servera
network-drive-description =
    Adresy serverov obsahujú prefix protokolu a adresu.
    Príklady: ssh://192.168.0.1, ftp://[2001:db8::1]

### Make sure to keep the comma which separates the columns

network-drive-schemes =
    Dostupné protokoly,Prefix
    AppleTalk,afp://
    File Transfer Protocol,ftp:// alebo ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// alebo ssh://
    WebDAV,dav:// alebo davs://
network-drive-error = Nie je možné získať prístup k sieťovému disku
password = Heslo
remember-password = Zapamätať heslo
try-again = Skúsiť znova
username = Používateľské meno

## Operations

cancelled = Zrušené
edit-history = Upraviť históriu
history = História
no-history = Žiadne položky v histórii.
pending = Čakajúce
progress = { $percent }%
progress-cancelled = { $percent }%, zrušené
progress-paused = { $percent }%, pozastavené
failed = Zlyhalo
complete = Dokončené
compressing =
    Komprimujem { $items } { $items ->
        [one] položku
        [few] položky
        [many] položiek
       *[other] položky
    } z "{ $from }" do "{ $to }" ({ $progress })...
compressed =
    Komprimované { $items } { $items ->
        [one] položka
        [few] položky
        [many] položiek
       *[other] položky
    } z "{ $from }" do "{ $to }"
copy_noun = Kopírovať
creating = Vytváram "{ $name }" v "{ $parent }"
created = Vytvorené "{ $name }" v "{ $parent }"
copying =
    Kopírujem { $items } { $items ->
        [one] položku
        [few] položky
        [many] položiek
       *[other] položky
    } z "{ $from }" do "{ $to }" ({ $progress })...
copied =
    Skopírované { $items } { $items ->
        [one] položka
        [few] položky
        [many] položiek
       *[other] položky
    } z "{ $from }" do "{ $to }"
deleting =
    Odstraňujem { $items } { $items ->
        [one] položku
        [few] položky
        [many] položiek
       *[other] položky
    } z { trash } ({ $progress })...
deleted =
    Odstránené { $items } { $items ->
        [one] položka
        [few] položky
        [many] položiek
       *[other] položky
    } z { trash }
emptying-trash = Vyprázdňujem { trash } ({ $progress })...
emptied-trash = Kôš bol vyprázdnený
extracting =
    Extrahujem { $items } { $items ->
        [one] položku
        [few] položky
        [many] položiek
       *[other] položky
    } z "{ $from }" do "{ $to }" ({ $progress })...
extracted =
    Extrahované { $items } { $items ->
        [one] položka
        [few] položky
        [many] položiek
       *[other] položky
    } z "{ $from }" do "{ $to }"
setting-executable-and-launching = Nastavujem "{ $name }" ako spustiteľné a spúšťam
set-executable-and-launched = "{ $name }" nastavené ako spustiteľné a spustené
setting-permissions = Nastavujem oprávnenia pre "{ $name }" na { $mode }
set-permissions = Oprávnenia pre "{ $name }" nastavené na { $mode }
moving =
    Presúvam { $items } { $items ->
        [one] položku
        [few] položky
        [many] položiek
       *[other] položky
    } z "{ $from }" do "{ $to }" ({ $progress })...
moved =
    Presunuté { $items } { $items ->
        [one] položka
        [few] položky
        [many] položiek
       *[other] položky
    } z "{ $from }" do "{ $to }"
permanently-deleting =
    Trvalo odstraňujem { $items } { $items ->
        [one] položku
        [few] položky
        [many] položiek
       *[other] položky
    }
permanently-deleted =
    Trvalo odstránené { $items } { $items ->
        [one] položka
        [few] položky
        [many] položiek
       *[other] položky
    }
removing-from-recents =
    Odstraňujem { $items } { $items ->
        [one] položku
        [few] položky
        [many] položiek
       *[other] položky
    } z { recents }
removed-from-recents =
    Odstránené { $items } { $items ->
        [one] položka
        [few] položky
        [many] položiek
       *[other] položky
    } z { recents }
renaming = Premenovávam "{ $from }" na "{ $to }"
renamed = Premenované "{ $from }" na "{ $to }"
restoring =
    Obnovujem { $items } { $items ->
        [one] položku
        [few] položky
        [many] položiek
       *[other] položky
    } z { trash } ({ $progress })...
restored =
    Obnovené { $items } { $items ->
        [one] položka
        [few] položky
        [many] položiek
       *[other] položky
    } z { trash }
unknown-folder = neznámy priečinok

## Open with

menu-open-with = Otvoriť pomocou...
default-app = { $name } (predvolené)

## Show details

show-details = Zobraziť podrobnosti
type = Typ: { $mime }
items = Položky: { $items }
item-size = Veľkosť: { $size }
item-created = Vytvorené: { $created }
item-modified = Upravené: { $modified }
item-accessed = Prístup: { $accessed }
calculating = Vypočítavam...

## Settings

settings = Nastavenia
single-click = Otvoriť jedným kliknutím

### Appearance

appearance = Vzhľad
theme = Téma
match-desktop = Prispôsobiť pracovnej ploche
dark = Tmavá
light = Svetlá

### Type to Search

type-to-search = Hľadať písaním
type-to-search-recursive = Prehľadáva aktuálny priečinok a všetky podpriečinky
type-to-search-enter-path = Zadajte cestu k adresáru alebo súboru
# Context menu
add-to-sidebar = Pridať do bočného panela
compress = Komprimovať
delete-permanently = Trvalo odstrániť
eject = Vysunúť
extract-here = Extrahovať sem
new-file = Nový súbor...
new-folder = Nový priečinok...
open-in-terminal = Otvoriť v termináli
move-to-trash = Presunúť do koša
restore-from-trash = Obnoviť z koša
remove-from-sidebar = Odstrániť z bočného panela
sort-by-name = Zoradiť podľa názvu
sort-by-modified = Zoradiť podľa úpravy
sort-by-size = Zoradiť podľa veľkosti
sort-by-trashed = Zoradiť podľa času odstránenia
remove-from-recents = Odstrániť z nedávnych

## Desktop

change-wallpaper = Zmeniť tapetu...
desktop-appearance = Vzhľad pracovnej plochy...
display-settings = Nastavenia zobrazenia...

# Menu


## File

file = Súbor
new-tab = Nová záložka
new-window = Nové okno
reload-folder = Obnoviť priečinok
rename = Premenovať...
close-tab = Zavrieť záložku
quit = Ukončiť

## Edit

edit = Upraviť
cut = Vystrihnúť
copy = Kopírovať
paste = Prilepiť
select-all = Vybrať všetko

## View

zoom-in = Priblížiť
default-size = Predvolená veľkosť
zoom-out = Oddialiť
view = Zobraziť
grid-view = Zobrazenie mriežky
list-view = Zobrazenie zoznamu
show-hidden-files = Zobraziť skryté súbory
list-directories-first = Najskôr priečinky
gallery-preview = Náhľad galérie
menu-settings = Nastavenia...
menu-about = O aplikácii Súbory COSMIC...

## Sort

sort = Zoradiť
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Najnovšie najskôr
sort-oldest-first = Najstaršie najskôr
sort-smallest-to-largest = Od najmenších po najväčšie
sort-largest-to-smallest = Od najväčších po najmenšie
repository = Repozitár
support = Podpora
progress-failed = { $percent }%, zlyhalo
